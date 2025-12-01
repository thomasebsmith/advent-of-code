use std::io;

use crate::errors::invalid_input;
use crate::parse::{lines, paragraphs};
use crate::part::Part;

#[derive(Clone, Debug)]
struct Lock {
    heights: Vec<usize>,
    vertical_space: usize,
}

#[derive(Clone, Debug)]
struct Key {
    heights: Vec<usize>,
}

impl Lock {
    fn can_fit(&self, key: &Key) -> bool {
        self.heights
            .iter()
            .zip(key.heights.iter())
            .all(|(h1, h2)| h1 + h2 <= self.vertical_space)
    }
}

pub fn run<R: io::Read>(
    _part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let all_paragraphs = paragraphs(lines(reader)?).collect::<Vec<_>>();

    let mut locks = Vec::<Lock>::new();
    let mut keys = Vec::<Key>::new();

    for paragraph in all_paragraphs {
        let mut is_lock = false;
        let mut heights = Vec::<usize>::new();
        let mut max_row_num: Option<usize> = None;
        for (row_num, line) in paragraph.into_iter().enumerate() {
            max_row_num = Some(row_num);
            for (col_num, ch) in line.chars().enumerate() {
                if row_num == 0 && col_num == 0 && ch == '#' {
                    is_lock = true;
                }

                while col_num >= heights.len() {
                    heights.push(0);
                }
                if ch == '#' {
                    heights[col_num] += 1;
                }
            }
        }

        let Some(max_row_num) = max_row_num else {
            return Err(invalid_input("No rows in key/lock"));
        };

        if is_lock {
            locks.push(Lock {
                heights,
                vertical_space: max_row_num + 1,
            });
        } else {
            keys.push(Key { heights });
        }
    }

    let mut result = 0usize;
    for lock in &locks {
        for key in &keys {
            if lock.can_fit(key) {
                result += 1;
            }
        }
    }

    println!("{result}");

    Ok(())
}
