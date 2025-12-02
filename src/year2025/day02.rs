use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::part::Part;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Range {
    start: i64, // inclusive
    end: i64,   // exclusive
}

fn digits(mut num: i64) -> Vec<u8> {
    let mut the_digits = Vec::<u8>::new();
    loop {
        the_digits.push((num % 10) as u8);
        num /= 10;
        if num == 0 {
            break;
        }
    }
    the_digits.reverse();
    the_digits
}

fn is_invalid_part1(id: i64) -> bool {
    let the_digits = digits(id);

    if the_digits.len() % 2 != 0 {
        return false;
    }

    let mut i: usize = 0;
    let half_len = the_digits.len() / 2;
    while i < half_len {
        if the_digits[i] != the_digits[i + half_len] {
            return false;
        }
        i += 1;
    }
    true
}

fn is_invalid_part2(id: i64) -> bool {
    let the_digits = digits(id);

    if the_digits.len() <= 1 {
        return false;
    }

    'outer: for rep_len in 1..=(the_digits.len() / 2) {
        if the_digits.len() % rep_len != 0 {
            continue;
        }
        let num_reps = the_digits.len() / rep_len;
        let mut i = 0usize;
        while i < rep_len {
            let expected_digit = the_digits[i];
            for rep_offset in 1..num_reps {
                if the_digits[rep_offset * rep_len + i] != expected_digit {
                    continue 'outer;
                }
            }
            i += 1;
        }
        return true;
    }
    false
}

impl Range {
    fn from_inclusive_bounds(start: i64, end_inclusive: i64) -> Self {
        Self {
            start,
            end: end_inclusive + 1,
        }
    }

    fn sum_of_invalid_ids(self, part: Part) -> i64 {
        (self.start..self.end)
            .into_iter()
            .filter(|id| {
                if part == Part::Part1 {
                    is_invalid_part1(*id)
                } else {
                    is_invalid_part2(*id)
                }
            })
            .sum()
    }
}

fn solve(ranges: Vec<Range>, part: Part) -> i64 {
    ranges
        .into_iter()
        .map(|range| range.sum_of_invalid_ids(part))
        .sum()
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let ranges = reader
        .split(b',')
        .map(|range_bytes| {
            let range_bytes = range_bytes?;
            let range_str =
                str::from_utf8(&range_bytes).map_err(invalid_input)?.trim();
            let &[start, end] = &range_str
                .split('-')
                .map(|num_str| num_str.parse::<i64>().map_err(invalid_input))
                .collect::<io::Result<Vec<_>>>()?[..]
            else {
                return Err(invalid_input("Expected one dash"));
            };
            Ok(Range::from_inclusive_bounds(start, end))
        })
        .collect::<io::Result<Vec<_>>>()?;

    let result = solve(ranges, part);
    println!("{result}");

    Ok(())
}
