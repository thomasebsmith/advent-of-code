use std::collections::{HashMap, HashSet, VecDeque};
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
    Up,
    Down,
    Left,
    Right,
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

    fn neighbors(self) -> impl Iterator<Item = Self> {
        Direction::ALL
            .into_iter()
            .map(|direction| self.move_one(direction))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

struct Map {
    heights: Vec<Vec<i64>>,
    width: isize,
    height: isize,
}

impl Map {
    fn new<R: io::Read>(reader: io::BufReader<R>) -> io::Result<Self> {
        let mut width: Option<isize> = None;
        let mut heights: Vec<Vec<i64>> = Vec::new();
        for line in reader.lines() {
            let line = line?;
            let line_heights = line
                .chars()
                .map(|ch| ch.to_digit(10).map(|n| n as i64))
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| invalid_input("Non-digit character"))?;
            if let Some(current_width) = width {
                if current_width != line_heights.len() as isize {
                    return Err(invalid_input("Mismatched widths"));
                }
            } else {
                width = Some(line_heights.len() as isize);
            }

            heights.push(line_heights);
        }

        let Some(width) = width else {
            return Err(invalid_input("No lines"));
        };
        let height = heights.len() as isize;

        Ok(Self {
            heights,
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

    fn height_at(&self, position: Position) -> i64 {
        const DEFAULT_HEIGHT: i64 = i64::MAX;
        if !self.in_bounds(position) {
            DEFAULT_HEIGHT
        } else {
            self.heights[position.row as usize][position.col as usize]
        }
    }

    fn potential_trailheads(&self) -> Vec<Position> {
        let mut height_zero_positions = Vec::<Position>::new();

        for row in 0..self.height {
            for col in 0..self.width {
                let position = Position { row, col };
                if self.height_at(position) == 0 {
                    height_zero_positions.push(position);
                }
            }
        }

        height_zero_positions
    }

    fn num_trails_from_position(&self, position: Position) -> usize {
        const DESTINATION_HEIGHT: i64 = 9;

        let mut num_trails = 0usize;

        let mut visited = HashSet::<Position>::new();
        let mut to_visit = VecDeque::<Position>::new();
        to_visit.push_back(position);

        while let Some(this_position) = to_visit.pop_front() {
            if visited.contains(&this_position) {
                continue;
            }
            visited.insert(this_position);

            let this_height = self.height_at(this_position);
            if this_height == DESTINATION_HEIGHT {
                num_trails += 1;
                continue;
            }

            let next_height = this_height + 1;
            to_visit.extend(
                this_position.neighbors().filter(|neighbor| {
                    self.height_at(*neighbor) == next_height
                }),
            );
        }

        num_trails
    }

    fn sum_of_all_trailhead_scores(&self) -> usize {
        self.potential_trailheads()
            .into_iter()
            .map(|position| self.num_trails_from_position(position))
            .sum()
    }

    fn sum_of_all_trailhead_ratings(&self) -> usize {
        let mut unique_ways = 0usize;
        let mut old_ways_to_reach = HashMap::<Position, usize>::new();
        for height in 0..=9 {
            let mut ways_to_reach = HashMap::<Position, usize>::new();
            unique_ways = 0;
            for row in 0..self.height {
                for col in 0..self.width {
                    let position = Position { row, col };
                    if self.height_at(position) == height {
                        let ways_to_reach_here = if height == 0 {
                            1
                        } else {
                            position
                                .neighbors()
                                .filter_map(|neighbor| {
                                    old_ways_to_reach.get(&neighbor)
                                })
                                .sum()
                        };
                        ways_to_reach.insert(position, ways_to_reach_here);
                        unique_ways += ways_to_reach_here
                    }
                }
            }
            old_ways_to_reach = ways_to_reach;
        }

        unique_ways
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let map = Map::new(reader)?;
    let result = match part {
        Part::Part1 => map.sum_of_all_trailhead_scores(),
        Part::Part2 => map.sum_of_all_trailhead_ratings(),
    };
    println!("{result}");

    Ok(())
}
