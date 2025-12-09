use std::cmp::{max, min};
use std::collections::{BTreeMap, VecDeque};
use std::io;

use crate::cellmap::{Cell, CellMap, Direction, Position};
use crate::errors::invalid_input;
use crate::parse::{lines, parse_all};
use crate::part::Part;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum Tile {
    Red,
    Green,
    Other,
}

impl Cell for Tile {
    fn from_char(ch: char) -> Option<Self> {
        match ch {
            '#' => Some(Self::Red),
            '.' => Some(Self::Other),
            _ => None,
        }
    }

    fn to_char(self) -> char {
        match self {
            Self::Red => '#',
            Self::Other => '.',
            Self::Green => 'X',
        }
    }
}

fn area_with_corners(pos_1: Position, pos_2: Position) -> isize {
    ((pos_1.row - pos_2.row).abs() + 1) * ((pos_1.col - pos_2.col).abs() + 1)
}

fn largest_area(possible_corners: &Vec<Position>) -> isize {
    let mut max_area = 0isize;
    for i in 0..possible_corners.len() {
        let corner_1 = possible_corners[i];
        for j in i + 1..possible_corners.len() {
            let corner_2 = possible_corners[j];
            max_area = max(max_area, area_with_corners(corner_1, corner_2));
        }
    }

    max_area
}

fn largest_redgreen_area(red: &Vec<Position>) -> io::Result<isize> {
    if red.is_empty() {
        return Ok(0);
    }

    let mut rows_old_to_new = BTreeMap::<isize, isize>::new();
    let mut cols_old_to_new = BTreeMap::<isize, isize>::new();
    for &position in red {
        rows_old_to_new.insert(position.row, 0);
        cols_old_to_new.insert(position.col, 0);
    }
    let mut new_row_counter = 0isize;
    let mut new_col_counter = 0isize;
    for (_, value) in rows_old_to_new.iter_mut() {
        *value = new_row_counter;
        new_row_counter += 1;
    }
    for (_, value) in cols_old_to_new.iter_mut() {
        *value = new_col_counter;
        new_col_counter += 1;
    }

    let translate = |old_position: Position| -> Position {
        Position {
            row: *rows_old_to_new.get(&old_position.row).unwrap(),
            col: *cols_old_to_new.get(&old_position.col).unwrap(),
        }
    };

    let mut map = CellMap::<Tile>::filled_with(
        Tile::Other,
        new_col_counter as usize,
        new_row_counter as usize,
    );

    let mut last_position = translate(*red.last().unwrap());
    let mut net_right_turns = 0isize;
    let mut last_direction: Option<Direction> = None;
    for old_red in red.iter() {
        let new_red = translate(*old_red);

        *map.at_mut(new_red).unwrap() = Tile::Red;

        let Some((line, direction)) = last_position.straight_line_to(new_red)
        else {
            return Err(invalid_input(
                "Consecutive red tiles don't connect in a straight line",
            ));
        };
        if let Some(last_direction) = last_direction {
            let turn_direction = last_direction.turn_direction(direction);
            match turn_direction {
                Direction::Left => {
                    net_right_turns -= 1;
                }
                Direction::Right => {
                    net_right_turns += 1;
                }
                Direction::Up => { /* ignore */ }
                Direction::Down => {
                    return Err(invalid_input("Immediate turn around"));
                }
            }
        }

        for position in line {
            let tile_ref = map.at_mut(position).unwrap();
            if *tile_ref == Tile::Other {
                *tile_ref = Tile::Green;
            }
        }

        last_position = new_red;
        last_direction = Some(direction);
    }

    // Loose bounds, but we miss the turn from the last connection as we loop,
    // so the net turns can be 3-5
    let is_clockwise = if net_right_turns == 3 || net_right_turns == 5 {
        true
    } else if net_right_turns == -3 || net_right_turns == -5 {
        false
    } else {
        return Err(invalid_input(format!(
            "Bad shape (net right turns: {})",
            net_right_turns
        )));
    };

    let mut to_make_green = VecDeque::<Position>::new();
    let mut last_position = translate(*red.last().unwrap());
    for old_red in red.iter() {
        let new_red = translate(*old_red);
        let Some((line, direction)) = last_position.straight_line_to(new_red)
        else {
            return Err(invalid_input(
                "Consecutive red tiles don't connect in a straight line",
            ));
        };
        for line_pos in line {
            let green_pos = line_pos.move_one(if is_clockwise {
                direction.turn_right()
            } else {
                direction.turn_left()
            });
            to_make_green.push_back(green_pos);
        }
        last_position = new_red;
    }

    while let Some(new_green_pos) = to_make_green.pop_front() {
        let contents_mut = map.at_mut(new_green_pos).unwrap();
        if *contents_mut != Tile::Other {
            continue;
        }

        *contents_mut = Tile::Green;
        to_make_green.extend(new_green_pos.four_neighbors());
    }

    let mut max_area = 0isize;
    for i in 0..red.len() {
        let old_corner_1 = red[i];
        let new_corner_1 = translate(old_corner_1);
        'j_loop: for j in i + 1..red.len() {
            let old_corner_2 = red[j];
            let new_corner_2 = translate(old_corner_2);

            let new_area = area_with_corners(old_corner_1, old_corner_2);
            if new_area <= max_area {
                continue;
            }

            let min_row = min(new_corner_1.row, new_corner_2.row);
            let max_row = max(new_corner_1.row, new_corner_2.row);
            let min_col = min(new_corner_1.col, new_corner_2.col);
            let max_col = max(new_corner_1.col, new_corner_2.col);
            for row in min_row..=max_row {
                for col in min_col..=max_col {
                    if map.at(Position { row, col }).unwrap() == Tile::Other {
                        continue 'j_loop;
                    }
                }
            }

            max_area = new_area;
        }
    }

    Ok(max_area)
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let red = lines(reader)?
        .map(|line| {
            let &[col, row] = &parse_all::<_, isize>(line.split(','))?[..]
            else {
                return Err(invalid_input("Expected col,row"));
            };
            Ok(Position { row, col })
        })
        .collect::<io::Result<Vec<_>>>()?;

    let result = match part {
        Part::Part1 => largest_area(&red),
        Part::Part2 => largest_redgreen_area(&red)?,
    };

    println!("{result}");

    Ok(())
}
