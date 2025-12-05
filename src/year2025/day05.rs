use std::cmp::max;
use std::io;
use std::ops::Range;

use crate::errors::invalid_input;
use crate::parse::{lines, paragraphs, parse_all};
use crate::part::Part;

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug)]
struct Ingredient(i64);

struct FreshIngredients {
    ranges: Vec<Range<Ingredient>>,
}

impl FreshIngredients {
    fn new(mut ranges: Vec<Range<Ingredient>>) -> Self {
        ranges.sort_by_key(|range| range.start);
        let mut merged_ranges =
            Vec::<Range<Ingredient>>::with_capacity(ranges.len());
        for range in ranges {
            if let Some(last_range) = merged_ranges.last() {
                let last_range = last_range.clone();

                if last_range.end >= range.start {
                    // The ranges overlap or are immediately next to each other
                    let new_start = last_range.start;
                    let new_end = max(last_range.end, range.end);
                    *merged_ranges.last_mut().unwrap() = new_start..new_end;
                    continue;
                }
            }

            merged_ranges.push(range);
        }

        Self {
            ranges: merged_ranges,
        }
    }

    fn compute_size(&self) -> usize {
        self.ranges
            .iter()
            .map(|range| (range.end.0 - range.start.0) as usize)
            .sum()
    }
}

fn part1(fresh: FreshIngredients, mut available: Vec<Ingredient>) -> usize {
    available.sort();

    let mut result = 0usize;
    let mut fresh_index = 0usize;
    let mut available_index = 0usize;

    while fresh_index < fresh.ranges.len() && available_index < available.len()
    {
        let cur_range = &fresh.ranges[fresh_index];
        let cur_available = available[available_index];

        if cur_range.contains(&cur_available) {
            // println!("{cur_available:?} is fresh");
            result += 1;
            available_index += 1;
            continue;
        }

        // println!("failure on {cur_available:?} in {cur_range:?}");

        if cur_available < cur_range.start {
            available_index += 1;
            continue;
        }

        // cur_available >= cur_range.end
        fresh_index += 1;
    }

    result
}

fn part2(fresh: FreshIngredients) -> usize {
    fresh.compute_size()
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let [fresh_lines, available_lines] =
        &paragraphs(lines(reader)?).collect::<Vec<_>>()[..]
    else {
        return Err(invalid_input("Expected two sections"));
    };

    let fresh_ranges = fresh_lines
        .into_iter()
        .map(|line| {
            let [start, end_inclusive] =
                parse_all::<_, i64>(line.split('-'))?[..]
            else {
                return Err(invalid_input(
                    "Expected a range of the form start-end",
                ));
            };

            if end_inclusive < start {
                return Err(invalid_input(
                    "Expected a range with start <= end",
                ));
            }

            Ok(Ingredient(start)..Ingredient(end_inclusive + 1))
        })
        .collect::<io::Result<Vec<_>>>()?;

    let fresh = FreshIngredients::new(fresh_ranges);

    let available = available_lines
        .into_iter()
        .map(|line| Ok(Ingredient(line.parse::<i64>().map_err(invalid_input)?)))
        .collect::<io::Result<Vec<_>>>()?;

    let result = match part {
        Part::Part1 => part1(fresh, available),
        Part::Part2 => part2(fresh),
    };
    println!("{result}");

    Ok(())
}
