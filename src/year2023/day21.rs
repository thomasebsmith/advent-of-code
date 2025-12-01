use std::collections::{HashMap, HashSet, VecDeque};
use std::io;

use crate::errors::invalid_input;
use crate::parse::lines;
use crate::part::Part;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum Tile {
    Start,
    Plot,
    Rock,
}

impl Tile {
    fn visitable(self) -> bool {
        match self {
            Self::Start | Self::Plot => true,
            Self::Rock => false,
        }
    }

    fn from_char(ch: char) -> io::Result<Self> {
        match ch {
            'S' => Ok(Self::Start),
            '.' => Ok(Self::Plot),
            '#' => Ok(Self::Rock),
            _ => Err(invalid_input("Invalid tile character")),
        }
    }
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
    row: isize,
    col: isize,
}

impl Position {
    fn moved(self, direction: Direction) -> Self {
        match direction {
            Direction::Left => Self {
                row: self.row,
                col: self.col - 1,
            },
            Direction::Right => Self {
                row: self.row,
                col: self.col + 1,
            },
            Direction::Up => Self {
                row: self.row - 1,
                col: self.col,
            },
            Direction::Down => Self {
                row: self.row + 1,
                col: self.col,
            },
        }
    }
}

struct Map {
    map: Vec<Vec<Tile>>,
    starting_position: Position,
    width: usize,
    height: usize,
}

