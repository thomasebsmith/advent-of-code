use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
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

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Cheat {
    saved: i64,
    start: Position,
    end: Position,
}

struct Maze {
    layout: Vec<Vec<Cell>>,
    width: isize,
    height: isize,
    start: Position,
    end: Position,
}

impl Maze {
    fn new<R: io::Read>(reader: io::BufReader<R>) -> io::Result<Self> {
        let mut width: Option<isize> = None;
        let mut layout: Vec<Vec<Cell>> = Vec::new();
        let mut start: Option<Position> = None;
        let mut end: Option<Position> = None;
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
                        if start.is_some() {
                            return Err(invalid_input(
                                "Multiple start positions",
                            ));
                        }
                        start = Some(position);
                        Cell::Empty
                    }
                    'E' => {
                        if end.is_some() {
                            return Err(invalid_input(
                                "Multiple end positions",
                            ));
                        }
                        end = Some(position);
                        Cell::Empty
                    }
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

        let Some(start) = start else {
            return Err(invalid_input("No start position"));
        };
        let Some(end) = end else {
            return Err(invalid_input("No start position"));
        };

        Ok(Self {
            layout,
            width,
            height,
            start,
            end,
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
        for (row, row_layout) in self.layout.iter().enumerate() {
            for (col, &cell) in row_layout.iter().enumerate() {
                let position = Position {
                    row: row as isize,
                    col: col as isize,
                };
                print!(
                    "{}",
                    if position == self.end {
                        'E'
                    } else if position == self.start {
                        'S'
                    } else {
                        match cell {
                            Cell::Empty => '.',
                            Cell::Wall => '#',
                        }
                    }
                );
            }
            println!();
        }
    }

    fn compute_end_distances(&self) -> HashMap<Position, i64> {
        #[derive(PartialEq, Eq, PartialOrd, Ord)]
        struct ToVisit {
            score: i64,
            position: Position,
        }

        let mut unvisited = BinaryHeap::<Reverse<ToVisit>>::new();
        let mut min_scores = HashMap::<Position, i64>::new();
        unvisited.push(Reverse(ToVisit {
            score: 0,
            position: self.end,
        }));
        min_scores.insert(self.end, 0);

        while let Some(Reverse(to_visit)) = unvisited.pop() {
            if let Some(&other_score) = min_scores.get(&to_visit.position) {
                if other_score < to_visit.score {
                    continue;
                }
            }

            let neighbors =
                Direction::ALL.into_iter().map(|direction| ToVisit {
                    score: to_visit.score + 1,
                    position: to_visit.position.move_one(direction),
                });

            for neighbor_to_visit in neighbors {
                let position = neighbor_to_visit.position;
                let Some(cell) = self.at(position) else {
                    continue;
                };
                if cell == Cell::Wall {
                    continue;
                }

                let existing_min_score =
                    min_scores.entry(position).or_insert(i64::MAX);
                let score = neighbor_to_visit.score;
                if score < *existing_min_score {
                    *existing_min_score = score;
                    unvisited.push(Reverse(neighbor_to_visit));
                }
            }
        }

        min_scores
    }

    fn compute_cheats(&self, max_cheat_length: i64) -> BinaryHeap<Cheat> {
        let mut result = BinaryHeap::<Cheat>::new();
        let end_distances = self.compute_end_distances();
        for row in 0..self.height {
            for col in 0..self.width {
                let cheat_start = Position { row, col };
                if let Some(cheat_start_dist) = end_distances.get(&cheat_start)
                {
                    for row_diff in -max_cheat_length..=max_cheat_length {
                        let dest_row = row + row_diff as isize;
                        let remaining_cheat_length =
                            max_cheat_length - row_diff.abs();
                        for col_diff in
                            -remaining_cheat_length..=remaining_cheat_length
                        {
                            let dest_col = col + col_diff as isize;
                            let cheat_moves = row_diff.abs() + col_diff.abs();

                            if cheat_moves < 2 {
                                continue;
                            }

                            let cheat_end = Position {
                                row: dest_row,
                                col: dest_col,
                            };
                            if let Some(&cheat_end_dist) =
                                end_distances.get(&cheat_end)
                            {
                                result.push(Cheat {
                                    saved: cheat_start_dist
                                        - cheat_end_dist
                                        - cheat_moves,
                                    start: cheat_start,
                                    end: cheat_end,
                                });
                            }
                        }
                    }
                }
            }
        }
        result
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let maze = Maze::new(reader)?;
    let mut cheats = maze.compute_cheats(match part {
        Part::Part1 => 2,
        Part::Part2 => 20,
    });

    let mut result = 0usize;
    while matches!(cheats.pop(), Some(cheat) if cheat.saved >= 100) {
        result += 1;
    }

    println!("{result}");

    Ok(())
}
