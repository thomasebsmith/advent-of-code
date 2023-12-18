use std::cmp::{max, min};
use std::collections::{HashSet, VecDeque};
use std::io;
use std::ops::Range;

use crate::errors::invalid_input;
use crate::parse::lines;
use crate::part::Part;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn from_char(ch: char) -> io::Result<Self> {
        match ch {
            'U' => Ok(Self::Up),
            'D' => Ok(Self::Down),
            'L' => Ok(Self::Left),
            'R' => Ok(Self::Right),
            _ => Err(invalid_input("invalid direction char")),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
struct Position {
    row: isize,
    col: isize,
}

impl Position {
    fn moved(self, direction: Direction, count: usize) -> Self {
        let count = count as isize;
        match direction {
            Direction::Left => Self {
                row: self.row,
                col: self.col - count,
            },
            Direction::Right => Self {
                row: self.row,
                col: self.col + count,
            },
            Direction::Up => Self {
                row: self.row - count,
                col: self.col,
            },
            Direction::Down => Self {
                row: self.row + count,
                col: self.col,
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
struct Step {
    direction: Direction,
    count: usize,
}

impl Step {
    fn from_line(line: &str, part: Part) -> io::Result<Self> {
        let [direction_str, count_str, color_str] =
            line.split_whitespace().collect::<Vec<_>>()[..]
        else {
            return Err(invalid_input("Expected 3 words"));
        };

        let (direction, count) = match part {
            Part::Part1 => {
                if direction_str.len() != 1 {
                    return Err(invalid_input("Invalid direction"));
                }

                let direction = Direction::from_char(
                    direction_str.chars().next().unwrap(),
                )?;

                let count =
                    count_str.parse::<usize>().map_err(invalid_input)?;

                (direction, count)
            }
            Part::Part2 => {
                let Some(color_str) = color_str.strip_prefix("(#") else {
                    return Err(invalid_input("Expected (# color prefix"));
                };
                let Some(color_str) = color_str.strip_suffix(")") else {
                    return Err(invalid_input("Expected ) color suffix"));
                };
                if color_str.len() != 6 {
                    return Err(invalid_input("Expected 6-digit hex color"));
                }

                let count_str = &color_str[..5];
                let count = usize::from_str_radix(count_str, 16)
                    .map_err(invalid_input)?;

                let direction = match color_str[5..].chars().next().unwrap() {
                    '0' => Ok(Direction::Right),
                    '1' => Ok(Direction::Down),
                    '2' => Ok(Direction::Left),
                    '3' => Ok(Direction::Up),
                    _ => Err(invalid_input("Invalid direction char")),
                }?;

                (direction, count)
            }
        };

        Ok(Self { direction, count })
    }
}

struct DigPlan {
    steps: Vec<Step>,
}

impl DigPlan {
    fn from_lines<I: Iterator<Item = String>>(
        lines: I,
        part: Part,
    ) -> io::Result<Self> {
        Ok(Self {
            steps: lines
                .map(|line| Step::from_line(line.as_ref(), part))
                .collect::<io::Result<Vec<_>>>()?,
        })
    }
}

struct DigGrid {
    position: Position,
    dug: VecDeque<VecDeque<bool>>,
    row_mapping: VecDeque<Range<isize>>,
    col_mapping: VecDeque<Range<isize>>,
}

impl DigGrid {
    fn new() -> Self {
        let start_position = Position { row: 0, col: 0 };

        let dug = VecDeque::<VecDeque<bool>>::from([
            VecDeque::<bool>::from([false, false, false]),
            VecDeque::<bool>::from([false, true, false]),
            VecDeque::<bool>::from([false, false, false]),
        ]);
        let row_mapping = VecDeque::<Range<isize>>::from([
            isize::MIN..0,
            0..1,
            1..isize::MAX,
        ]);
        let col_mapping = VecDeque::<Range<isize>>::from([
            isize::MIN..0,
            0..1,
            1..isize::MAX,
        ]);
        Self {
            position: start_position,
            dug,
            row_mapping,
            col_mapping,
        }
    }

    fn find_rows_cols_vertical(
        &mut self,
        start_row: isize,
        end_row: isize,
    ) -> Range<usize> {
        assert!(end_row >= start_row);

        let start_row_index = match self
            .row_mapping
            .binary_search_by_key(&start_row, |range| range.start)
        {
            Ok(index) => index,
            Err(insertion_index) => {
                assert!(insertion_index != 0);
                let old_end = self.row_mapping[insertion_index - 1].end;
                self.row_mapping[insertion_index - 1].end = start_row;
                self.row_mapping.insert(insertion_index, start_row..old_end);
                self.dug.insert(
                    insertion_index,
                    self.dug[insertion_index - 1].clone(),
                );
                insertion_index
            }
        };

        let end_row_index = match self
            .row_mapping
            .binary_search_by_key(&(end_row + 1), |range| range.end)
        {
            Ok(index) => index,
            Err(insertion_index) => {
                assert!(insertion_index != self.row_mapping.len());
                let old_start = self.row_mapping[insertion_index].start;
                self.row_mapping[insertion_index].start = end_row + 1;
                self.row_mapping
                    .insert(insertion_index, old_start..(end_row + 1));
                self.dug
                    .insert(insertion_index, self.dug[insertion_index].clone());
                insertion_index
            }
        };

        assert!(start_row_index <= end_row_index);
        start_row_index..(end_row_index + 1)
    }

    fn find_rows_cols_horizontal(
        &mut self,
        start_col: isize,
        end_col: isize,
    ) -> Range<usize> {
        assert!(end_col >= start_col);

        let start_col_index = match self
            .col_mapping
            .binary_search_by_key(&start_col, |range| range.start)
        {
            Ok(index) => index,
            Err(insertion_index) => {
                assert!(insertion_index != 0);
                let old_end = self.col_mapping[insertion_index - 1].end;
                self.col_mapping[insertion_index - 1].end = start_col;
                self.col_mapping.insert(insertion_index, start_col..old_end);
                for row in &mut self.dug {
                    row.insert(insertion_index, row[insertion_index - 1]);
                }
                insertion_index
            }
        };

        let end_col_index = match self
            .col_mapping
            .binary_search_by_key(&(end_col + 1), |range| range.end)
        {
            Ok(index) => index,
            Err(insertion_index) => {
                assert!(insertion_index != self.col_mapping.len());
                let old_start = self.col_mapping[insertion_index].start;
                self.col_mapping[insertion_index].start = end_col + 1;
                self.col_mapping
                    .insert(insertion_index, old_start..(end_col + 1));
                for row in &mut self.dug {
                    row.insert(insertion_index, row[insertion_index]);
                }
                insertion_index
            }
        };

        assert!(start_col_index <= end_col_index);
        start_col_index..(end_col_index + 1)
    }

    fn find_rows_cols(
        &mut self,
        position: Position,
        direction: Direction,
        count: usize,
    ) -> (Range<usize>, Range<usize>) {
        match direction {
            Direction::Up | Direction::Down => {
                let dest_position = position.moved(direction, count);
                let min_row = min(position.row, dest_position.row);
                let max_row = max(position.row, dest_position.row);
                let row_range = self.find_rows_cols_vertical(min_row, max_row);
                let col_range =
                    self.find_rows_cols_horizontal(position.col, position.col);
                (row_range, col_range)
            }
            Direction::Left | Direction::Right => {
                let dest_position = position.moved(direction, count);
                let min_col = min(position.col, dest_position.col);
                let max_col = max(position.col, dest_position.col);
                let col_range =
                    self.find_rows_cols_horizontal(min_col, max_col);
                let row_range =
                    self.find_rows_cols_vertical(position.row, position.row);
                (row_range, col_range)
            }
        }
    }

    fn follow_plan(&mut self, plan: &DigPlan) {
        for step in &plan.steps {
            let (rows, cols) =
                self.find_rows_cols(self.position, step.direction, step.count);
            for row in rows {
                for col in cols.clone() {
                    self.dug[row][col] = true;
                }
            }
            self.position = self.position.moved(step.direction, step.count);
        }
    }

    fn dig_out_interior(&mut self) {
        let mut reachable = HashSet::<(usize, usize)>::new();
        let mut reachable_queue = VecDeque::<(usize, usize)>::new();
        for row in 0..self.row_mapping.len() {
            reachable_queue.push_back((row, 0));
            reachable_queue.push_back((row, self.col_mapping.len() - 1));
        }
        for col in 0..self.col_mapping.len() {
            reachable_queue.push_back((0, col));
            reachable_queue.push_back((self.row_mapping.len() - 1, col));
        }

        while let Some(to_check) = reachable_queue.pop_front() {
            if reachable.contains(&to_check) {
                continue;
            }
            let (row, col) = to_check;
            if self.dug[row][col] {
                continue;
            }

            reachable.insert(to_check);
            if row != 0 {
                reachable_queue.push_back((row - 1, col));
            }
            if row != self.row_mapping.len() - 1 {
                reachable_queue.push_back((row + 1, col));
            }
            if col != 0 {
                reachable_queue.push_back((row, col - 1));
            }
            if col != self.col_mapping.len() - 1 {
                reachable_queue.push_back((row, col + 1));
            }
        }

        for row in 0..self.row_mapping.len() {
            for col in 0..self.col_mapping.len() {
                if !reachable.contains(&(row, col)) {
                    self.dug[row][col] = true;
                }
            }
        }
    }

    fn holdable_lava(&self) -> isize {
        let mut total: isize = 0;
        for row in 0..self.row_mapping.len() {
            for col in 0..self.col_mapping.len() {
                if self.dug[row][col] {
                    let row_range = &self.row_mapping[row];
                    let col_range = &self.col_mapping[col];
                    total += (row_range.end - row_range.start)
                        * (col_range.end - col_range.start);
                }
            }
        }
        total
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let plan = DigPlan::from_lines(lines(reader)?, part)?;
    let mut grid = DigGrid::new();

    grid.follow_plan(&plan);
    grid.dig_out_interior();

    let result = grid.holdable_lava();

    println!("{result}");

    Ok(())
}
