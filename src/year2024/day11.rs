use std::collections::HashMap;
use std::io;

use crate::errors::invalid_input;
use crate::parse::{lines, parse_words};
use crate::part::Part;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    value: i64,
    blinks: usize,
}

struct Solver {
    cache: HashMap<Key, usize>,
}

impl Solver {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    fn calculate(&mut self, key: Key) -> usize {
        /*
            num_stones(value, blinks) =
                1 if blinks == 0
                sum(num_stones(value_out, blinks - 1) for value_out in compute(value))
        */
        if key.blinks == 0 {
            return 1;
        }

        let next_blinks = key.blinks - 1;

        if key.value == 0 {
            return self.solve_key(Key {
                value: 1,
                blinks: next_blinks,
            });
        }

        let num_digits = key.value.ilog10() + 1;
        if num_digits % 2 == 0 {
            let power_of_ten_mask = 10i64.pow(num_digits / 2);
            let left_stone = key.value / power_of_ten_mask;
            let right_stone = key.value - left_stone * power_of_ten_mask;
            return self.solve_key(Key {
                value: left_stone,
                blinks: next_blinks,
            }) + self.solve_key(Key {
                value: right_stone,
                blinks: next_blinks,
            });
        }

        self.solve_key(Key {
            value: key.value * 2024,
            blinks: next_blinks,
        })
    }

    fn solve_key(&mut self, key: Key) -> usize {
        if let Some(&solution) = self.cache.get(&key) {
            solution
        } else {
            let solution = self.calculate(key);
            self.cache.insert(key, solution);
            solution
        }
    }

    pub fn solve(&mut self, value: i64, blinks: usize) -> usize {
        self.solve_key(Key { value, blinks })
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let line_strings = lines(reader)?.collect::<Vec<_>>();
    if line_strings.len() != 1 {
        return Err(invalid_input("Expected one line"));
    };

    let mut result = 0usize;
    let mut solver = Solver::new();
    let num_blinks = match part {
        Part::Part1 => 25usize,
        Part::Part2 => 75usize,
    };
    for stone in parse_words::<i64>(&line_strings[0])? {
        result += solver.solve(stone, num_blinks);
    }
    println!("{result}");

    Ok(())
}
