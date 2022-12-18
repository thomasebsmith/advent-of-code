use std::cmp::max;
use std::collections::VecDeque;
use std::io;
use std::io::BufRead;
use std::ops::Add;

use crate::errors::invalid_input;
use crate::part::Part;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Point {
    pub x: usize,
    pub y: usize,
}

impl Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RockType {
    Flat,
    Plus,
    Corner,
    Vertical,
    Square,
}

struct Rock {
    pub relative_points: Vec<Point>,
    pub height: usize,
}

impl Rock {
    pub fn new(rock_type: RockType) -> Self {
        let relative_points = match rock_type {
            RockType::Flat => vec![
                Point { x: 0, y: 0 },
                Point { x: 1, y: 0 },
                Point { x: 2, y: 0 },
                Point { x: 3, y: 0 },
            ],
            RockType::Plus => vec![
                Point { x: 1, y: 0 },
                Point { x: 0, y: 1 },
                Point { x: 1, y: 1 },
                Point { x: 2, y: 1 },
                Point { x: 1, y: 2 },
            ],
            RockType::Corner => vec![
                Point { x: 0, y: 0 },
                Point { x: 1, y: 0 },
                Point { x: 2, y: 0 },
                Point { x: 2, y: 1 },
                Point { x: 2, y: 2 },
            ],
            RockType::Vertical => vec![
                Point { x: 0, y: 0 },
                Point { x: 0, y: 1 },
                Point { x: 0, y: 2 },
                Point { x: 0, y: 3 },
            ],
            RockType::Square => vec![
                Point { x: 0, y: 0 },
                Point { x: 1, y: 0 },
                Point { x: 0, y: 1 },
                Point { x: 1, y: 1 },
            ],
        };

        let height = match rock_type {
            RockType::Flat => 1,
            RockType::Plus => 3,
            RockType::Corner => 3,
            RockType::Vertical => 4,
            RockType::Square => 2,
        };

        Self {
            relative_points,
            height,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Block {
    Air,
    Rock,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Wind {
    Left,
    Right,
}

struct Chamber<'a, I> {
    width: usize,
    winds: I,
    fixed_layout: VecDeque<Vec<Block>>,
    fixed_layout_y_offset: usize,
    rocks: &'a [&'a Rock],
    rock_num: usize,
    pub wind_movements: usize,
    pub tower_height: usize,
}

impl<'a, I: Iterator<Item = Wind>> Chamber<'a, I> {
    pub fn new(width: usize, winds: I, rocks: &'a [&'a Rock]) -> Self {
        Self {
            width,
            winds,
            fixed_layout: VecDeque::new(),
            fixed_layout_y_offset: 0,
            rocks,
            rock_num: 0,
            wind_movements: 0,
            tower_height: 0,
        }
    }

    fn next_rock(&mut self) -> &'a Rock {
        let rock = self.rocks[self.rock_num % self.rocks.len()];
        self.rock_num += 1;
        rock
    }

    fn mut_ref_to_block(&mut self, location: Point) -> &mut Block {
        assert!(location.y >= self.fixed_layout_y_offset);
        let relative_y = location.y - self.fixed_layout_y_offset;
        while relative_y >= self.fixed_layout.len() {
            self.fixed_layout.push_back(vec![Block::Air; self.width]);
        }

        &mut self.fixed_layout[relative_y][location.x]
    }

    fn get_block(&mut self, location: Point) -> Block {
        *self.mut_ref_to_block(location)
    }

    fn fill_with_rock(&mut self, location: Point) {
        *self.mut_ref_to_block(location) = Block::Rock;
    }

    fn new_rock_can_be_at(
        &mut self,
        rock: &Rock,
        rock_location: Point,
    ) -> bool {
        rock.relative_points
            .iter()
            .map(|point| *point + rock_location)
            .all(|point| {
                point.x < self.width && self.get_block(point) == Block::Air
            })
    }

    fn garbage_collect(&mut self) {
        for (i, row) in self.fixed_layout.iter().enumerate() {
            if row.iter().all(|block| *block == Block::Rock) {
                for _ in 0..i {
                    self.fixed_layout.pop_front();
                }
                self.fixed_layout_y_offset += i;
                break;
            }
        }
    }

    pub fn simulate_rock_fall(&mut self, x: usize) {
        let rock = self.next_rock();
        let mut rock_location = Point {
            x,
            y: self.tower_height + 3,
        };

        loop {
            // Try to move with the wind
            if let Some(wind) = self.winds.next() {
                match wind {
                    Wind::Left => {
                        let mut new_location = rock_location;
                        if new_location.x != 0 {
                            new_location.x -= 1;
                            if self.new_rock_can_be_at(&rock, new_location) {
                                rock_location = new_location;
                            }
                        }
                    }
                    Wind::Right => {
                        let mut new_location = rock_location;
                        new_location.x += 1;
                        if self.new_rock_can_be_at(&rock, new_location) {
                            rock_location = new_location;
                        }
                    }
                }
                self.wind_movements += 1;
            }

            // Try to move downwards
            if rock_location.y == 0 {
                break;
            }

            let mut new_location = rock_location;
            new_location.y -= 1;
            if self.new_rock_can_be_at(&rock, new_location) {
                rock_location = new_location;
            } else {
                break;
            }
        }

        // Solidify the rock
        for relative_point in rock.relative_points.iter() {
            let point = *relative_point + rock_location;
            self.fill_with_rock(point);
        }

        self.tower_height =
            max(self.tower_height, rock_location.y + rock.height);
        if self.rock_num % 10_000 == 0 {
            self.garbage_collect();
        }
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let [Ok(line)] = &reader.lines().collect::<Vec<_>>()[..] else {
        Err(invalid_input("Expected 1 line"))?
    };

    let winds = line
        .chars()
        .map(|ch| match ch {
            '<' => Ok(Wind::Left),
            '>' => Ok(Wind::Right),
            _ => Err(invalid_input("Bad wind character")),
        })
        .collect::<io::Result<Vec<_>>>()?;

    let rocks: [&Rock; 5] = [
        &Rock::new(RockType::Flat),
        &Rock::new(RockType::Plus),
        &Rock::new(RockType::Corner),
        &Rock::new(RockType::Vertical),
        &Rock::new(RockType::Square),
    ];

    let mut chamber = Chamber::new(7, winds.into_iter().cycle(), &rocks);

    let num_rocks: usize = match part {
        Part::Part1 => 2_022,
        Part::Part2 => 20_000,
    };

    let mut first_heights = Vec::<usize>::new();
    let mut recent_heights = VecDeque::<usize>::new();
    for i in 0..num_rocks {
        let before_height = chamber.tower_height;

        chamber.simulate_rock_fall(2);

        let after_height = chamber.tower_height;

        let diff = after_height - before_height;

        const REPEAT_CHECK_SIZE: usize = 100;
        if first_heights.len() < REPEAT_CHECK_SIZE {
            const REPEAT_SAMPLE_START: usize = 5_000;
            if i >= REPEAT_SAMPLE_START {
                first_heights.push(diff);
            }
        } else {
            recent_heights.push_back(diff);

            if recent_heights.len() > first_heights.len() {
                recent_heights.pop_front();

                if recent_heights.iter().eq(first_heights.iter()) {
                    println!(
                        "Potential repeat of size {} before num_rocks={} height={}",
                        REPEAT_CHECK_SIZE,
                        i + 1,
                        chamber.tower_height,
                    );
                }
            }
        }
    }

    println!(
        "Total height for num_rocks={}: {}",
        num_rocks, chamber.tower_height
    );

    Ok(())
}
