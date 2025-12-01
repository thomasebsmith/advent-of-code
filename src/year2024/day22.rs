use std::collections::{HashMap, VecDeque};
use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::part::Part;

struct PriceGenerator {
    secret: i64,
}

impl PriceGenerator {
    fn new(seed: i64) -> Self {
        Self { secret: seed }
    }

    fn next(&mut self) -> i64 {
        self.secret ^= self.secret * 64; // ^ (<< 6)
        self.secret %= 16777216; // lowest 24 bits
        self.secret ^= self.secret / 32; // ^ (>> 5)
        self.secret %= 16777216; // lowest 24 bits
        self.secret ^= self.secret * 2048; // ^ (<< 11)
        self.secret %= 16777216; // lowest 24 bits
        self.secret
    }
}

const NUM_PRICES: usize = 2000;

fn part1(generators: Vec<PriceGenerator>) -> i64 {
    let mut result = 0i64;

    for mut generator in generators {
        for _ in 0..(NUM_PRICES - 1) {
            generator.next();
        }

        result += generator.next();
    }

    result
}

fn part2(mut generators: Vec<PriceGenerator>) -> i64 {
    // Brute force (kind of). Just store the outcomes for all diff sequences
    // and find the best one at the end.
    let mut diff_group_to_prices = HashMap::<[i64; 4], Vec<Option<i64>>>::new();
    let num_generators = generators.len();
    let mut add_diff = |generator_index: usize, key: [i64; 4], price: i64| {
        let entry = diff_group_to_prices
            .entry(key)
            .or_insert(vec![None; num_generators]);
        if entry[generator_index].is_none() {
            entry[generator_index] = Some(price);
        }
    };

    for (generator_index, generator) in generators.iter_mut().enumerate() {
        let mut last_value = generator.secret % 10;
        let mut trailing_diffs = VecDeque::<i64>::new();
        for _ in 0..NUM_PRICES {
            let this_value = generator.next() % 10;
            let diff = this_value - last_value;
            trailing_diffs.push_back(diff);

            if trailing_diffs.len() > 4 {
                trailing_diffs.pop_front();
            }
            if trailing_diffs.len() == 4 {
                let key = [
                    trailing_diffs[0],
                    trailing_diffs[1],
                    trailing_diffs[2],
                    trailing_diffs[3],
                ];
                add_diff(generator_index, key, this_value);
            }

            last_value = this_value;
        }
    }

    let max_bananas = diff_group_to_prices
        .values()
        .map(|prices| prices.iter().map(|price| price.unwrap_or(0)).sum())
        .max()
        .unwrap();

    max_bananas
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let generators = reader
        .lines()
        .map(|seed_str| {
            let seed = seed_str?.parse::<i64>().map_err(invalid_input)?;
            Ok(PriceGenerator::new(seed))
        })
        .collect::<io::Result<Vec<_>>>()?;

    let result = match part {
        Part::Part1 => part1(generators),
        Part::Part2 => part2(generators),
    };

    println!("{result}");

    Ok(())
}
