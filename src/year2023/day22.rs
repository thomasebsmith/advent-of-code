use std::cmp::{max, min};
use std::collections::HashSet;
use std::io;

use crate::errors::invalid_input;
use crate::parse::lines;
use crate::part::Part;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
struct Point {
    x: usize,
    y: usize,
    z: usize,
}

impl Point {
    fn from_string(string: &str) -> io::Result<Self> {
        let [x_str, y_str, z_str] = string.split(",").collect::<Vec<_>>()[..]
        else {
            return Err(invalid_input("Expected <x>,<y>,<z>"));
        };

        let x = x_str.parse::<usize>().map_err(invalid_input)?;
        let y = y_str.parse::<usize>().map_err(invalid_input)?;
        let z = z_str.parse::<usize>().map_err(invalid_input)?;

        Ok(Self { x, y, z })
    }
}

const Z_ON_GROUND: usize = 1;

// For each brick, we keep track of a set of blocks that it occupies. This isn't
// very fast, but it's fast enough for the inputs we're dealing with.
#[derive(Clone, Debug)]
struct Brick {
    start: Point,
    end: Point,
    blocks: HashSet<Point>,
}

impl Brick {
    fn from_line(line: impl AsRef<str>) -> io::Result<Self> {
        let [start_str, end_str] =
            line.as_ref().split("~").collect::<Vec<_>>()[..]
        else {
            return Err(invalid_input("Expected <start>~<end>"));
        };
        let start = Point::from_string(start_str)?;
        let end = Point::from_string(end_str)?;

        let num_equal = (start.x == end.x) as u64
            + (start.y == end.y) as u64
            + (start.z == end.z) as u64;
        if num_equal < 2 {
            return Err(invalid_input("Bricks must be straight lines"));
        }
        Ok(Self {
            start,
            end,
            blocks: Self::_blocks(start, end),
        })
    }

    fn _blocks(start: Point, end: Point) -> HashSet<Point> {
        if start.x != end.x {
            (min(start.x, end.x)..=max(start.x, end.x))
                .map(|x| Point {
                    x,
                    y: start.y,
                    z: start.z,
                })
                .collect()
        } else if start.y != end.y {
            (min(start.y, end.y)..=max(start.y, end.y))
                .map(|y| Point {
                    x: start.x,
                    y,
                    z: start.z,
                })
                .collect()
        } else if start.z != end.z {
            (min(start.z, end.z)..=max(start.z, end.z))
                .map(|z| Point {
                    x: start.x,
                    y: start.y,
                    z,
                })
                .collect()
        } else {
            HashSet::from([start])
        }
    }

    fn fallen_blocks(&self) -> Option<impl Iterator<Item = Point> + '_> {
        if min(self.start.z, self.end.z) == Z_ON_GROUND {
            None
        } else {
            Some(self.blocks.iter().map(|block| Point {
                x: block.x,
                y: block.y,
                z: block.z - 1,
            }))
        }
    }

    fn fall(&mut self) {
        if let Some(fallen) = self
            .fallen_blocks()
            .map(|fallen| fallen.collect::<HashSet<_>>())
        {
            self.start.z -= 1;
            self.end.z -= 1;
            self.blocks = fallen;
        }
    }
}

#[derive(Clone, Debug)]
struct Snapshot {
    bricks: Vec<Brick>,
    occupied: HashSet<Point>,
}

impl Snapshot {
    fn new(bricks: Vec<Brick>) -> io::Result<Self> {
        let mut occupied = HashSet::<Point>::new();
        for brick in &bricks {
            for block in &brick.blocks {
                if !occupied.insert(*block) {
                    return Err(invalid_input("Overlapping bricks"));
                }
            }
        }
        Ok(Self { bricks, occupied })
    }

    fn from_reader<R: io::Read>(reader: io::BufReader<R>) -> io::Result<Self> {
        lines(reader)?
            .map(Brick::from_line)
            .collect::<io::Result<Vec<_>>>()
            .and_then(Self::new)
    }

    fn simulate_one(&mut self) -> Option<usize> {
        // We simulate bricks falling by repetitively moving the first brick
        // that we can find that will fall one block. This isn't efficient, but
        // it's good enough.
        for (i, brick) in self.bricks.iter_mut().enumerate() {
            let can_fall = if let Some(new_blocks) = brick.fallen_blocks() {
                let mut can_fall = true;
                for block in new_blocks {
                    if !brick.blocks.contains(&block)
                        && self.occupied.contains(&block)
                    {
                        can_fall = false;
                        break;
                    }
                }
                can_fall
            } else {
                false
            };

            if !can_fall {
                continue;
            }
            for block in &brick.blocks {
                self.occupied.remove(block);
            }
            brick.fall();
            for block in &brick.blocks {
                self.occupied.insert(*block);
            }
            return Some(i);
        }
        None
    }

    fn simulate_until_stable(&mut self) -> usize {
        let mut bricks_that_have_fallen = HashSet::<usize>::new();
        while let Some(brick_index) = self.simulate_one() {
            bricks_that_have_fallen.insert(brick_index);
        }
        bricks_that_have_fallen.len()
    }

    fn remove_brick(&mut self, brick_index: usize) {
        let brick = self.bricks.remove(brick_index);
        for block in brick.blocks {
            self.occupied.remove(&block);
        }
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut snapshot = Snapshot::from_reader(reader)?;

    snapshot.simulate_until_stable();

    let mut result = 0usize;
    for brick_index in 0..snapshot.bricks.len() {
        let mut clone = snapshot.clone();
        clone.remove_brick(brick_index);
        match part {
            Part::Part1 => {
                if clone.simulate_one().is_none() {
                    result += 1;
                }
            }
            Part::Part2 => {
                result += clone.simulate_until_stable();
            }
        }
    }

    println!("{result}");

    Ok(())
}