impl Map {
    fn from_reader<R: io::Read>(reader: io::BufReader<R>) -> io::Result<Self> {
        let mut map = Vec::<Vec<Tile>>::new();
        let mut width: Option<usize> = None;
        let mut starting_position: Option<Position> = None;

        for line in lines(reader)? {
            let mut row = Vec::<Tile>::new();
            for ch in line.chars() {
                let tile = Tile::from_char(ch)?;
                if tile == Tile::Start {
                    match starting_position {
                        None => {
                            starting_position = Some(Position {
                                row: map.len() as isize,
                                col: row.len() as isize,
                            });
                        }
                        Some(_) => {
                            return Err(invalid_input(
                                "Multiple starting positions",
                            ));
                        }
                    }
                }
                row.push(tile);
            }
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

        let Some(starting_position) = starting_position else {
            return Err(invalid_input("No starting position"));
        };

        Ok(Self {
            map,
            starting_position,
            width,
            height,
        })
    }

    fn tile_at(&self, position: Position, loop_edges: bool) -> Option<Tile> {
        let width = self.width as isize;
        let height = self.height as isize;

        if !loop_edges
            && (position.row < 0
                || position.row >= height
                || position.col < 0
                || position.col >= width)
        {
            None
        } else {
            Some(
                self.map[position.row.rem_euclid(height) as usize]
                    [position.col.rem_euclid(width as isize) as usize],
            )
        }
    }

    fn visit_from(
        &self,
        num_steps: usize,
        from_loc: Position,
        loop_edges: bool,
    ) -> (usize, usize, usize) {
        let mut visited = HashSet::<Position>::new();
        let mut num_visited_exact: usize = 0;
        let mut max_steps: usize = 0;

        let mut visit_queue = VecDeque::<(Position, usize)>::new();
        visit_queue.push_back((from_loc, 0));

        while let Some((to_visit, steps_to_get_here)) = visit_queue.pop_front()
        {
            if visited.contains(&to_visit) {
                continue;
            }

            let tile = self.tile_at(to_visit, loop_edges).unwrap();
            assert!(tile.visitable());

            visited.insert(to_visit);
            if steps_to_get_here > max_steps {
                max_steps = steps_to_get_here;
            }
            if steps_to_get_here % 2 == num_steps % 2 {
                num_visited_exact += 1;
            }

            for direction in [
                Direction::Left,
                Direction::Right,
                Direction::Up,
                Direction::Down,
            ] {
                let neighbor_position = to_visit.moved(direction);
                if let Some(neighbor_tile) =
                    self.tile_at(neighbor_position, loop_edges)
                {
                    if neighbor_tile.visitable()
                        && steps_to_get_here < num_steps
                    {
                        visit_queue.push_back((
                            neighbor_position,
                            steps_to_get_here + 1,
                        ));
                    }
                }
            }
        }

        (max_steps, visited.len(), num_visited_exact)
    }

    fn is_optimizable(&self) -> bool {
        // If these conditions hold, an optimization allows us to solve part 2
        // more quickly.
        for col in 0..self.width {
            if !self.map[0][col].visitable() {
                return false;
            }
            if !self.map[self.starting_position.row as usize][col].visitable() {
                return false;
            }
            if !self.map[self.height - 1][col].visitable() {
                return false;
            }
        }
        for row in 0..self.height {
            if !self.map[row][0].visitable() {
                return false;
            }
            if !self.map[row][self.starting_position.col as usize].visitable() {
                return false;
            }
            if !self.map[row][self.width - 1].visitable() {
                return false;
            }
        }
        true
    }

    fn num_visitable_in_exactly(
        &self,
        num_steps: usize,
        loop_edges: bool,
    ) -> usize {
        if !loop_edges {
            self.visit_from(num_steps, self.starting_position, false).2
        } else {
            if !self.is_optimizable() {
                println!(
                    "Warning: Unable to optimize for part 2. This may be slow."
                );
                return self
                    .visit_from(num_steps, self.starting_position, true)
                    .2;
            }
            struct LocationCache {
                max_steps: usize,
                max_reachable_even: usize,
                max_reachable_odd: usize,
                reachable_by_steps: HashMap<usize, usize>,
            }

            let middle_right = Position {
                row: self.starting_position.row,
                col: self.width as isize - 1,
            };
            let middle_left = Position {
                row: self.starting_position.row,
                col: 0,
            };
            let bottom_middle = Position {
                row: self.height as isize - 1,
                col: self.starting_position.col,
            };
            let top_middle = Position {
                row: 0,
                col: self.starting_position.col,
            };
            let bottom_right = Position {
                row: self.height as isize - 1,
                col: self.width as isize - 1,
            };
            let bottom_left = Position {
                row: self.height as isize - 1,
                col: 0,
            };
            let top_right = Position {
                row: 0,
                col: self.width as isize - 1,
            };
            let top_left = Position { row: 0, col: 0 };

            let locs = [
                self.starting_position,
                middle_right,
                middle_left,
                bottom_middle,
                top_middle,
                bottom_right,
                bottom_left,
                top_right,
                top_left,
            ];

            let mut cache: [LocationCache; 9] = std::array::from_fn(|index| {
                let (max_steps, total_visited, visited_exact) =
                    self.visit_from(usize::MAX, locs[index], false);
                let (visited_even, visited_odd) = if usize::MAX % 2 == 0 {
                    (visited_exact, total_visited - visited_exact)
                } else {
                    (total_visited - visited_exact, visited_exact)
                };

                LocationCache {
                    max_steps,
                    max_reachable_even: visited_even,
                    max_reachable_odd: visited_odd,
                    reachable_by_steps: HashMap::new(),
                }
            });

            let mut result: usize = 0;

            let vertical_radius = (num_steps / self.height) as isize + 1;
            let horizontal_radius = (num_steps / self.width) as isize + 1;
            for row in -vertical_radius..=vertical_radius {
                for col in -horizontal_radius..=horizontal_radius {
                    let (dist, origin_index) = if row == 0 && col == 0 {
                        (0, 0)
                    } else if row == 0 && col < 0 {
                        // Middle right is closest
                        (
                            self.starting_position.col
                                + 1
                                + self.width as isize * (col.abs() - 1),
                            1,
                        )
                    } else if row == 0 && col > 0 {
                        // Middle left is closest
                        (
                            self.width as isize - self.starting_position.col
                                + self.width as isize * (col - 1),
                            2,
                        )
                    } else if row < 0 && col == 0 {
                        // Bottom middle is closest
                        (
                            self.starting_position.row
                                + 1
                                + self.height as isize * (row.abs() - 1),
                            3,
                        )
                    } else if row > 0 && col == 0 {
                        // Top middle is closest
                        (
                            self.height as isize - self.starting_position.row
                                + self.height as isize * (row - 1),
                            4,
                        )
                    } else if row < 0 && col < 0 {
                        // Bottom right is closest
                        (
                            self.starting_position.row
                                + 1
                                + self.starting_position.col
                                + 1
                                + self.height as isize * (row.abs() - 1)
                                + self.width as isize * (col.abs() - 1),
                            5,
                        )
                    } else if row < 0 && col > 0 {
                        // Bottom left is closest
                        (
                            self.starting_position.row
                                + 1
                                + self.width as isize
                                - self.starting_position.col
                                + self.height as isize * (row.abs() - 1)
                                + self.width as isize * (col - 1),
                            6,
                        )
                    } else if row > 0 && col < 0 {
                        // Top right is closest
                        (
                            self.height as isize - self.starting_position.row
                                + self.starting_position.col
                                + 1
                                + self.height as isize * (row - 1)
                                + self.width as isize * (col.abs() - 1),
                            7,
                        )
                    } else {
                        // if row > 0 && col > 0 {
                        // Top left is closest
                        (
                            self.height as isize - self.starting_position.row
                                + self.width as isize
                                - self.starting_position.col
                                + self.height as isize * (row - 1)
                                + self.width as isize * (col - 1),
                            8,
                        )
                    };

                    let remaining = num_steps as isize - dist;
                    if remaining < 0 {
                        continue;
                    }
                    let remaining = remaining as usize;

                    let cache_entry = &mut cache[origin_index];
                    if remaining >= cache_entry.max_steps {
                        result += if remaining % 2 == 0 {
                            cache_entry.max_reachable_even
                        } else {
                            cache_entry.max_reachable_odd
                        };
                    } else {
                        result += *cache_entry
                            .reachable_by_steps
                            .entry(remaining)
                            .or_insert_with(|| {
                                self.visit_from(
                                    remaining,
                                    locs[origin_index],
                                    false,
                                )
                                .2
                            });
                    }
                }
            }
            result
        }
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let map = Map::from_reader(reader)?;
    let num_steps: usize = match part {
        Part::Part1 => 64,
        Part::Part2 => 26501365,
    };
    let result = map.num_visitable_in_exactly(num_steps, part == Part::Part2);

    println!("{result}");

    Ok(())
}
