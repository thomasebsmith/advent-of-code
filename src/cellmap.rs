use std::cmp::{max, min};
use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Position {
    pub row: isize,
    pub col: isize,
}

impl Position {
    pub fn straight_line_to(
        self,
        other: Position,
    ) -> Option<(Vec<Position>, Direction)> {
        if self == other {
            None
        } else if self.row == other.row {
            let min_val = min(self.col, other.col);
            let max_val = max(self.col, other.col);
            Some((
                (min_val..=max_val)
                    .map(|col| Position { row: self.row, col })
                    .collect(),
                if self.col < other.col {
                    Direction::Right
                } else {
                    Direction::Left
                },
            ))
        } else if self.col == other.col {
            let min_val = min(self.row, other.row);
            let max_val = max(self.row, other.row);
            Some((
                (min_val..=max_val)
                    .map(|row| Position { row, col: self.col })
                    .collect(),
                if self.row < other.row {
                    Direction::Down
                } else {
                    Direction::Up
                },
            ))
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Direction {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

impl Direction {
    #[allow(dead_code)]
    pub const ALL: [Self; 4] = [Self::Up, Self::Down, Self::Left, Self::Right];

    pub fn turn_direction(self, other: Self) -> Self {
        match (self, other) {
            (Self::Up, Self::Up)
            | (Self::Down, Self::Down)
            | (Self::Left, Self::Left)
            | (Self::Right, Self::Right) => Direction::Up,
            (Self::Up, Self::Down)
            | (Self::Down, Self::Up)
            | (Self::Left, Self::Right)
            | (Self::Right, Self::Left) => Direction::Down,
            (Self::Up, Self::Right)
            | (Self::Down, Self::Left)
            | (Self::Left, Self::Up)
            | (Self::Right, Self::Down) => Direction::Right,
            (Self::Up, Self::Left)
            | (Self::Down, Self::Right)
            | (Self::Left, Self::Down)
            | (Self::Right, Self::Up) => Direction::Left,
        }
    }

    pub fn turn_right(self) -> Self {
        match self {
            Self::Up => Self::Right,
            Self::Down => Self::Left,
            Self::Left => Self::Up,
            Self::Right => Self::Down,
        }
    }

    pub fn turn_left(self) -> Self {
        match self {
            Self::Up => Self::Left,
            Self::Down => Self::Right,
            Self::Left => Self::Down,
            Self::Right => Self::Up,
        }
    }
}

impl Position {
    pub fn move_one(self, direction: Direction) -> Self {
        let row = self.row;
        let col = self.col;
        match direction {
            Direction::Up => Self { row: row - 1, col },
            Direction::Down => Self { row: row + 1, col },
            Direction::Left => Self { row, col: col - 1 },
            Direction::Right => Self { row, col: col + 1 },
        }
    }

    pub fn eight_neighbors(self) -> [Self; 8] {
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
    }

    #[allow(dead_code)]
    pub fn four_neighbors(self) -> [Self; 4] {
        [
            self.move_one(Direction::Up),
            self.move_one(Direction::Down),
            self.move_one(Direction::Left),
            self.move_one(Direction::Right),
        ]
    }
}

pub trait Cell: Copy {
    fn to_char(self) -> char;

    fn from_char(ch: char) -> Option<Self>;
}

pub struct CellMap<C> {
    layout: Vec<Vec<C>>,
    width: isize,
    height: isize,
}

impl<C: Cell> CellMap<C> {
    pub fn new<R: io::Read>(reader: io::BufReader<R>) -> io::Result<Self> {
        let mut width: Option<isize> = None;
        let mut layout = Vec::<Vec<C>>::new();
        for line in reader.lines() {
            let line_layout = line?
                .chars()
                .map(|ch| {
                    C::from_char(ch).ok_or_else(|| {
                        invalid_input(format!(
                            "Unexpected cell character '{ch}'"
                        ))
                    })
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
}

impl<C> CellMap<C> {
    #[allow(dead_code)]
    pub fn width(&self) -> isize {
        self.width
    }

    #[allow(dead_code)]
    pub fn height(&self) -> isize {
        self.height
    }

    pub fn all_positions(&self) -> impl Iterator<Item = Position> {
        (0..self.height).flat_map(move |row| {
            (0..self.width).map(move |col| Position { row, col })
        })
    }

    pub fn in_bounds(&self, position: Position) -> bool {
        position.row >= 0
            && position.row < self.height
            && position.col >= 0
            && position.col < self.width
    }

    pub fn at_mut(&mut self, position: Position) -> Option<&mut C> {
        if !self.in_bounds(position) {
            None
        } else {
            Some(&mut self.layout[position.row as usize][position.col as usize])
        }
    }
}

impl<C: Copy> CellMap<C> {
    pub fn filled_with(cell: C, width: usize, height: usize) -> Self {
        Self {
            layout: vec![vec![cell; width]; height],
            width: width as isize,
            height: height as isize,
        }
    }

    pub fn at(&self, position: Position) -> Option<C> {
        if !self.in_bounds(position) {
            None
        } else {
            Some(self.layout[position.row as usize][position.col as usize])
        }
    }
}

impl<C: Cell> CellMap<C> {
    #[allow(dead_code)]
    pub fn print(&self) {
        for row_layout in self.layout.iter() {
            for &cell in row_layout.iter() {
                print!("{}", cell.to_char());
            }
            println!();
        }
    }
}
