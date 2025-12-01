use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::part::Part;

#[derive(Clone, Copy, Debug)]
struct Position {
    row: isize,
    col: isize,
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

impl Direction {
    fn all_directions() -> [Self; 8] {
        [
            Self::Up,
            Self::Down,
            Self::Left,
            Self::Right,
            Self::UpLeft,
            Self::UpRight,
            Self::DownLeft,
            Self::DownRight,
        ]
    }
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
            Direction::UpLeft => Self {
                row: row - 1,
                col: col - 1,
            },
            Direction::UpRight => Self {
                row: row - 1,
                col: col + 1,
            },
            Direction::DownLeft => Self {
                row: row + 1,
                col: col - 1,
            },
            Direction::DownRight => Self {
                row: row + 1,
                col: col + 1,
            },
        }
    }
}

struct WordGrid {
    letters: Vec<Vec<char>>,
    width: isize,
    height: isize,
}

impl WordGrid {
    fn new<R: io::Read>(reader: io::BufReader<R>) -> io::Result<Self> {
        let mut width: Option<isize> = None;
        let mut letters: Vec<Vec<char>> = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let line_letters: Vec<char> = line.chars().collect();

            if let Some(current_width) = width {
                if current_width != line_letters.len() as isize {
                    return Err(invalid_input("Mismatched widths"));
                }
            } else {
                width = Some(line_letters.len() as isize);
            }

            letters.push(line_letters);
        }
        let Some(width) = width else {
            return Err(invalid_input("No lines"));
        };
        let height = letters.len() as isize;
        Ok(Self {
            letters,
            width,
            height,
        })
    }

    fn at(&self, position: Position) -> Option<char> {
        if position.row < 0
            || position.row >= self.height
            || position.col < 0
            || position.col >= self.width
        {
            None
        } else {
            Some(self.letters[position.row as usize][position.col as usize])
        }
    }

    fn has_match_at(
        &self,
        word: &str,
        position: Position,
        direction: Direction,
    ) -> bool {
        let mut current_position = position;
        for ch in word.chars() {
            if self.at(current_position) != Some(ch) {
                return false;
            }
            current_position = current_position.move_one(direction);
        }
        return true;
    }

    fn count_matches(&self, word: &str) -> isize {
        let mut matches: isize = 0;
        for row in 0..self.height {
            for col in 0..self.width {
                let position = Position { row, col };
                for direction in Direction::all_directions() {
                    if self.has_match_at(word, position, direction) {
                        matches += 1;
                    }
                }
            }
        }
        matches
    }

    fn count_mas_x(&self) -> isize {
        let mut matches: isize = 0;
        for row in 0..self.height {
            for col in 0..self.width {
                let position = Position { row, col };
                let fwd_diag = [Direction::UpRight, Direction::DownLeft];
                let back_diag = [Direction::UpLeft, Direction::DownRight];

                let mut fwd_match = false;
                let mut back_match = false;
                for i in 0..2 {
                    fwd_match = fwd_match
                        || self.has_match_at(
                            "MAS",
                            position.move_one(fwd_diag[i]),
                            fwd_diag[1 - i],
                        );
                    back_match = back_match
                        || self.has_match_at(
                            "MAS",
                            position.move_one(back_diag[i]),
                            back_diag[1 - i],
                        );
                }
                if fwd_match && back_match {
                    matches += 1;
                }
            }
        }
        matches
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let grid = WordGrid::new(reader)?;
    let result = match part {
        Part::Part1 => grid.count_matches("XMAS"),
        Part::Part2 => grid.count_mas_x(),
    };
    println!("{result}");

    Ok(())
}
