use std::collections::HashSet;
use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::iter::n_elements;
use crate::part::Part;

#[derive(Copy, Clone)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    pub fn new(letter: &str) -> io::Result<Self> {
        match letter {
            "L" => Ok(Self::Left),
            "R" => Ok(Self::Right),
            "U" => Ok(Self::Up),
            "D" => Ok(Self::Down),
            _ => Err(invalid_input("Invalid direction")),
        }
    }
}

#[derive(Clone, Copy)]
struct Movement {
    pub direction: Direction,
    pub count: isize,
}

impl Movement {
    pub fn new(line: &str) -> io::Result<Self> {
        let words = n_elements(2, line.split(' '))
            .ok_or_else(|| invalid_input("More than 2 words on line"))?;
        let direction = Direction::new(words[0])?;
        let count = words[1].parse::<isize>().map_err(invalid_input)?;
        Ok(Self { direction, count })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Position {
    pub x: isize,
    pub y: isize,
}

impl Position {
    pub fn new() -> Self {
        Self { x: 0, y: 0 }
    }
}

struct Rope {
    positions: Vec<Position>,
}

impl Rope {
    pub fn new(num_knots: usize) -> io::Result<Self> {
        if num_knots < 1 {
            Err(invalid_input("Must be at least 1 knot in rope"))?
        }
        Ok(Self {
            positions: vec![Position::new(); num_knots],
        })
    }

    pub fn tail_position(&self) -> Position {
        *self.positions.last().unwrap()
    }

    pub fn move_head(&mut self, direction: Direction) {
        match direction {
            Direction::Left => self.positions[0].x -= 1,
            Direction::Right => self.positions[0].x += 1,
            Direction::Up => self.positions[0].y += 1,
            Direction::Down => self.positions[0].y -= 1,
        }

        self.catch_up(0);
    }

    fn catch_up(&mut self, knots_behind_knot_index: usize) {
        // At the end of the rope
        if knots_behind_knot_index + 1 >= self.positions.len() {
            return;
        }

        let head_position = self.positions[knots_behind_knot_index];
        let tail_position = &mut self.positions[knots_behind_knot_index + 1];

        let x_diff = head_position.x - tail_position.x;
        let y_diff = head_position.y - tail_position.y;

        let move_x =
            x_diff.abs() > 1 || (x_diff.abs() == 1 && y_diff.abs() > 1);
        let move_y =
            y_diff.abs() > 1 || (y_diff.abs() == 1 && x_diff.abs() > 1);

        if move_x {
            if x_diff > 0 {
                tail_position.x += 1;
            } else {
                tail_position.x -= 1;
            }
        }

        if move_y {
            if y_diff > 0 {
                tail_position.y += 1
            } else {
                tail_position.y -= 1
            }
        }

        self.catch_up(knots_behind_knot_index + 1);
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let num_knots = match part {
        Part::Part1 => 2,
        Part::Part2 => 10,
    };

    let mut rope = Rope::new(num_knots)?;

    let mut tail_positions = HashSet::<Position>::new();
    tail_positions.insert(rope.tail_position());

    for line in reader.lines() {
        let line = line?;
        let movement = Movement::new(&line)?;

        for _ in 0..movement.count {
            rope.move_head(movement.direction);
            tail_positions.insert(rope.tail_position());
        }
    }

    println!("{}", tail_positions.len());

    Ok(())
}
