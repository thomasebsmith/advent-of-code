//use std::cmp::max;
//use std::collections::{HashMap, HashSet, VecDeque};
use std::io;

//use crate::errors::invalid_input;
//use crate::parse::lines;
use crate::part::Part;

/*
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum Tile {
    Path,
    Forest,
    Slope(Direction),
}

impl Tile {
    fn from_char(ch: char) -> io::Result<Self> {
        match ch {
            '.' => Ok(Self::Path),
            '#' => Ok(Self::Forest),
            '^' => Ok(Self::Slope(Direction::Up)),
            '>' => Ok(Self::Slope(Direction::Right)),
            'v' => Ok(Self::Slope(Direction::Down)),
            '<' => Ok(Self::Slope(Direction::Left)),
            _ => Err(invalid_input("Invalid tile character")),
        }
    }
}

// TODO: Bounds checking??
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

#[derive(Clone, Copy)]
struct Edge {
    vertex_index: usize,
    length: usize,
}

struct Vertex {
    neighbors: Vec<Edge>,
}

struct HikingTrails {
    vertices: Vec<Vertex>,
    starting_vertex_index: usize,
    goal_vertex_index: usize,
    /*map: Vec<Vec<Tile>>,
    starting_position: Position,
    goal_position: Position,
    width: usize,
    height: usize,*/
}

impl HikingTrails {
    fn from_reader<R: io::Read>(reader: io::BufReader<R>) -> io::Result<Self> {
        let mut map = Vec::<Vec<Tile>>::new();
        let mut width: Option<usize> = None;
        let mut starting_position: Option<Position> = None;
        let mut goal_position: Option<Position> = None;

        for line in lines(reader)? {
            let mut row = Vec::<Tile>::new();
            for ch in line.chars() {
                let tile = Tile::from_char(ch)?;
                if map.is_empty() && tile == Tile::Path {
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
                if tile == Tile::Path {
                    goal_position = Some(Position {
                        row: map.len() as isize,
                        col: row.len() as isize,
                    });
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
        let Some(goal_position) = goal_position else {
            return Err(invalid_input("No goal position"));
        };

        // Convert to graph
        let mut vertices = Vec::<Vertex>::new();
        let mut vertex_locations = HashMap::<Position, usize>::new();
        let mut occupied_by_edges = HashSet::<Position>::new();
        vertices.push(Vertex { neighbors: Vec::new() });
        vertex_locations.insert(starting_position, 0);

        if goal_position != starting_position {
            vertices.push(Vertex { neighbors: Vec::new() });
            vertex_locations.insert(goal_position, 1);
        }

        let mut paths = VecDeque::<Position>::new();
        paths.push_back(starting_position);

        while let Some(position) = paths.pop_back() {
            // TODO
            // check neighbors
            // if only 1, continue the current edge that way
            // if more than 1, finsh the edge and create a vertex if there isn't one here
            // add to occupied_by_edges as appropriate
        }

        Ok(Self {
            vertices,
            starting_vertex_index: 0,
            goal_vertex_index: if goal_position != starting_position { 1 } else { 0 },
        })
    }

    fn tile_at(&self, position: Position) -> Option<Tile> {
        if position.row < 0 || position.row >= self.height as isize ||
           position.col < 0 || position.col >= self.width as isize {
           None
        } else {
            Some(self.map[position.row as usize][position.col as usize])
        }
    }

    fn longest_hike(&self, can_climb_slopes: bool) -> usize {
        let mut longest_distance = 0usize;
        let mut visit_queue = VecDeque::<(Position, HashSet<Position>, usize)>::new(); // TODO: This is probably slow
        visit_queue.push_back((self.starting_position, HashSet::new(), 0));

        while let Some((position, mut visited, mut num_visited)) = visit_queue.pop_back() {
            if position == self.goal_position {
                /*if num_visited > longest_distance {
                    println!("Longest distance is now {num_visited}");
                }*/
                longest_distance = max(longest_distance, num_visited);
                continue;
            }

            visited.insert(position);
            num_visited += 1;

            let directions = if can_climb_slopes {
                vec![Direction::Left, Direction::Right, Direction::Up, Direction::Down]
            } else {
                match self.tile_at(position) {
                    Some(Tile::Path) => vec![Direction::Left, Direction::Right, Direction::Up, Direction::Down],
                    Some(Tile::Slope(direction)) => vec![direction],
                    _ => panic!("Unexpected tile type - invalid internal state")
                }
            };

            let mut to_visit = Vec::<Position>::new();
            for direction in directions {
                let neighbor_position = position.moved(direction);
                let can_visit_neighbor = !visited.contains(&neighbor_position) && matches!(self.tile_at(neighbor_position), Some(Tile::Path | Tile::Slope(_)));
                if can_visit_neighbor {
                    to_visit.push(neighbor_position);
                }
            }

            if to_visit.len() == 0 {
                continue;
            }

            for i in 0..(to_visit.len() - 1) {
                visit_queue.push_back((to_visit[i], visited.clone(), num_visited));
            }
            visit_queue.push_back((to_visit[to_visit.len() - 1], visited, num_visited));
        }

        longest_distance
    }
}*/

pub fn run<R: io::Read>(
    _part: Part,
    _reader: io::BufReader<R>,
) -> io::Result<()> {
    //let trails = HikingTrails::from_reader(reader)?;

    let result = 0usize; //trails.longest_hike(part == Part::Part2);

    println!("{result}");

    Ok(())
}
