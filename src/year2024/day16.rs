use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::part::Part;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Position {
    row: isize,
    col: isize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Direction {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

impl Direction {
    const ALL: [Self; 4] = [Self::Up, Self::Down, Self::Left, Self::Right];

    fn turn_left(self) -> Self {
        match self {
            Self::Up => Self::Left,
            Self::Down => Self::Right,
            Self::Left => Self::Down,
            Self::Right => Self::Up,
        }
    }

    fn turn_right(self) -> Self {
        match self {
            Self::Up => Self::Right,
            Self::Down => Self::Left,
            Self::Left => Self::Up,
            Self::Right => Self::Down,
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
    Start,
    End,
}

struct Maze {
    layout: Vec<Vec<Cell>>,
    width: isize,
    height: isize,
    start_position: Position,
}

struct MazeSolution {
    best_score: i64,
    positions_in_an_optimal_path: HashSet<Position>,
}

impl Maze {
    fn new<R: io::Read>(reader: io::BufReader<R>) -> io::Result<Self> {
        let mut width: Option<isize> = None;
        let mut layout: Vec<Vec<Cell>> = Vec::new();
        let mut start_position: Option<Position> = None;
        for (row, line) in reader.lines().enumerate() {
            let line = line?;
            let mut line_layout: Vec<Cell> = Vec::new();
            for (col, ch) in line.chars().enumerate() {
                let position = Position {
                    row: row as isize,
                    col: col as isize,
                };
                line_layout.push(match ch {
                    '.' => Cell::Empty,
                    '#' => Cell::Wall,
                    'S' => {
                        if start_position.is_some() {
                            return Err(invalid_input(
                                "Multiple start positions",
                            ));
                        }
                        start_position = Some(position);
                        Cell::Start
                    }
                    'E' => Cell::End,
                    _ => return Err(invalid_input("Unknown cell")),
                });
            }
            if let Some(current_width) = width {
                if current_width != line_layout.len() as isize {
                    return Err(invalid_input("Mismatched widths"));
                }
            } else {
                width = Some(line_layout.len() as isize);
            }

            layout.push(line_layout);
        }

        let Some(width) = width else {
            return Err(invalid_input("No lines"));
        };
        let height = layout.len() as isize;

        let Some(start_position) = start_position else {
            return Err(invalid_input("No start position"));
        };

        Ok(Self {
            layout,
            width,
            height,
            start_position,
        })
    }

    fn in_bounds(&self, position: Position) -> bool {
        position.row >= 0
            && position.row < self.height
            && position.col >= 0
            && position.col < self.width
    }

    fn at(&self, position: Position) -> Option<Cell> {
        if !self.in_bounds(position) {
            None
        } else {
            Some(self.layout[position.row as usize][position.col as usize])
        }
    }

    #[allow(dead_code)]
    fn print(&self) {
        for row_layout in self.layout.iter() {
            for &cell in row_layout.iter() {
                print!(
                    "{}",
                    match cell {
                        Cell::Empty => '.',
                        Cell::Wall => '#',
                        Cell::Start => 'S',
                        Cell::End => 'E',
                    }
                );
            }
            println!();
        }
    }

    fn solve(&self) -> MazeSolution {
        const TURN_COST: i64 = 1000;
        const MOVE_COST: i64 = 1;

        #[derive(PartialEq, Eq, PartialOrd, Ord)]
        struct ToVisit {
            score: i64,
            position: Position,
            facing: Direction,
        }

        let mut unvisited = BinaryHeap::<Reverse<ToVisit>>::new();
        let mut min_scores = HashMap::<(Position, Direction), i64>::new();
        let mut min_paths =
            HashMap::<(Position, Direction), Vec<Position>>::new();
        for row in 0..self.height {
            for col in 0..self.width {
                for direction in Direction::ALL {
                    let position = Position { row, col };
                    let score = if position == self.start_position
                        && direction == Direction::Right
                    {
                        0
                    } else {
                        i64::MAX
                    };
                    unvisited.push(Reverse(ToVisit {
                        score,
                        position,
                        facing: direction,
                    }));
                    min_scores.insert((position, direction), score);
                    min_paths.insert((position, direction), vec![position]);
                }
            }
        }

        let mut set_to_return = HashSet::<Position>::new();
        let mut value_to_return = i64::MAX;

        while let Some(Reverse(to_visit)) = unvisited.pop() {
            if to_visit.score > value_to_return {
                break;
            }

            if let Some(&other_score) =
                min_scores.get(&(to_visit.position, to_visit.facing))
            {
                if other_score < to_visit.score {
                    continue;
                }
            }

            let path = min_paths
                .get(&(to_visit.position, to_visit.facing))
                .unwrap();

            if self.at(to_visit.position) == Some(Cell::End) {
                set_to_return.extend(path);
                value_to_return = to_visit.score;
                continue;
            }

            let mut path_move = path.clone();
            path_move.push(to_visit.position.move_one(to_visit.facing));
            let move_one = (
                ToVisit {
                    score: to_visit.score + MOVE_COST,
                    position: to_visit.position.move_one(to_visit.facing),
                    facing: to_visit.facing,
                },
                path_move,
            );
            let turn_left = (
                ToVisit {
                    score: to_visit.score + TURN_COST,
                    position: to_visit.position,
                    facing: to_visit.facing.turn_left(),
                },
                path.clone(),
            );
            let turn_right = (
                ToVisit {
                    score: to_visit.score + TURN_COST,
                    position: to_visit.position,
                    facing: to_visit.facing.turn_right(),
                },
                path.clone(),
            );

            let neighbors = [move_one, turn_left, turn_right];

            for (neighbor_to_visit, neighbor_path) in neighbors {
                let position = neighbor_to_visit.position;
                let Some(cell) = self.at(position) else {
                    continue;
                };
                if cell == Cell::Wall {
                    continue;
                }

                let key = (position, neighbor_to_visit.facing);
                let existing_min_score =
                    min_scores.entry(key).or_insert(i64::MAX);
                let score = neighbor_to_visit.score;
                if score < *existing_min_score {
                    *existing_min_score = score;
                    unvisited.push(Reverse(neighbor_to_visit));
                    *min_paths.entry(key).or_insert_with(Vec::new) =
                        neighbor_path;
                } else if score == *existing_min_score {
                    min_paths
                        .entry(key)
                        .or_insert_with(Vec::new)
                        .extend(neighbor_path);
                }
            }
        }

        MazeSolution {
            best_score: value_to_return,
            positions_in_an_optimal_path: set_to_return,
        }
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let maze = Maze::new(reader)?;

    let solution = maze.solve();
    let result = match part {
        Part::Part1 => solution.best_score,
        Part::Part2 => solution
            .positions_in_an_optimal_path
            .len()
            .try_into()
            .unwrap(),
    };

    println!("{result}");

    Ok(())
}
