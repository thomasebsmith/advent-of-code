use std::collections::HashMap;
use std::io;
use std::ops::{Add, AddAssign};
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

use crate::errors::invalid_input;
use crate::parse::{lines, parse_all};
use crate::part::Part;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Vec2D {
    x: i64,
    y: i64,
}

impl FromStr for Vec2D {
    type Err = io::Error;

    fn from_str(s: &str) -> io::Result<Self> {
        let &[x, y] = &parse_all::<_, i64>(s.split(","))?[..] else {
            return Err(invalid_input("Expected x,y"));
        };
        Ok(Self { x, y })
    }
}

impl Add for Vec2D {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl AddAssign for Vec2D {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

struct Robot {
    position: Vec2D,
    velocity: Vec2D,
}

impl Robot {
    fn from_line(line: &str) -> io::Result<Self> {
        let &[pos_str, vel_str] =
            &line.split_whitespace().collect::<Vec<_>>()[..]
        else {
            return Err(invalid_input(
                "Expected position <whitespace> velocity",
            ));
        };
        let Some(pos_vec_str) = pos_str.strip_prefix("p=") else {
            return Err(invalid_input("Expected p=position"));
        };
        let Some(vel_vec_str) = vel_str.strip_prefix("v=") else {
            return Err(invalid_input("Expected v=velocity"));
        };
        Ok(Self {
            position: pos_vec_str.parse()?,
            velocity: vel_vec_str.parse()?,
        })
    }

    fn move_robot(&mut self, width: i64, height: i64, moves: usize) {
        for _ in 0..moves {
            self.position += self.velocity;

            // Better hope the compiler optimizes this
            while self.position.x < 0 {
                self.position.x += width;
            }
            while self.position.x >= width {
                self.position.x -= width;
            }
            while self.position.y < 0 {
                self.position.y += height;
            }
            while self.position.y >= height {
                self.position.y -= height;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Quadrant {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

struct Bathroom {
    width: i64,
    height: i64,
    robots: Vec<Robot>,
    positions: HashMap<Vec2D, usize>,
}

impl Bathroom {
    fn new<R: io::Read>(reader: io::BufReader<R>) -> io::Result<Self> {
        let robots = lines(reader)?
            .map(|line| Robot::from_line(&line))
            .collect::<io::Result<Vec<_>>>()?;
        let mut positions = HashMap::<Vec2D, usize>::new();
        for robot in &robots {
            *positions.entry(robot.position).or_insert(0) += 1;
        }
        Ok(Self {
            width: 101,
            height: 103,
            robots,
            positions,
        })
    }

    fn quadrant(&self, position: Vec2D) -> Option<Quadrant> {
        if position.x < 0
            || position.x >= self.width
            || position.y < 0
            || position.y >= self.height
        {
            return None;
        }
        if self.width % 2 == 1 && position.x == self.width / 2 {
            return None;
        }
        if self.height % 2 == 1 && position.y == self.height / 2 {
            return None;
        }
        Some(
            if position.x < self.width / 2 && position.y < self.height / 2 {
                Quadrant::TopLeft
            } else if position.x >= self.width / 2
                && position.y < self.height / 2
            {
                Quadrant::TopRight
            } else if position.x < self.width / 2
                && position.y >= self.height / 2
            {
                Quadrant::BottomLeft
            } else {
                Quadrant::BottomRight
            },
        )
    }

    fn move_robots(&mut self, moves: usize) {
        for robot in &mut self.robots {
            *self.positions.get_mut(&robot.position).unwrap() -= 1;
            robot.move_robot(self.width, self.height, moves);
            *self.positions.entry(robot.position).or_insert(0) += 1;
        }
    }

    fn safety_factor(&self) -> usize {
        let mut map = HashMap::<Quadrant, usize>::new();
        for robot in &self.robots {
            if let Some(quadrant) = self.quadrant(robot.position) {
                *map.entry(quadrant).or_insert(0) += 1;
            }
        }
        map.values().product()
    }

    fn print(&self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let pos = Vec2D { x, y };
                if *self.positions.get(&pos).unwrap_or(&0) != 0 {
                    print!("*");
                } else {
                    print!(" ");
                }
            }
            println!();
        }
    }

    #[allow(dead_code)]
    fn move_and_print(&mut self, moves: usize) {
        self.print();
        for move_num in 0..moves {
            sleep(Duration::from_millis(250));
            println!("After {} seconds:", move_num + 1);
            self.move_robots(1);
            self.print();
            println!();
        }
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut bathroom = Bathroom::new(reader)?;

    match part {
        Part::Part1 => bathroom.move_robots(100),
        Part::Part2 => {
            bathroom.move_robots(7774);
            bathroom.print();
            // bathroom.move_and_print(10000),
        }
    }

    let result = bathroom.safety_factor();

    println!("{result}");

    Ok(())
}
