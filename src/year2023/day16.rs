use std::collections::HashMap;
use std::io;

use crate::errors::invalid_input;
use crate::parse::lines;
use crate::part::Part;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum Tile {
    Empty,
    MirrorLeaningRight,
    MirrorLeaningLeft,
    VerticalSplitter,
    HorizontalSplitter,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
struct Position {
    row: usize,
    col: usize,
}

impl Position {
    fn moved(
        self,
        direction: Direction,
        width: usize,
        height: usize,
    ) -> Option<Self> {
        match direction {
            Direction::Left => {
                if self.col == 0 {
                    None
                } else {
                    Some(Self {
                        row: self.row,
                        col: self.col - 1,
                    })
                }
            }
            Direction::Right => {
                if self.col == width - 1 {
                    None
                } else {
                    Some(Self {
                        row: self.row,
                        col: self.col + 1,
                    })
                }
            }
            Direction::Up => {
                if self.row == 0 {
                    None
                } else {
                    Some(Self {
                        row: self.row - 1,
                        col: self.col,
                    })
                }
            }
            Direction::Down => {
                if self.row == height - 1 {
                    None
                } else {
                    Some(Self {
                        row: self.row + 1,
                        col: self.col,
                    })
                }
            }
        }
    }
}

impl Tile {
    fn from_char(ch: char) -> io::Result<Self> {
        match ch {
            '.' => Ok(Self::Empty),
            '/' => Ok(Self::MirrorLeaningRight),
            '\\' => Ok(Self::MirrorLeaningLeft),
            '|' => Ok(Self::VerticalSplitter),
            '-' => Ok(Self::HorizontalSplitter),
            _ => Err(invalid_input("Invalid tile character")),
        }
    }
}

#[derive(Clone)]
struct TileState {
    tile: Tile,
    beams: [bool; 4],
}

impl TileState {
    fn from_char(ch: char) -> io::Result<Self> {
        Tile::from_char(ch).map(|tile| Self {
            tile,
            beams: [false, false, false, false],
        })
    }

    fn energized(&self) -> bool {
        self.beams.iter().any(|beam| *beam)
    }
}

#[derive(Clone)]
struct Contraption {
    map: Vec<Vec<TileState>>,
    width: usize,
    height: usize,
}

impl Contraption {
    fn from_reader<R: io::Read>(reader: io::BufReader<R>) -> io::Result<Self> {
        let mut map = Vec::<Vec<TileState>>::new();
        let mut width: Option<usize> = None;

        for line in lines(reader)? {
            let row = line
                .chars()
                .map(TileState::from_char)
                .collect::<io::Result<Vec<_>>>()?;
            if let Some(the_width) = width {
                if row.len() != the_width {
                    return Err(invalid_input("Differing row widths"));
                }
            } else {
                width = Some(row.len());
            }
            map.push(row);
        }

        let height = map.len();
        if height == 0 || width.unwrap() == 0 {
            return Err(invalid_input("Empty map"));
        }

        let width = width.unwrap();

        Ok(Self { map, width, height })
    }

    fn add_beam(&mut self, position: Position, direction: Direction) {
        assert!(position.row < self.height && position.col < self.width);

        let tile_state = &mut self.map[position.row][position.col];

        if tile_state.beams[direction as usize] {
            return;
        }

        tile_state.beams[direction as usize] = true;

        let new_directions = match tile_state.tile {
            Tile::Empty => vec![direction],
            Tile::MirrorLeaningRight => vec![match direction {
                Direction::Left => Direction::Down,
                Direction::Right => Direction::Up,
                Direction::Up => Direction::Right,
                Direction::Down => Direction::Left,
            }],
            Tile::MirrorLeaningLeft => vec![match direction {
                Direction::Left => Direction::Up,
                Direction::Right => Direction::Down,
                Direction::Up => Direction::Left,
                Direction::Down => Direction::Right,
            }],
            Tile::VerticalSplitter => match direction {
                Direction::Left | Direction::Right => {
                    vec![Direction::Up, Direction::Down]
                }
                _ => vec![direction],
            },
            Tile::HorizontalSplitter => match direction {
                Direction::Up | Direction::Down => {
                    vec![Direction::Left, Direction::Right]
                }
                _ => vec![direction],
            },
        };

        for new_direction in new_directions {
            if let Some(new_position) =
                position.moved(new_direction, self.width, self.height)
            {
                self.add_beam(new_position, new_direction);
            }
        }
    }

    fn num_energized_tiles(&self) -> usize {
        self.map
            .iter()
            .map(|row| {
                row.iter()
                    .map(
                        |tile_state| if tile_state.energized() { 1 } else { 0 },
                    )
                    .sum::<usize>()
            })
            .sum()
    }

    fn edge_vectors(&self) -> Vec<(Position, Direction)> {
        let mut result = Vec::<(Position, Direction)>::new();
        for row in 0..self.height {
            result.push((Position { row, col: 0 }, Direction::Right));
            result.push((
                Position {
                    row,
                    col: self.width - 1,
                },
                Direction::Left,
            ));
        }
        for col in 0..self.width {
            result.push((Position { row: 0, col }, Direction::Down));
            result.push((
                Position {
                    row: self.height - 1,
                    col,
                },
                Direction::Up,
            ));
        }
        result
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut contraption = Contraption::from_reader(reader)?;

    let result = match part {
        Part::Part1 => {
            contraption.add_beam(Position { row: 0, col: 0 }, Direction::Right);
            contraption.num_energized_tiles()
        }
        Part::Part2 => {
            // Not the most efficient, but it'll work.
            // Ideally, we would only recreate the beam state instead of the
            // tile map.
            contraption
                .edge_vectors()
                .into_iter()
                .map(|(position, direction)| {
                    let mut clone = contraption.clone();
                    clone.add_beam(position, direction);
                    clone.num_energized_tiles()
                })
                .max()
                .unwrap()
        }
    };

    println!("{result}");

    Ok(())
}
