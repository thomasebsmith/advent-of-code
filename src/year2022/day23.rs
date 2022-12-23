use std::collections::{HashMap, HashSet, VecDeque};
use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::part::Part;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Point {
    pub x: i64,
    pub y: i64,
}

impl Point {
    pub fn moved_by(self, movement: Movement) -> Self {
        match movement {
            Movement::N => Self {
                x: self.x,
                y: self.y - 1,
            },
            Movement::S => Self {
                x: self.x,
                y: self.y + 1,
            },
            Movement::W => Self {
                x: self.x - 1,
                y: self.y,
            },
            Movement::E => Self {
                x: self.x + 1,
                y: self.y,
            },
            Movement::NW => self.moved_by(Movement::N).moved_by(Movement::W),
            Movement::NE => self.moved_by(Movement::N).moved_by(Movement::E),
            Movement::SW => self.moved_by(Movement::S).moved_by(Movement::W),
            Movement::SE => self.moved_by(Movement::S).moved_by(Movement::E),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Movement {
    N,
    NW,
    NE,
    S,
    SW,
    SE,
    W,
    E,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MovementCategory {
    North,
    South,
    West,
    East,
}

impl MovementCategory {
    pub fn movements(self) -> Vec<Movement> {
        match self {
            Self::North => vec![Movement::N, Movement::NE, Movement::NW],
            Self::South => vec![Movement::S, Movement::SE, Movement::SW],
            Self::West => vec![Movement::W, Movement::NW, Movement::SW],
            Self::East => vec![Movement::E, Movement::NE, Movement::SE],
        }
    }

    pub fn primary_movement(self) -> Movement {
        match self {
            Self::North => Movement::N,
            Self::South => Movement::S,
            Self::West => Movement::W,
            Self::East => Movement::E,
        }
    }
}

struct Map {
    elves: HashSet<Point>,
    proposal_order: VecDeque<MovementCategory>,
}

impl Map {
    pub fn new() -> Self {
        let mut proposal_order = VecDeque::<MovementCategory>::new();
        proposal_order.push_back(MovementCategory::North);
        proposal_order.push_back(MovementCategory::South);
        proposal_order.push_back(MovementCategory::West);
        proposal_order.push_back(MovementCategory::East);

        Self {
            elves: HashSet::new(),
            proposal_order,
        }
    }

    pub fn add_elf(&mut self, location: Point) {
        self.elves.insert(location);
    }

    pub fn score(&self) -> i64 {
        let mut min_x: Option<i64> = None;
        let mut min_y: Option<i64> = None;
        let mut max_x: Option<i64> = None;
        let mut max_y: Option<i64> = None;

        for elf in &self.elves {
            match min_x {
                None => {
                    min_x = Some(elf.x);
                }
                Some(min_x_num) => {
                    if elf.x < min_x_num {
                        min_x = Some(elf.x);
                    }
                }
            }
            match min_y {
                None => {
                    min_y = Some(elf.y);
                }
                Some(min_y_num) => {
                    if elf.y < min_y_num {
                        min_y = Some(elf.y);
                    }
                }
            }
            match max_x {
                None => {
                    max_x = Some(elf.x);
                }
                Some(max_x_num) => {
                    if elf.x > max_x_num {
                        max_x = Some(elf.x);
                    }
                }
            }
            match max_y {
                None => {
                    max_y = Some(elf.y);
                }
                Some(max_y_num) => {
                    if elf.y > max_y_num {
                        max_y = Some(elf.y);
                    }
                }
            }
        }

        // TODO unwrap
        let (min_x, min_y, max_x, max_y) = (
            min_x.unwrap(),
            min_y.unwrap(),
            max_x.unwrap(),
            max_y.unwrap(),
        );

        let num_elves: i64 = self.elves.len().try_into().unwrap();

        (max_x - min_x + 1) * (max_y - min_y + 1) - num_elves
    }

    pub fn run_round(&mut self) -> bool {
        // Create proposals

        // Map from new location to original locations
        let mut proposals = HashMap::<Point, Vec<Point>>::new();
        for elf in &self.elves {
            let neighbors = self
                .proposal_order
                .iter()
                .map(|category| {
                    (
                        elf.moved_by(category.primary_movement()),
                        category
                            .movements()
                            .iter()
                            .map(|direction| elf.moved_by(*direction))
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<Vec<_>>();

            if !neighbors.iter().any(|(_, spots)| {
                spots.iter().any(|spot| self.elves.contains(spot))
            }) {
                // Don't move the elf if no one is nearby.
                continue;
            }

            for (spot, to_check) in neighbors {
                if !to_check
                    .into_iter()
                    .any(|neighbor| self.elves.contains(&neighbor))
                {
                    proposals.entry(spot).or_default().push(*elf);
                    break;
                }
            }
        }

        // Run proposals
        let mut elf_moved = false;
        for (proposed_spot, elves) in proposals {
            if elves.len() != 1 {
                continue;
            }

            let elf = elves[0];

            self.elves.remove(&elf);
            self.elves.insert(proposed_spot);
            elf_moved = true;
        }

        // Update proposal order
        let old_front = self.proposal_order.pop_front().unwrap();
        self.proposal_order.push_back(old_front);

        elf_moved
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut map = Map::new();

    for (y, line) in reader.lines().enumerate() {
        let line = line?;
        for (x, ch) in line.chars().enumerate() {
            if ch == '#' {
                map.add_elf(Point {
                    x: x.try_into().map_err(invalid_input)?,
                    y: y.try_into().map_err(invalid_input)?,
                });
            }
        }
    }

    match part {
        Part::Part1 => {
            for _ in 0..10 {
                map.run_round();
            }
            println!("{}", map.score());
        }
        Part::Part2 => {
            let mut round: usize = 1;
            while map.run_round() {
                round += 1;
            }
            println!("{}", round);
        }
    }

    Ok(())
}
