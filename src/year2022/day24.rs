use std::collections::HashSet;
use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::part::Part;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Hurricanes {
    going_left: u32,
    going_right: u32,
    going_up: u32,
    going_down: u32,
}

impl Hurricanes {
    pub fn left(n: u32) -> Self {
        Hurricanes {
            going_left: n,
            going_right: 0,
            going_up: 0,
            going_down: 0,
        }
    }

    pub fn right(n: u32) -> Self {
        Hurricanes {
            going_left: 0,
            going_right: n,
            going_up: 0,
            going_down: 0,
        }
    }

    pub fn up(n: u32) -> Self {
        Hurricanes {
            going_left: 0,
            going_right: 0,
            going_up: n,
            going_down: 0,
        }
    }

    pub fn down(n: u32) -> Self {
        Hurricanes {
            going_left: 0,
            going_right: 0,
            going_up: 0,
            going_down: n,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tile {
    Wall,
    Ground,
    Stormy(Hurricanes),
}

impl Tile {
    pub fn new(ch: char) -> Option<Self> {
        match ch {
            '#' => Some(Self::Wall),
            '.' => Some(Self::Ground),
            '<' => Some(Self::Stormy(Hurricanes::left(1))),
            '>' => Some(Self::Stormy(Hurricanes::right(1))),
            '^' => Some(Self::Stormy(Hurricanes::up(1))),
            'v' => Some(Self::Stormy(Hurricanes::down(1))),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Point {
    pub x: usize,
    pub y: usize,
}

impl Point {
    pub fn corner_dist(self, other: Point) -> usize {
        let x_diff = if self.x > other.x {
            self.x - other.x
        } else {
            other.x - self.x
        };
        let y_diff = if self.y > other.y {
            self.y - other.y
        } else {
            other.y - self.y
        };
        x_diff + y_diff
    }
}

struct TilesCache {
    cache: Vec<Vec<Vec<Tile>>>,
    width: usize,
    height: usize,
}

impl TilesCache {
    pub fn new(initial: Vec<Vec<Tile>>) -> io::Result<Self> {
        let height = initial.len();
        if height == 0 {
            Err(invalid_input("Empty tile map"))?
        }
        let width = initial[0].len();
        Ok(Self {
            cache: vec![initial],
            width,
            height,
        })
    }

    fn moved_blizzards(&self, tiles: &Vec<Vec<Tile>>) -> Vec<Vec<Tile>> {
        let mut new_tiles = tiles.clone();
        for row in &mut new_tiles {
            for tile in row {
                if matches!(*tile, Tile::Stormy(_)) {
                    *tile = Tile::Ground;
                }
            }
        }

        for y in 0..self.height {
            for x in 0..self.width {
                let Tile::Stormy(hurricanes) = tiles[y][x] else {
                    continue;
                };

                // TODO assumes hurricane never goes out of bounds
                // TODO position assumptions
                // TODO dup
                if hurricanes.going_left != 0 {
                    let mut new_tile = &mut new_tiles[y][x - 1];
                    if *new_tile == Tile::Wall {
                        new_tile = &mut new_tiles[y][self.width - 2];
                    }

                    match *new_tile {
                        Tile::Ground => {
                            *new_tile = Tile::Stormy(Hurricanes::left(
                                hurricanes.going_left,
                            ));
                        }
                        Tile::Stormy(ref mut new_hurricanes) => {
                            new_hurricanes.going_left += hurricanes.going_left;
                        }
                        _ => panic!("Unexpected wall"),
                    }
                }

                if hurricanes.going_right != 0 {
                    let mut new_tile = &mut new_tiles[y][x + 1];
                    if *new_tile == Tile::Wall {
                        new_tile = &mut new_tiles[y][1];
                    }

                    match *new_tile {
                        Tile::Ground => {
                            *new_tile = Tile::Stormy(Hurricanes::right(
                                hurricanes.going_right,
                            ));
                        }
                        Tile::Stormy(ref mut new_hurricanes) => {
                            new_hurricanes.going_right +=
                                hurricanes.going_right;
                        }
                        _ => panic!("Unexpected wall"),
                    }
                }

                if hurricanes.going_up != 0 {
                    let mut new_tile = &mut new_tiles[y - 1][x];
                    if *new_tile == Tile::Wall {
                        new_tile = &mut new_tiles[self.height - 2][x];
                    }

                    match *new_tile {
                        Tile::Ground => {
                            *new_tile = Tile::Stormy(Hurricanes::up(
                                hurricanes.going_up,
                            ));
                        }
                        Tile::Stormy(ref mut new_hurricanes) => {
                            new_hurricanes.going_up += hurricanes.going_up;
                        }
                        _ => panic!("Unexpected wall"),
                    }
                }

                if hurricanes.going_down != 0 {
                    let mut new_tile = &mut new_tiles[y + 1][x];
                    if *new_tile == Tile::Wall {
                        new_tile = &mut new_tiles[1][x];
                    }

                    match *new_tile {
                        Tile::Ground => {
                            *new_tile = Tile::Stormy(Hurricanes::down(
                                hurricanes.going_down,
                            ));
                        }
                        Tile::Stormy(ref mut new_hurricanes) => {
                            new_hurricanes.going_down += hurricanes.going_down;
                        }
                        _ => panic!("Unexpected wall"),
                    }
                }
            }
        }

        new_tiles
    }

    pub fn get(&mut self, minute: usize) -> &Vec<Vec<Tile>> {
        while minute >= self.cache.len() {
            let new_entry = self.moved_blizzards(self.cache.last().unwrap());
            self.cache.push(new_entry);
        }
        &self.cache[minute]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct SearchState {
    minutes_elapsed: usize,
    position: Point,
}

struct Map {
    tiles: Vec<Vec<Tile>>,
    width: usize,
    position: Point,
    goal: Point,
}

impl Map {
    pub fn new<'a, I: Iterator<Item = &'a String>>(
        lines: I,
    ) -> io::Result<Self> {
        let mut width: Option<usize> = None;

        let mut tiles = Vec::<Vec<Tile>>::new();

        for line in lines {
            let this_line_tiles = line
                .chars()
                .map(Tile::new)
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| invalid_input("Unknown tile character"))?;

            match width {
                None => {
                    width = Some(this_line_tiles.len());
                }
                Some(len) => {
                    if this_line_tiles.len() != len {
                        Err(invalid_input("Differing line lengths"))?
                    }
                }
            }

            tiles.push(this_line_tiles);
        }

        let Some(width) = width else {
            Err(invalid_input("No lines parsed"))?
        };

        let position = tiles
            .first()
            .unwrap()
            .iter()
            .enumerate()
            .find_map(|(x, tile)| {
                if *tile == Tile::Ground {
                    Some(Point { x, y: 0 })
                } else {
                    None
                }
            })
            .ok_or_else(|| invalid_input("No starting position found"))?;

        let goal = tiles
            .last()
            .unwrap()
            .iter()
            .enumerate()
            .find_map(|(x, tile)| {
                if *tile == Tile::Ground {
                    Some(Point {
                        x,
                        y: tiles.len() - 1,
                    })
                } else {
                    None
                }
            })
            .ok_or_else(|| invalid_input("No goal found"))?;

        Ok(Self {
            tiles,
            width,
            position,
            goal,
        })
    }

    pub fn shortest_time(
        &self,
        starting_minute: usize,
        backwards: bool,
    ) -> io::Result<usize> {
        let mut to_try = Vec::<SearchState>::new();

        let mut tiles_cache = TilesCache::new(self.tiles.clone())?;
        let mut seen = HashSet::<SearchState>::new();

        let mut min_minutes: Option<usize> = None;

        let (position, goal) = if backwards {
            (self.goal, self.position)
        } else {
            (self.position, self.goal)
        };

        to_try.push(SearchState {
            minutes_elapsed: starting_minute,
            position,
        });

        while let Some(state) = to_try.pop() {
            if seen.contains(&state) {
                continue;
            }
            seen.insert(state);

            if state.position == goal {
                match min_minutes {
                    None => {
                        min_minutes = Some(state.minutes_elapsed);
                    }
                    Some(ref mut mins) => {
                        if state.minutes_elapsed < *mins {
                            *mins = state.minutes_elapsed;
                        }
                    }
                }
            } else {
                if let Some(min) = min_minutes {
                    if state.minutes_elapsed + state.position.corner_dist(goal)
                        >= min
                    {
                        continue;
                    }
                };

                let minutes_elapsed = state.minutes_elapsed + 1;
                let new_tiles = tiles_cache.get(minutes_elapsed);

                let mut new_states = Vec::<SearchState>::new();
                // Move left
                if state.position.x > 0
                    && new_tiles[state.position.y][state.position.x - 1]
                        == Tile::Ground
                {
                    new_states.push(SearchState {
                        minutes_elapsed,
                        position: Point {
                            x: state.position.x - 1,
                            y: state.position.y,
                        },
                    });
                }

                // Move up
                if state.position.y > 0
                    && new_tiles[state.position.y - 1][state.position.x]
                        == Tile::Ground
                {
                    new_states.push(SearchState {
                        minutes_elapsed,
                        position: Point {
                            x: state.position.x,
                            y: state.position.y - 1,
                        },
                    });
                }

                // Don't move
                if new_tiles[state.position.y][state.position.x] == Tile::Ground
                {
                    new_states.push(SearchState {
                        minutes_elapsed,
                        position: state.position,
                    });
                }

                // Move down
                if state.position.y < self.tiles.len() - 1
                    && new_tiles[state.position.y + 1][state.position.x]
                        == Tile::Ground
                {
                    new_states.push(SearchState {
                        minutes_elapsed,
                        position: Point {
                            x: state.position.x,
                            y: state.position.y + 1,
                        },
                    });
                }

                // Move right
                if state.position.x < self.width - 1
                    && new_tiles[state.position.y][state.position.x + 1]
                        == Tile::Ground
                {
                    new_states.push(SearchState {
                        minutes_elapsed,
                        position: Point {
                            x: state.position.x + 1,
                            y: state.position.y,
                        },
                    });
                }

                if backwards {
                    new_states.reverse();
                }
                to_try.append(&mut new_states);
            }
        }

        let result =
            min_minutes.ok_or_else(|| invalid_input("No path found"))?;

        Ok(result)
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let map = Map::new(reader.lines().collect::<io::Result<Vec<_>>>()?.iter())?;

    let result = match part {
        Part::Part1 => map.shortest_time(0, false)?,
        Part::Part2 => {
            let time1 = map.shortest_time(0, false)?;
            let time2 = map.shortest_time(time1, true)?;
            let time3 = map.shortest_time(time2, false)?;
            time3
        }
    };

    println!("{result}");

    Ok(())
}
