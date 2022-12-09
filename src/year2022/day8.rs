use std::collections::BTreeMap;
use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::part::Part;

struct Forest {
    trees: Vec<Vec<u64>>,
    width: usize,
    height: usize,
}

// TODO: Clean up duplicated code

impl Forest {
    fn new<R: io::Read>(reader: io::BufReader<R>) -> io::Result<Forest> {
        let mut trees = Vec::<Vec<u64>>::new();
        let mut width: Option<usize> = None;
        let mut height: usize = 0;

        for line in reader.lines() {
            let line = line?;

            let mut tree_row = Vec::<u64>::new();
            for ch in line.chars() {
                if !ch.is_ascii_digit() {
                    Err(invalid_input("Non-digit character in grid"))?
                }
                tree_row.push(ch as u64 - '0' as u64);
            }
            match width {
                None => width = Some(tree_row.len()),
                Some(width) => {
                    if tree_row.len() != width {
                        Err(invalid_input("Different row widths in grid"))?;
                    }
                }
            }
            trees.push(tree_row);
            height += 1;
        }

        let Some(width) = width else {
            return Err(invalid_input("Empty grid"));
        };

        Ok(Forest {
            trees,
            width,
            height,
        })
    }

    fn scenic_score_grid(&self) -> Vec<Vec<u64>> {
        let mut grid = vec![vec![0 as u64; self.width]; self.height];

        let mut left_grid = vec![vec![0 as usize; self.width]; self.height];
        for row in 0..self.height {
            let mut map = BTreeMap::<u64, usize>::new();
            for col in 0..self.width {
                match map.range(self.trees[row][col]..).next() {
                    None => left_grid[row][col] = col,
                    Some((_, index)) => left_grid[row][col] = col - *index,
                }
                map = map.split_off(&self.trees[row][col]);
                map.insert(self.trees[row][col], col);
            }
        }

        let mut right_grid = vec![vec![0 as usize; self.width]; self.height];
        for row in 0..self.height {
            let mut map = BTreeMap::<u64, usize>::new();
            for col in (0..self.width).rev() {
                match map.range(self.trees[row][col]..).next() {
                    None => right_grid[row][col] = self.width - col - 1,
                    Some((_, index)) => right_grid[row][col] = *index - col,
                }
                map = map.split_off(&self.trees[row][col]);
                map.insert(self.trees[row][col], col);
            }
        }

        let mut top_grid = vec![vec![0 as usize; self.width]; self.height];
        for col in 0..self.width {
            let mut map = BTreeMap::<u64, usize>::new();
            for row in 0..self.height {
                match map.range(self.trees[row][col]..).next() {
                    None => top_grid[row][col] = row,
                    Some((_, index)) => top_grid[row][col] = row - *index,
                }
                map = map.split_off(&self.trees[row][col]);
                map.insert(self.trees[row][col], row);
            }
        }
        let mut bottom_grid = vec![vec![0 as usize; self.width]; self.height];
        for col in 0..self.width {
            let mut map = BTreeMap::<u64, usize>::new();
            for row in (0..self.height).rev() {
                match map.range(self.trees[row][col]..).next() {
                    None => bottom_grid[row][col] = self.height - row - 1,
                    Some((_, index)) => bottom_grid[row][col] = *index - row,
                }
                map = map.split_off(&self.trees[row][col]);
                map.insert(self.trees[row][col], row);
            }
        }

        for row in 0..self.width {
            for col in 0..self.height {
                let left_val = left_grid[row][col] as u64;
                let right_val = right_grid[row][col] as u64;
                let top_val = top_grid[row][col] as u64;
                let bottom_val = bottom_grid[row][col] as u64;
                grid[row][col] = left_val * right_val * top_val * bottom_val;
            }
        }

        grid
    }

    pub fn max_scenic_score(&self) -> Option<u64> {
        let grid = self.scenic_score_grid();
        grid.into_iter()
            .map(|row| row.into_iter().max())
            .max()
            .flatten()
    }

    fn visibility_grid(&self) -> Vec<Vec<bool>> {
        let mut grid = vec![vec![false; self.width]; self.height];

        for row in 0..self.height {
            let mut max_left_height: Option<u64> = None;
            for col in 0..self.width {
                let height = self.trees[row][col];
                match max_left_height {
                    None => {
                        grid[row][col] = true;
                        max_left_height = Some(self.trees[row][col]);
                    }
                    Some(blocking_height) if height > blocking_height => {
                        grid[row][col] = true;
                        max_left_height = Some(self.trees[row][col]);
                    }
                    _ => {}
                }
            }
        }

        for row in 0..self.height {
            let mut max_right_height: Option<u64> = None;
            for col in (0..self.width).rev() {
                let height = self.trees[row][col];
                match max_right_height {
                    None => {
                        grid[row][col] = true;
                        max_right_height = Some(self.trees[row][col]);
                    }
                    Some(blocking_height) if height > blocking_height => {
                        grid[row][col] = true;
                        max_right_height = Some(self.trees[row][col]);
                    }
                    _ => {}
                }
            }
        }

        for col in 0..self.width {
            let mut max_top_height: Option<u64> = None;
            for row in 0..self.height {
                let height = self.trees[row][col];
                match max_top_height {
                    None => {
                        grid[row][col] = true;
                        max_top_height = Some(self.trees[row][col]);
                    }
                    Some(blocking_height) if height > blocking_height => {
                        grid[row][col] = true;
                        max_top_height = Some(self.trees[row][col]);
                    }
                    _ => {}
                }
            }
        }

        for col in 0..self.width {
            let mut max_bottom_height: Option<u64> = None;
            for row in (0..self.height).rev() {
                let height = self.trees[row][col];
                match max_bottom_height {
                    None => {
                        grid[row][col] = true;
                        max_bottom_height = Some(self.trees[row][col]);
                    }
                    Some(blocking_height) if height > blocking_height => {
                        grid[row][col] = true;
                        max_bottom_height = Some(self.trees[row][col]);
                    }
                    _ => {}
                }
            }
        }

        grid
    }

    pub fn num_visible_trees(&self) -> usize {
        self.visibility_grid()
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|visible| visible as usize)
                    .sum::<usize>()
            })
            .sum()
    }
}
pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let forest = Forest::new(reader)?;
    match part {
        Part::Part1 => println!("{}", forest.num_visible_trees()),
        Part::Part2 => println!(
            "{}",
            forest
                .max_scenic_score()
                .ok_or_else(|| invalid_input("No scenic scores"))?
        ),
    }
    Ok(())
}
