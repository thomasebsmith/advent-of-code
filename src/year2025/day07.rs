use std::collections::HashMap;
use std::io;
use std::mem::swap;

use crate::cellmap::{Cell, CellMap, Direction, Position};
use crate::part::Part;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Tile {
    Empty,
    Splitter,
    Beam,
}

impl Cell for Tile {
    fn from_char(ch: char) -> Option<Self> {
        match ch {
            '.' => Some(Self::Empty),
            '^' => Some(Self::Splitter),
            'S' => Some(Self::Beam),
            '|' => Some(Self::Beam),
            _ => None,
        }
    }

    fn to_char(self) -> char {
        match self {
            Self::Empty => '.',
            Self::Splitter => '^',
            Self::Beam => '|',
        }
    }
}

struct Manifold {
    map: CellMap<Tile>,
    unevaluated_beam_destinations: HashMap<Position, usize>,
    num_splits: usize,
    num_final_paths: usize,
}

impl Manifold {
    fn new(map: CellMap<Tile>) -> Self {
        let mut unevaluated_beam_destinations =
            HashMap::<Position, usize>::new();
        for position in map.all_positions() {
            if matches!(map.at(position), Some(Tile::Beam)) {
                unevaluated_beam_destinations
                    .insert(position.move_one(Direction::Down), 1);
            }
        }

        Self {
            map,
            unevaluated_beam_destinations,
            num_splits: 0,
            num_final_paths: 0,
        }
    }

    fn handle_beam_to(&mut self, beam_position: Position, num_ways: usize) {
        let Some(contents) = self.map.at(beam_position) else {
            // Out of bounds
            self.num_final_paths += num_ways;
            return;
        };

        match contents {
            Tile::Empty => {
                *self.map.at_mut(beam_position).unwrap() = Tile::Beam;
                self.unevaluated_beam_destinations
                    .insert(beam_position.move_one(Direction::Down), num_ways);
            }
            Tile::Beam => {
                *self
                    .unevaluated_beam_destinations
                    .get_mut(&beam_position.move_one(Direction::Down))
                    .unwrap() += num_ways;
            }
            Tile::Splitter => {
                self.num_splits += 1;
                self.handle_beam_to(
                    beam_position.move_one(Direction::Left),
                    num_ways,
                );
                self.handle_beam_to(
                    beam_position.move_one(Direction::Right),
                    num_ways,
                );
            }
        }
    }

    fn move_beams(&mut self) -> bool {
        if self.unevaluated_beam_destinations.is_empty() {
            return false;
        }

        let mut to_evaluate = HashMap::<Position, usize>::new();
        swap(&mut to_evaluate, &mut self.unevaluated_beam_destinations);
        for (beam_position, num_ways) in to_evaluate {
            self.handle_beam_to(beam_position, num_ways)
        }

        true
    }

    fn splits(&self) -> usize {
        self.num_splits
    }

    fn final_paths(&self) -> usize {
        self.num_final_paths
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let map = CellMap::<Tile>::new(reader)?;
    let mut manifold = Manifold::new(map);
    while manifold.move_beams() {}

    let result = match part {
        Part::Part1 => manifold.splits(),
        Part::Part2 => manifold.final_paths(),
    };

    println!("{result}");

    Ok(())
}
