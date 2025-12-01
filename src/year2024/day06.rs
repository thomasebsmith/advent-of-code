use std::collections::HashSet;
use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::part::Part;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Position {
    row: isize,
    col: isize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn turn_right(self) -> Self {
        match self {
            Self::Up => Self::Right,
            Self::Right => Self::Down,
            Self::Down => Self::Left,
            Self::Left => Self::Up,
        }
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
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Cell {
    Empty,
    Wall,
}

struct Map {
    cells: Vec<Vec<Cell>>,
    width: isize,
    height: isize,
    guard_position: Position,
    guard_direction: Direction,
    original_guard_position: Position,
    original_guard_direction: Direction,
    guard_visited_locations: HashSet<Position>,
    guard_visited_vectors: HashSet<(Position, Direction)>,
}

impl Map {
    fn new<R: io::Read>(reader: io::BufReader<R>) -> io::Result<Self> {
        let mut width: Option<isize> = None;
        let mut cells: Vec<Vec<Cell>> = Vec::new();
        let mut guard_position: Option<Position> = None;
        let mut guard_direction: Option<Direction> = None;
        let mut guard_visited_locations: HashSet<Position> = HashSet::new();
        let mut guard_visited_vectors: HashSet<(Position, Direction)> =
            HashSet::new();

        for line in reader.lines() {
            let line = line?;
            let mut line_cells: Vec<Cell> = Vec::new();

            for ch in line.chars() {
                let current_position = Position {
                    row: cells.len() as isize,
                    col: line_cells.len() as isize,
                };
                line_cells.push(match ch {
                    '.' => Cell::Empty,
                    '#' => Cell::Wall,
                    '^' => {
                        guard_position = Some(current_position);
                        guard_direction = Some(Direction::Up);
                        Cell::Empty
                    }
                    '>' => {
                        guard_position = Some(current_position);
                        guard_direction = Some(Direction::Right);
                        Cell::Empty
                    }
                    '<' => {
                        guard_position = Some(current_position);
                        guard_direction = Some(Direction::Left);
                        Cell::Empty
                    }
                    'v' => {
                        guard_position = Some(current_position);
                        guard_direction = Some(Direction::Down);
                        Cell::Empty
                    }
                    _ => {
                        return Err(invalid_input("Invalid character"));
                    }
                });
            }

            if let Some(current_width) = width {
                if current_width != line_cells.len() as isize {
                    return Err(invalid_input("Mismatched widths"));
                }
            } else {
                width = Some(line_cells.len() as isize);
            }

            cells.push(line_cells);
        }
        let Some(width) = width else {
            return Err(invalid_input("No lines"));
        };

        let (Some(guard_position), Some(guard_direction)) =
            (guard_position, guard_direction)
        else {
            return Err(invalid_input("No guard"));
        };
        guard_visited_locations.insert(guard_position);
        guard_visited_vectors.insert((guard_position, guard_direction));

        let height = cells.len() as isize;
        Ok(Self {
            cells,
            width,
            height,
            guard_position,
            guard_direction,
            original_guard_position: guard_position,
            original_guard_direction: guard_direction,
            guard_visited_locations,
            guard_visited_vectors,
        })
    }

    fn reset_guard_info(&mut self) {
        self.guard_position = self.original_guard_position;
        self.guard_direction = self.original_guard_direction;
        self.guard_visited_locations.clear();
        self.guard_visited_locations.insert(self.guard_position);
        self.guard_visited_vectors.clear();
        self.guard_visited_vectors
            .insert((self.guard_position, self.guard_direction));
    }

    fn in_bounds(&self, position: Position) -> bool {
        position.row >= 0
            && position.row < self.height
            && position.col >= 0
            && position.col < self.width
    }

    fn at(&self, position: Position) -> Cell {
        if !self.in_bounds(position) {
            Cell::Empty
        } else {
            self.cells[position.row as usize][position.col as usize]
        }
    }

    fn at_mut(&mut self, position: Position) -> Option<&mut Cell> {
        if !self.in_bounds(position) {
            None
        } else {
            Some(&mut self.cells[position.row as usize][position.col as usize])
        }
    }

    fn step_guard(&mut self) -> bool {
        let new_position = self.guard_position.move_one(self.guard_direction);
        if self.at(new_position) != Cell::Empty {
            self.guard_direction = self.guard_direction.turn_right();
        } else {
            self.guard_position = new_position;
            self.guard_visited_locations.insert(new_position);
        }
        let is_new_vector = self
            .guard_visited_vectors
            .insert((self.guard_position, self.guard_direction));
        self.in_bounds(self.guard_position) && is_new_vector
    }

    fn num_in_bounds_visited_locations(&self) -> usize {
        self.guard_visited_locations
            .iter()
            .filter(|pos| self.in_bounds(**pos))
            .count()
    }

    fn try_simulate_loop(&mut self, position: Position) -> bool {
        self.reset_guard_info();
        if !self.in_bounds(position)
            || self.at(position) != Cell::Empty
            || self.guard_position == position
        {
            return false;
        }

        *self.at_mut(position).unwrap() = Cell::Wall;

        while self.step_guard() {
            // Loop until the guard steps out of bounds or loops
        }

        *self.at_mut(position).unwrap() = Cell::Empty;

        self.in_bounds(self.guard_position)
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut map = Map::new(reader)?;
    while map.step_guard() {
        // Loop until the guard steps out of bounds or loops
    }

    let result = match part {
        Part::Part1 => map.num_in_bounds_visited_locations(),
        Part::Part2 => {
            let visited_locations_unmodified =
                map.guard_visited_locations.clone();

            visited_locations_unmodified
                .into_iter()
                .filter(|pos| map.try_simulate_loop(*pos))
                .count()
        }
    };
    println!("{result}");

    Ok(())
}
