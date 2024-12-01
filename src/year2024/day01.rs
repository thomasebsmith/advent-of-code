use std::collections::HashMap;
use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::part::Part;
use crate::parse::parse_words;

fn part1(left_list: Vec<i64>, right_list: Vec<i64>) -> i64 {
    left_list
        .iter()
        .zip(right_list)
        .map(|(left, right)| (left - right).abs())
        .sum()
}

fn part2(left_list: Vec<i64>, right_list: Vec<i64>) -> i64 {
    let mut frequencies = HashMap::<i64, i64>::new();
    for item in right_list {
        *frequencies.entry(item).or_insert(0) += 1;
    }
    left_list.iter()
        .map(|item| frequencies.get(item).unwrap_or(&0) * item)
        .sum()
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut left_list = Vec::<i64>::new();
    let mut right_list = Vec::<i64>::new();
    for line in reader.lines() {
        let [left, right] = parse_words::<i64>(&line?)?[..] else {
            return Err(invalid_input("Expected \"<left> <right>\""));
        };
        left_list.push(left);
        right_list.push(right);
    }
    left_list.sort();
    right_list.sort();

    let result = match part {
        Part::Part1 => part1(left_list, right_list),
        Part::Part2 => part2(left_list, right_list),
    };
    println!("{result}");

    Ok(())
}
