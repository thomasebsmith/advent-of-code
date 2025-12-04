use std::io;

use crate::cellmap::{Cell, CellMap, Position};
use crate::part::Part;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum PaperCell {
    Empty,
    RollOfPaper,
}

impl Cell for PaperCell {
    fn from_char(ch: char) -> Option<Self> {
        match ch {
            '.' => Some(Self::Empty),
            '@' => Some(Self::RollOfPaper),
            _ => None,
        }
    }

    fn to_char(self) -> char {
        match self {
            Self::Empty => '.',
            Self::RollOfPaper => '@',
        }
    }
}

impl CellMap<PaperCell> {
    fn is_roll(&self, position: Position) -> bool {
        self.at(position)
            .map(|cell| cell == PaperCell::RollOfPaper)
            .unwrap_or(false)
    }

    fn is_accessible(&self, position: Position) -> bool {
        let neighbors = position.eight_neighbors().into_iter();
        let num_adjacent_rolls =
            neighbors.filter(|neighbor| self.is_roll(*neighbor)).count();
        num_adjacent_rolls < 4
    }

    fn accessible_rolls(&self) -> Vec<Position> {
        self.all_positions()
            .filter(|position| {
                self.is_roll(*position) && self.is_accessible(*position)
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
            *self.at_mut(position).unwrap() = PaperCell::Empty;
        }
        num_removed
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut map = CellMap::<PaperCell>::new(reader)?;

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
