use std::cmp::{max, min};
use std::io;
use std::io::BufRead;
use std::ops::Index;

use crate::errors::invalid_input;
use crate::iter::{consecutive_sequences, n_elements};
use crate::part::Part;

#[derive(Clone, Copy, PartialEq, Eq)]
enum BlockType {
    Air,
    Rock,
    Sand,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Point {
    pub x: isize,
    pub y: isize,
}

impl Point {
    pub fn down(self) -> Self {
        Self {
            x: self.x,
            y: self.y + 1,
        }
    }

    pub fn down_left(self) -> Self {
        Self {
            x: self.x - 1,
            y: self.y + 1,
        }
    }

    pub fn down_right(self) -> Self {
        Self {
            x: self.x + 1,
            y: self.y + 1,
        }
    }

    pub fn unsigned_x(self) -> Option<usize> {
        self.x.try_into().ok()
    }

    pub fn unsigned_y(self) -> Option<usize> {
        self.y.try_into().ok()
    }
}

struct Chunk {
    data: [[BlockType; Self::WIDTH]; Self::HEIGHT],
    active_sand_location: Option<Point>,
}

enum SandUpdate {
    Moving,
    AtRest,
    Abyss,
}

impl Chunk {
    pub const WIDTH: usize = 1024;
    pub const HEIGHT: usize = 1024;

    // TODO: Figure out a better way of doing this
    pub const SIGNED_WIDTH: isize = 1024;
    pub const SIGNED_HEIGHT: isize = 1024;

    pub fn new() -> Self {
        Self {
            data: [[BlockType::Air; Self::WIDTH]; Self::HEIGHT],
            active_sand_location: None,
        }
    }

    fn mut_block(&mut self, location: Point) -> &mut BlockType {
        &mut self.data[location.unsigned_y().unwrap()]
            [location.unsigned_x().unwrap()]
    }

    pub fn fill_with_rock(&mut self, from: Point, to: Point) {
        if from.x == to.x {
            let x = from.x;
            let y_range = min(from.y, to.y)..=max(from.y, to.y);
            for y in y_range {
                *self.mut_block(Point { x, y }) = BlockType::Rock;
            }
        } else if from.y == to.y {
            let y = from.y;
            let x_range = min(from.x, to.x)..=max(from.x, to.x);
            for x in x_range {
                *self.mut_block(Point { x, y }) = BlockType::Rock;
            }
        } else {
            panic!("Diagonal lines are not supported");
        }
    }

    pub fn spawn_sand(&mut self, location: Point) -> io::Result<()> {
        if !self.active_sand_location.is_none() {
            Err(invalid_input(
                "Cannot spawn sand while sand is still active",
            ))?
        }

        let block = self.mut_block(location);
        if *block != BlockType::Air {
            Err(invalid_input("Cannot spawn sand unless there is air"))?
        }
        *block = BlockType::Sand;
        self.active_sand_location = Some(location);

        Ok(())
    }

    pub fn simulate_gravity(&mut self) -> SandUpdate {
        let Some(location) = self.active_sand_location else {
            return SandUpdate::AtRest;
        };

        for candidate_location in
            [location.down(), location.down_left(), location.down_right()]
        {
            if candidate_location.y >= Self::SIGNED_HEIGHT {
                *self.mut_block(location) = BlockType::Air;
                self.active_sand_location = None;
                return SandUpdate::Abyss;
            }

            if candidate_location.x < 0
                || candidate_location.x >= Self::SIGNED_WIDTH
            {
                continue;
            }

            let contents = self[candidate_location];
            if contents == BlockType::Air {
                *self.mut_block(location) = BlockType::Air;
                *self.mut_block(candidate_location) = BlockType::Sand;
                self.active_sand_location = Some(candidate_location);
                return SandUpdate::Moving;
            }
        }

        // Everything below us is rock or sand. This sand granule is done
        // moving.
        self.active_sand_location = None;
        return SandUpdate::AtRest;
    }
}

impl Index<Point> for Chunk {
    type Output = BlockType;

    fn index(&self, index: Point) -> &Self::Output {
        &self.data[index.unsigned_y().unwrap()][index.unsigned_x().unwrap()]
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut chunk = Chunk::new();

    let mut floor_y: isize = 2;

    for line in reader.lines() {
        let line = line?;
        let points = line
            .split(" -> ")
            .map(|point_str| -> io::Result<Point> {
                let coords = n_elements(2, point_str.split(','))
                    .ok_or_else(|| invalid_input("Expected 2 coordinates"))?;
                Ok(Point {
                    x: coords[0].parse().map_err(invalid_input)?,
                    y: coords[1].parse().map_err(invalid_input)?,
                })
            })
            .collect::<io::Result<Vec<Point>>>()?;

        if let Some(y) = points.iter().map(|point| point.y + 2).max() {
            floor_y = max(floor_y, y);
        }

        for points in consecutive_sequences(2, points.into_iter()) {
            chunk.fill_with_rock(points[0], points[1]);
        }
    }

    let spawn_location = Point { x: 500, y: 0 };

    match part {
        Part::Part1 => {
            let mut num_sand_granules_spawned = 0;
            loop {
                match chunk.simulate_gravity() {
                    SandUpdate::AtRest => {
                        // Start a new granule
                        num_sand_granules_spawned += 1;
                        chunk.spawn_sand(spawn_location)?;
                    }
                    SandUpdate::Moving => {
                        // Continue
                    }
                    SandUpdate::Abyss => {
                        break;
                    }
                }
            }

            // - 1 since the last spawned one fell into the abyss
            println!("{}", num_sand_granules_spawned - 1);
        }
        Part::Part2 => {
            // TODO: What if floor_y is too low for the chunk size?
            chunk.fill_with_rock(
                Point { x: 0, y: floor_y },
                Point {
                    x: Chunk::SIGNED_WIDTH - 1,
                    y: floor_y,
                },
            );

            let mut num_sand_granules_spawned = 0;
            loop {
                match chunk.simulate_gravity() {
                    SandUpdate::AtRest => {
                        // If there's sand at the spawn location, we're done.
                        if chunk[spawn_location] == BlockType::Sand {
                            break;
                        }

                        // Otherwise, start a new granule.
                        num_sand_granules_spawned += 1;
                        chunk.spawn_sand(spawn_location)?;
                    }
                    SandUpdate::Moving => {
                        // Continue
                    }
                    SandUpdate::Abyss => Err(invalid_input(
                        "Sand unexpectedly fell into the abyss",
                    ))?,
                }
            }

            println!("{}", num_sand_granules_spawned);
        }
    }

    Ok(())
}
