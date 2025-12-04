use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::part::Part;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Position {
    row: isize,
    col: isize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Direction {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
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

    fn eight_neighbors(self) -> impl Iterator<Item = Self> {
        [
            self.move_one(Direction::Up).move_one(Direction::Left),
            self.move_one(Direction::Up),
            self.move_one(Direction::Up).move_one(Direction::Right),
            self.move_one(Direction::Left),
            self.move_one(Direction::Right),
            self.move_one(Direction::Down).move_one(Direction::Left),
            self.move_one(Direction::Down),
            self.move_one(Direction::Down).move_one(Direction::Right),
        ]
        .into_iter()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Cell {
    Empty,
    RollOfPaper,
}

struct Map {
    layout: Vec<Vec<Cell>>,
    width: isize,
    height: isize,
}

impl Map {
    fn new<R: io::Read>(reader: io::BufReader<R>) -> io::Result<Self> {
        let mut width: Option<isize> = None;
        let mut layout = Vec::<Vec<Cell>>::new();
        for line in reader.lines() {
            let line_layout = line?
                .chars()
                .map(|ch| match ch {
                    '.' => Ok(Cell::Empty),
                    '@' => Ok(Cell::RollOfPaper),
                    _ => Err(invalid_input("Invalid character")),
                })
                .collect::<io::Result<Vec<_>>>()?;
            if let Some(known_width) = width {
                if known_width != line_layout.len() as isize {
                    return Err(invalid_input("Varying widths"));
                }
            } else {
                width = Some(line_layout.len() as isize);
            }
            layout.push(line_layout);
        }
        let height = layout.len() as isize;
        let Some(width) = width else {
            return Err(invalid_input("No lines"));
        };

        Ok(Self {
            layout,
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

    fn at(&self, position: Position) -> Option<Cell> {
        if !self.in_bounds(position) {
            None
        } else {
            Some(self.layout[position.row as usize][position.col as usize])
        }
    }

    fn at_mut(&mut self, position: Position) -> Option<&mut Cell> {
        if !self.in_bounds(position) {
            None
        } else {
            Some(&mut self.layout[position.row as usize][position.col as usize])
        }
    }

    #[allow(dead_code)]
    fn print(&self) {
        for row_layout in self.layout.iter() {
            for &cell in row_layout.iter() {
                print!(
                    "{}",
                    match cell {
                        Cell::Empty => '.',
                        Cell::RollOfPaper => '@',
                    }
                );
            }
            println!();
        }
    }

    fn is_roll(&self, position: Position) -> bool {
        self.at(position)
            .map(|cell| cell == Cell::RollOfPaper)
            .unwrap_or(false)
    }

    fn is_accessible(&self, position: Position) -> bool {
        let neighbors = position.eight_neighbors();
        let num_adjacent_rolls =
            neighbors.filter(|neighbor| self.is_roll(*neighbor)).count();
        num_adjacent_rolls < 4
    }

    fn accessible_rolls(&self) -> Vec<Position> {
        (0..self.height)
            .flat_map(move |row| {
                (0..self.width)
                    .map(move |col| Position { row, col })
                    .filter(|position| {
                        self.is_roll(*position) && self.is_accessible(*position)
                    })
            })
            .collect()
    }

    fn num_accessible_rolls(&self) -> usize {
        self.accessible_rolls().len()
    }

    fn remove_currently_accessible_rolls(&mut self) -> usize {
        let to_remove = self.accessible_rolls();
        let num_removed = to_remove.len();
        for position in to_remove {
            *self.at_mut(position).unwrap() = Cell::Empty;
        }
        num_removed
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut map = Map::new(reader)?;

    let result = match part {
        Part::Part1 => map.num_accessible_rolls(),
        Part::Part2 => {
            let mut count = 0usize;
            loop {
                let num_removed = map.remove_currently_accessible_rolls();
                count += num_removed;
                if num_removed == 0 {
                    break;
                }
            }
            count
        }
    };

    println!("{result}");

    Ok(())
}
