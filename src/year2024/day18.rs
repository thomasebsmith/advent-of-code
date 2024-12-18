use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::parse::parse_all;
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
    Safe,
    Corrupted,
}

struct Memory {
    layout: Vec<Vec<Cell>>,
    width: isize,
    height: isize,
    start: Position,
    end: Position,
    time: usize,
    corruptions: Vec<Position>,
}

impl Memory {
    fn new<R: io::Read>(reader: io::BufReader<R>) -> io::Result<Self> {
        let width = 71isize;
        let height = 71isize;
        let layout = vec![vec![Cell::Safe; width as usize]; height as usize];
        let start = Position { row: 0, col: 0 };
        let end = Position { row: 70, col: 70 };
        let time = 0usize;

        let mut corruptions = Vec::<Position>::new();
        for line in reader.lines() {
            let line = line?;
            let &[col, row] = &parse_all::<_, isize>(line.split(","))?[..]
            else {
                return Err(invalid_input("Expected x,y"));
            };
            corruptions.push(Position { row, col });
        }

        Ok(Self {
            layout,
            width,
            height,
            start,
            end,
            time,
            corruptions,
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

    fn at_mut(&mut self, position: Position) -> Option<&mut Cell> {
        if !self.in_bounds(position) {
            None
        } else {
            Some(&mut self.layout[position.row as usize][position.col as usize])
        }
    }

    #[allow(dead_code)]
    fn print(&self) {
        for row_layout in self.layout.iter() {
            for &cell in row_layout.iter() {
                print!(
                    "{}",
                    match cell {
                        Cell::Safe => '.',
                        Cell::Corrupted => '#',
                    }
                );
            }
            println!();
        }
    }

    fn simulate(&mut self, nanos: usize) {
        let start_time = self.time;
        let end_time = self.time + nanos;
        for time in start_time..end_time {
            if time < self.corruptions.len() {
                if let Some(cell) = self.at_mut(self.corruptions[time]) {
                    *cell = Cell::Corrupted;
                }
            } else {
                break;
            }
        }
        self.time = end_time;
    }

    fn shortest_path(&self) -> usize {
        #[derive(PartialEq, Eq, PartialOrd, Ord)]
        struct ToVisit {
            score: usize,
            position: Position,
        }

        let mut unvisited = BinaryHeap::<Reverse<ToVisit>>::new();
        let mut min_scores = HashMap::<Position, usize>::new();
        unvisited.push(Reverse(ToVisit {
            score: 0,
            position: self.start,
        }));
        min_scores.insert(self.start, 0);

        let mut value_to_return = usize::MAX;

        while let Some(Reverse(to_visit)) = unvisited.pop() {
            if to_visit.score > value_to_return {
                break;
            }

            if let Some(&other_score) = min_scores.get(&to_visit.position) {
                if other_score < to_visit.score {
                    continue;
                }
            }

            if to_visit.position == self.end {
                value_to_return = to_visit.score;
                continue;
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
                if cell == Cell::Corrupted {
                    continue;
                }

                let existing_min_score =
                    min_scores.entry(position).or_insert(usize::MAX);
                let score = neighbor_to_visit.score;
                if score < *existing_min_score {
                    *existing_min_score = score;
                    unvisited.push(Reverse(neighbor_to_visit));
                }
            }
        }

        value_to_return
    }

    fn exit_is_reachable(&self) -> bool {
        let mut visited = HashSet::<Position>::new();
        let mut to_visit = VecDeque::<Position>::new();
        to_visit.push_back(self.start);

        while let Some(next) = to_visit.pop_front() {
            if next == self.end {
                return true;
            }

            if self.at(next) != Some(Cell::Safe) {
                continue;
            }

            let neighbors = Direction::ALL
                .into_iter()
                .map(|direction| next.move_one(direction));

            for neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    to_visit.push_back(neighbor);
                }
            }
        }

        false
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut memory = Memory::new(reader)?;

    match part {
        Part::Part1 => {
            memory.simulate(1024);
            let result = memory.shortest_path();
            println!("{result}");
        }
        Part::Part2 => {
            while memory.exit_is_reachable() {
                memory.simulate(1);
            }
            if memory.time >= memory.corruptions.len() || memory.time == 0 {
                println!("<NONE>");
            } else {
                let last_corruption = memory.corruptions[memory.time - 1];
                println!("{},{}", last_corruption.col, last_corruption.row);
            }
        }
    }

    Ok(())
}
