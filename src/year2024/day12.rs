use std::collections::{HashSet, VecDeque};
use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::part::Part;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Position {
    row: isize,
    col: isize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Direction {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

impl Direction {
    const ALL: [Self; 4] = [Self::Up, Self::Down, Self::Left, Self::Right];
}

impl Position {
    fn move_one(self, direction: Direction) -> Self {
        let row = self.row;
        let col = self.col;
        match direction {
            Direction::Up => Self { row: row - 1, col },
            Direction::Down => Self { row: row + 1, col },
            Direction::Left => Self { row, col: col - 1 },
            Direction::Right => Self { row, col: col + 1 },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Plot {
    plant: char,
}

struct Map {
    plots: Vec<Vec<Plot>>,
    width: isize,
    height: isize,
}

impl Map {
    fn new<R: io::Read>(reader: io::BufReader<R>) -> io::Result<Self> {
        let mut width: Option<isize> = None;
        let mut plots: Vec<Vec<Plot>> = Vec::new();
        for line in reader.lines() {
            let line = line?;
            let line_plots =
                line.chars().map(|plant| Plot { plant }).collect::<Vec<_>>();
            if let Some(current_width) = width {
                if current_width != line_plots.len() as isize {
                    return Err(invalid_input("Mismatched widths"));
                }
            } else {
                width = Some(line_plots.len() as isize);
            }

            plots.push(line_plots);
        }

        let Some(width) = width else {
            return Err(invalid_input("No lines"));
        };
        let height = plots.len() as isize;

        Ok(Self {
            plots,
            width,
            height,
        })
    }

    fn in_bounds(&self, position: Position) -> bool {
        position.row >= 0
            && position.row < self.height
            && position.col >= 0
            && position.col < self.width
    }

    fn at(&self, position: Position) -> Option<Plot> {
        if !self.in_bounds(position) {
            None
        } else {
            Some(self.plots[position.row as usize][position.col as usize])
        }
    }

    fn total_price(&self, part: Part) -> i64 {
        let mut price_sum = 0i64;
        let mut visited = HashSet::<Position>::new();

        let mut visit = |position: Position| {
            let Some(target_value) = self.at(position) else {
                return;
            };

            if visited.contains(&position) {
                return;
            }

            let mut to_visit = VecDeque::<Position>::new();
            to_visit.push_back(position);

            let mut area = 0i64;
            let mut perimeter = 0i64;
            let mut sides = 0i64;

            while let Some(this_position) = to_visit.pop_front() {
                if !visited.insert(this_position) {
                    continue;
                }

                area += 1;

                let neighbors_with_same_value = Direction::ALL
                    .into_iter()
                    .enumerate()
                    .map(|(i, direction)| {
                        (i, this_position.move_one(direction))
                    })
                    .filter(|&(_, neighbor)| {
                        self.at(neighbor) == Some(target_value)
                    })
                    .collect::<Vec<_>>();

                perimeter += 4 - neighbors_with_same_value.len() as i64;

                let is_different_value_by_direction = Direction::ALL
                    .into_iter()
                    .enumerate()
                    .map(|(i, direction)| {
                        (i, this_position.move_one(direction))
                    })
                    .map(|(_, neighbor)| {
                        self.at(neighbor) != Some(target_value)
                    })
                    .collect::<Vec<bool>>();
                let diagonal_filled = |d1, d2| {
                    self.at(this_position.move_one(d1).move_one(d2))
                        == Some(target_value)
                };
                let num_new_sides = match &is_different_value_by_direction[..] {
                    &[true, true, true, true] => 4,
                    &[false, true, true, true] => 2,
                    &[true, false, true, true] => 2,
                    &[true, true, false, true] => 2,
                    &[true, true, true, false] => 2,
                    &[false, false, true, true] => 0,
                    &[true, true, false, false] => 0,
                    &[false, true, false, true] => {
                        1 + (if diagonal_filled(Direction::Up, Direction::Left)
                        {
                            0
                        } else {
                            1
                        })
                    }
                    &[false, true, true, false] => {
                        1 + (if diagonal_filled(Direction::Up, Direction::Right)
                        {
                            0
                        } else {
                            1
                        })
                    }
                    &[true, false, false, true] => {
                        1 + (if diagonal_filled(
                            Direction::Down,
                            Direction::Left,
                        ) {
                            0
                        } else {
                            1
                        })
                    }
                    &[true, false, true, false] => {
                        1 + (if diagonal_filled(
                            Direction::Down,
                            Direction::Right,
                        ) {
                            0
                        } else {
                            1
                        })
                    }
                    &[true, false, false, false] => {
                        (if diagonal_filled(Direction::Down, Direction::Left) {
                            0
                        } else {
                            1
                        }) + (if diagonal_filled(
                            Direction::Down,
                            Direction::Right,
                        ) {
                            0
                        } else {
                            1
                        })
                    }
                    &[false, true, false, false] => {
                        (if diagonal_filled(Direction::Up, Direction::Left) {
                            0
                        } else {
                            1
                        }) + (if diagonal_filled(
                            Direction::Up,
                            Direction::Right,
                        ) {
                            0
                        } else {
                            1
                        })
                    }
                    &[false, false, true, false] => {
                        (if diagonal_filled(Direction::Right, Direction::Up) {
                            0
                        } else {
                            1
                        }) + (if diagonal_filled(
                            Direction::Right,
                            Direction::Down,
                        ) {
                            0
                        } else {
                            1
                        })
                    }
                    &[false, false, false, true] => {
                        (if diagonal_filled(Direction::Left, Direction::Up) {
                            0
                        } else {
                            1
                        }) + (if diagonal_filled(
                            Direction::Left,
                            Direction::Down,
                        ) {
                            0
                        } else {
                            1
                        })
                    }
                    &[false, false, false, false] => [
                        (Direction::Up, Direction::Left),
                        (Direction::Up, Direction::Right),
                        (Direction::Down, Direction::Left),
                        (Direction::Down, Direction::Right),
                    ]
                    .into_iter()
                    .map(|(d1, d2)| if diagonal_filled(d1, d2) { 0 } else { 1 })
                    .sum(),
                    _ => 0,
                };
                sides += num_new_sides;

                to_visit.extend(
                    neighbors_with_same_value
                        .into_iter()
                        .map(|(_, neighbor)| neighbor),
                );
            }

            price_sum += area
                * match part {
                    Part::Part1 => perimeter,
                    Part::Part2 => sides,
                };
        };

        for row in 0..self.height {
            for col in 0..self.width {
                let position = Position { row, col };
                visit(position);
            }
        }

        price_sum
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let map = Map::new(reader)?;

    let result = map.total_price(part);

    println!("{result}");

    Ok(())
}
