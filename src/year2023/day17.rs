use std::collections::VecDeque;
use std::io;

use crate::errors::invalid_input;
use crate::parse::lines;
use crate::part::Part;

const NUM_DIRECTIONS: usize = 4;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum Turn {
    Left,
    Straight,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    fn turn(self, turn: Turn) -> Self {
        match (self, turn) {
            (_, Turn::Straight) => self,
            (Self::Left, Turn::Left) => Self::Down,
            (Self::Left, Turn::Right) => Self::Up,
            (Self::Right, Turn::Left) => Self::Up,
            (Self::Right, Turn::Right) => Self::Down,
            (Self::Up, Turn::Left) => Self::Left,
            (Self::Up, Turn::Right) => Self::Right,
            (Self::Down, Turn::Left) => Self::Right,
            (Self::Down, Turn::Right) => Self::Left,
        }
    }
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

struct CrucibleTileState<
    const MIN_BLOCKS_SINGLE_DIRECTION: usize,
    const PER_TILE_STATE_SIZE: usize,
> {
    minimum_heat_loss_to_reach: [i64; PER_TILE_STATE_SIZE],
}

impl<
        const MIN_BLOCKS_SINGLE_DIRECTION: usize,
        const PER_TILE_STATE_SIZE: usize,
    > std::fmt::Debug
    for CrucibleTileState<MIN_BLOCKS_SINGLE_DIRECTION, PER_TILE_STATE_SIZE>
{
    fn fmt(
        &self,
        fmt: &mut std::fmt::Formatter<'_>,
    ) -> Result<(), std::fmt::Error> {
        let nth = |n: usize| {
            let num = self.minimum_heat_loss_to_reach[n];
            if num == i64::MAX {
                "-".to_owned()
            } else {
                num.to_string()
            }
        };

        let max_blocks = PER_TILE_STATE_SIZE / NUM_DIRECTIONS;
        write!(fmt, "< ")?;
        for i in 0..PER_TILE_STATE_SIZE {
            write!(fmt, "[{}]", nth(i))?;
            if i % max_blocks == max_blocks - 1 {
                write!(fmt, " ")?;
            } else {
                write!(fmt, "-")?;
            }
        }
        write!(fmt, " >")?;
        Ok(())
    }
}

impl<
        const MIN_BLOCKS_SINGLE_DIRECTION: usize,
        const PER_TILE_STATE_SIZE: usize,
    > CrucibleTileState<MIN_BLOCKS_SINGLE_DIRECTION, PER_TILE_STATE_SIZE>
{
    fn new() -> Self {
        Self {
            minimum_heat_loss_to_reach: [i64::MAX; PER_TILE_STATE_SIZE],
        }
    }

    fn min_heat_loss(
        &mut self,
        entry_direction: Direction,
        blocks_moved_before_entry: usize,
    ) -> &mut i64 {
        let max_blocks = PER_TILE_STATE_SIZE / NUM_DIRECTIONS;
        assert!(blocks_moved_before_entry < max_blocks);
        &mut self.minimum_heat_loss_to_reach
            [entry_direction as usize * max_blocks + blocks_moved_before_entry]
    }

    fn min_heat_loss_any(&self) -> i64 {
        let max_blocks = PER_TILE_STATE_SIZE / NUM_DIRECTIONS;
        let mut min = i64::MAX;
        for direction in 0..NUM_DIRECTIONS {
            for blocks in (MIN_BLOCKS_SINGLE_DIRECTION - 1)..max_blocks {
                let mhl = self.minimum_heat_loss_to_reach
                    [direction * max_blocks + blocks];
                if mhl < min {
                    min = mhl;
                }
            }
        }
        min
    }
}

struct CrucibleState<
    const MIN_BLOCKS_SINGLE_DIRECTION: usize,
    const PER_TILE_STATE_SIZE: usize,
> {
    map: Vec<
        Vec<
            CrucibleTileState<MIN_BLOCKS_SINGLE_DIRECTION, PER_TILE_STATE_SIZE>,
        >,
    >,
}

impl<
        const MIN_BLOCKS_SINGLE_DIRECTION: usize,
        const PER_TILE_STATE_SIZE: usize,
    > CrucibleState<MIN_BLOCKS_SINGLE_DIRECTION, PER_TILE_STATE_SIZE>
{
    fn new(width: usize, height: usize) -> Self {
        let mut map = Vec::<
            Vec<
                CrucibleTileState<
                    MIN_BLOCKS_SINGLE_DIRECTION,
                    PER_TILE_STATE_SIZE,
                >,
            >,
        >::new();
        for _ in 0..height {
            let mut row = Vec::<
                CrucibleTileState<
                    MIN_BLOCKS_SINGLE_DIRECTION,
                    PER_TILE_STATE_SIZE,
                >,
            >::new();
            for _ in 0..width {
                row.push(CrucibleTileState::new());
            }
            map.push(row);
        }
        Self { map }
    }
}

struct CityBlocks {
    map: Vec<Vec<i64>>,
    width: usize,
    height: usize,
}

impl CityBlocks {
    fn from_reader<R: io::Read>(reader: io::BufReader<R>) -> io::Result<Self> {
        let mut map = Vec::<Vec<i64>>::new();
        let mut width: Option<usize> = None;

        for line in lines(reader)? {
            let row = line
                .chars()
                .map(|ch| {
                    TryInto::<i64>::try_into(
                        ch.to_digit(10)
                            .ok_or_else(|| invalid_input("Invalid digit"))?,
                    )
                    .map_err(invalid_input)
                })
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

    fn min_heat_loss<
        const MIN_BLOCKS_SINGLE_DIRECTION: usize,
        const MAX_BLOCKS_SINGLE_DIRECTION: usize,
        const PER_TILE_STATE_SIZE: usize,
    >(
        &self,
    ) -> i64 {
        struct Place {
            position: Position,
            direction: Direction,
            blocks_moved_before_entry: usize,
            heat_loss_so_far: i64,
        }

        let mut state = CrucibleState::<
            MIN_BLOCKS_SINGLE_DIRECTION,
            PER_TILE_STATE_SIZE,
        >::new(self.width, self.height);
        let mut places = VecDeque::<Place>::new();
        places.push_back(Place {
            position: Position { row: 0, col: 0 },
            direction: Direction::Right,
            blocks_moved_before_entry: 0,
            heat_loss_so_far: 0,
        });

        // This is pretty slow. It could probably be optimized by pruning duplicate position +
        // direction + bmbe combos.
        while let Some(place) = places.pop_front() {
            let hlsf_ref = state.map[place.position.row][place.position.col]
                .min_heat_loss(
                    place.direction,
                    place.blocks_moved_before_entry,
                );
            if place.heat_loss_so_far < *hlsf_ref {
                *hlsf_ref = place.heat_loss_so_far;

                let turns = if place.blocks_moved_before_entry + 1
                    < MIN_BLOCKS_SINGLE_DIRECTION
                {
                    assert!(
                        place.blocks_moved_before_entry
                            < MAX_BLOCKS_SINGLE_DIRECTION - 1
                    );
                    vec![Turn::Straight]
                } else if place.blocks_moved_before_entry
                    < MAX_BLOCKS_SINGLE_DIRECTION - 1
                {
                    vec![Turn::Left, Turn::Right, Turn::Straight]
                } else {
                    vec![Turn::Left, Turn::Right]
                };
                for turn in turns {
                    let new_direction = place.direction.turn(turn);
                    let Some(new_position) = place.position.moved(
                        new_direction,
                        self.width,
                        self.height,
                    ) else {
                        continue;
                    };
                    places.push_back(Place {
                        position: new_position,
                        direction: new_direction,
                        blocks_moved_before_entry: if turn == Turn::Straight {
                            place.blocks_moved_before_entry + 1
                        } else {
                            0
                        },
                        heat_loss_so_far: place.heat_loss_so_far
                            + self.map[new_position.row][new_position.col],
                    });
                }
            }
        }

        state.map[self.height - 1][self.width - 1].min_heat_loss_any()
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let city_blocks = CityBlocks::from_reader(reader)?;

    let result = match part {
        Part::Part1 => {
            city_blocks.min_heat_loss::<1, 3, { 3 * NUM_DIRECTIONS }>()
        }
        Part::Part2 => {
            city_blocks.min_heat_loss::<4, 10, { 10 * NUM_DIRECTIONS }>()
        }
    };

    println!("{result}");

    Ok(())
}
