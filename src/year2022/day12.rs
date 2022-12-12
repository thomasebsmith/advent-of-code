use std::collections::VecDeque;
use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::part::Part;

#[derive(Clone, Copy, PartialEq, Eq)]
struct Height(i64);

impl Height {
    pub fn lowest() -> Self {
        Self(0)
    }

    pub fn highest() -> Self {
        Self(25)
    }

    pub fn new(ch: char) -> io::Result<Self> {
        if ch.is_ascii_lowercase() {
            Ok(Self(ch as i64 - 'a' as i64))
        } else if ch == 'S' {
            Ok(Self::lowest())
        } else if ch == 'E' {
            Ok(Self::highest())
        } else {
            Err(invalid_input("Unknown map character"))
        }
    }
}

impl std::ops::Sub for Height {
    type Output = i64;

    fn sub(self, rhs: Self) -> Self::Output {
        self.0 - rhs.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Location {
    pub x: usize,
    pub y: usize,
}

struct Map {
    pub position: Location,
    destination: Location,
    width: usize,
    height: usize,
    grid: Vec<Vec<Height>>,
}

impl Map {
    pub fn new<I: Iterator<Item = io::Result<String>>>(
        lines: I,
    ) -> io::Result<Self> {
        let mut width: Option<usize> = None;
        let mut grid = Vec::<Vec<Height>>::new();
        let mut position: Option<Location> = None;
        let mut destination: Option<Location> = None;

        for (y, line) in lines.enumerate() {
            let line = line?;

            grid.push(Vec::new());

            let row = grid.last_mut().unwrap();

            for ch in line.chars() {
                if ch == 'S' {
                    if position.is_none() {
                        position = Some(Location { x: row.len(), y });
                    } else {
                        Err(invalid_input("Repeated S"))?
                    }
                } else if ch == 'E' {
                    if destination.is_none() {
                        destination = Some(Location { x: row.len(), y });
                    } else {
                        Err(invalid_input("Repeated E"))?
                    }
                }

                row.push(Height::new(ch)?);
            }

            match width {
                None => {
                    width = Some(row.len());
                }
                Some(expected_width) => {
                    if row.len() != expected_width {
                        Err(invalid_input("Differing row widths"))?
                    }
                }
            }
        }

        let position = position.ok_or_else(|| invalid_input("No position"))?;
        let destination =
            destination.ok_or_else(|| invalid_input("No destination"))?;
        let width = width.ok_or_else(|| invalid_input("Empty map"))?;

        Ok(Self {
            position,
            destination,
            width,
            height: grid.len(),
            grid,
        })
    }

    fn can_travel(&self, from: Location, to: Location) -> bool {
        // Assumes that from an to are adjacent - just checks heights.
        let from_height = self.grid[from.y][from.x];
        let to_height = self.grid[to.y][to.x];
        to_height - from_height <= 1
    }

    pub fn all_low_locations(&self) -> Vec<Location> {
        self.grid
            .iter()
            .enumerate()
            .flat_map(|(y, row)| {
                row.iter().enumerate().filter_map(move |(x, height)| {
                    if *height == Height::lowest() {
                        Some(Location { x, y })
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    pub fn pathfind(&self, from: Location) -> Option<usize> {
        struct NextVisit {
            location: Location,
            distance: usize,
        }

        let mut queued_visit = vec![vec![false; self.width]; self.height];

        let mut queue = VecDeque::<NextVisit>::new();
        queue.push_back(NextVisit {
            location: from,
            distance: 0,
        });

        while let Some(visit) = queue.pop_front() {
            if visit.location == self.destination {
                return Some(visit.distance);
            }

            let mut try_queue_visit = |location: Location| {
                if !queued_visit[location.y][location.x]
                    && self.can_travel(visit.location, location)
                {
                    queued_visit[location.y][location.x] = true;
                    queue.push_back(NextVisit {
                        location,
                        distance: visit.distance + 1,
                    });
                }
            };

            if visit.location.y != 0 {
                try_queue_visit(Location {
                    x: visit.location.x,
                    y: visit.location.y - 1,
                });
            }

            if visit.location.y != self.height - 1 {
                try_queue_visit(Location {
                    x: visit.location.x,
                    y: visit.location.y + 1,
                });
            }

            if visit.location.x != 0 {
                try_queue_visit(Location {
                    x: visit.location.x - 1,
                    y: visit.location.y,
                });
            }

            if visit.location.x != self.width - 1 {
                try_queue_visit(Location {
                    x: visit.location.x + 1,
                    y: visit.location.y,
                });
            }
        }

        None
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let map = Map::new(reader.lines())?;

    let locations = match part {
        Part::Part1 => vec![map.position],
        Part::Part2 => map.all_low_locations(),
    };

    // The part 2 solution could be faster with memoization, but this runs
    // quickly for the given input.
    let min_distance = locations
        .into_iter()
        .filter_map(|location| map.pathfind(location))
        .min();
    println!(
        "{}",
        min_distance.ok_or_else(|| invalid_input("No path found"))?
    );

    Ok(())
}
