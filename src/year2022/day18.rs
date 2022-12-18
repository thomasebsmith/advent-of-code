use std::collections::{HashSet, VecDeque};
use std::io;
use std::io::BufRead;
use std::str::FromStr;

use crate::errors::invalid_input;
use crate::part::Part;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Vec3D {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

impl FromStr for Vec3D {
    type Err = io::Error;

    fn from_str(string: &str) -> io::Result<Self> {
        let &[x, y, z] = &string
            .split(',')
            .map(|string| string.parse::<i64>()
            .map_err(invalid_input))
            .collect::<io::Result<Vec<_>>>()?[..] else {
            Err(invalid_input("Expected x,y,z"))?
        };
        Ok(Self { x, y, z })
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum Plane {
    XY,
    YZ,
    XZ,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct Side {
    pub location: Vec3D,
    pub plane: Plane,
}

#[derive(Clone, Debug)]
struct Cube {
    pub location: Vec3D,
}

impl Cube {
    fn side(&self, number: usize) -> Side {
        match number {
            0 => Side {
                location: self.location,
                plane: Plane::XY,
            },
            1 => Side {
                location: self.location,
                plane: Plane::YZ,
            },
            2 => Side {
                location: self.location,
                plane: Plane::XZ,
            },
            3 => {
                let mut location3 = self.location;
                location3.z += 1;
                Side {
                    location: location3,
                    plane: Plane::XY,
                }
            }
            4 => {
                let mut location4 = self.location;
                location4.x += 1;
                Side {
                    location: location4,
                    plane: Plane::YZ,
                }
            }
            5 => {
                let mut location5 = self.location;
                location5.y += 1;
                Side {
                    location: location5,
                    plane: Plane::XZ,
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn sides(&self) -> [Side; 6] {
        [
            self.side(0),
            self.side(1),
            self.side(2),
            self.side(3),
            self.side(4),
            self.side(5),
        ]
    }

    pub fn neighbors(&self) -> [(Side, Vec3D); 6] {
        let mut location1 = self.location;
        location1.z -= 1;

        let mut location2 = self.location;
        location2.x -= 1;

        let mut location3 = self.location;
        location3.y -= 1;

        let mut location4 = self.location;
        location4.z += 1;

        let mut location5 = self.location;
        location5.x += 1;

        let mut location6 = self.location;
        location6.y += 1;

        [
            (self.side(0), location1),
            (self.side(1), location2),
            (self.side(2), location3),
            (self.side(3), location4),
            (self.side(4), location5),
            (self.side(5), location6),
        ]
    }
}

fn part_1<R: io::Read>(reader: io::BufReader<R>) -> io::Result<()> {
    let mut unconnected_sides = HashSet::<Side>::new();
    let mut connected_sides = HashSet::<Side>::new();

    for line in reader.lines() {
        let point = line?.parse::<Vec3D>()?;
        let cube = Cube { location: point };
        for side in cube.sides() {
            if connected_sides.contains(&side) {
                continue;
            }

            if !unconnected_sides.insert(side.clone()) {
                // We just connected this side
                unconnected_sides.remove(&side);
                connected_sides.insert(side);
            }
        }
    }

    println!("{}", unconnected_sides.len());

    Ok(())
}

fn part_2<R: io::Read>(reader: io::BufReader<R>) -> io::Result<()> {
    let cube_locations = reader
        .lines()
        .map(|line: io::Result<String>| -> io::Result<Vec3D> {
            line?.parse::<Vec3D>()
        })
        .collect::<io::Result<HashSet<Vec3D>>>()?;

    // All cubes must fit in a larger cube of side length REACHABLE_SEARCH_DIM
    // with origin reachable_starting_location.
    // There must be a layer of air just on the inside of that cube as well.
    const REACHABLE_SEARCH_DIM: i64 = 40;
    let reachable_starting_location = Vec3D { x: 0, y: 0, z: 0 };

    let in_range = |location: Vec3D| -> bool {
        [location.x, location.y, location.z]
            .into_iter()
            .all(|coord| coord >= 0 && coord < REACHABLE_SEARCH_DIM)
    };

    let mut reachable_sides = HashSet::<Side>::new();

    let mut visited_locations = HashSet::<Vec3D>::new();

    let mut locations = VecDeque::<Vec3D>::new();
    locations.push_back(reachable_starting_location);

    loop {
        let Some(location) = locations.pop_front() else {
            break;
        };

        for (side, neighbor_location) in (Cube { location }).neighbors() {
            if in_range(neighbor_location) {
                if cube_locations.contains(&neighbor_location) {
                    reachable_sides.insert(side);
                } else if !visited_locations.contains(&neighbor_location) {
                    visited_locations.insert(neighbor_location);
                    locations.push_back(neighbor_location);
                }
            }
        }
    }

    println!("{}", reachable_sides.len());

    Ok(())
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let func = match part {
        Part::Part1 => part_1,
        Part::Part2 => part_2,
    };

    func(reader)
}
