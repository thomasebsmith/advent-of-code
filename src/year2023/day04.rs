use std::collections::HashSet;
use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::part::Part;

struct Scratchcard {
    winning_nums: HashSet<i64>,
    card_nums: Vec<i64>,
}

impl Scratchcard {
    fn from_line(line: &str) -> io::Result<Self> {
        let [_id_str, nums_str] = line.split(": ").collect::<Vec<_>>()[..]
        else {
            return Err(invalid_input("No \": \""));
        };

        let [winning_nums_str, card_nums_str] =
            &nums_str.split(" | ").collect::<Vec<_>>()[..]
        else {
            return Err(invalid_input("No \" | \""));
        };

        let winning_nums = winning_nums_str
            .split_whitespace()
            .map(|s| s.parse::<i64>().map_err(invalid_input))
            .collect::<io::Result<_>>()?;
        let card_nums = card_nums_str
            .split_whitespace()
            .map(|s| s.parse::<i64>().map_err(invalid_input))
            .collect::<io::Result<_>>()?;

        Ok(Self {
            winning_nums,
            card_nums,
        })
    }

    fn num_matches(&self) -> usize {
        self.card_nums
            .iter()
            .filter(|num| self.winning_nums.contains(num))
            .count()
    }

    fn points(&self) -> i64 {
        match self.num_matches() {
            0 => 0,
            x => 1 << (x - 1),
        }
    }
}

fn part1<R: io::Read>(reader: io::BufReader<R>) -> io::Result<()> {
    let mut total_points: i64 = 0;

    for line in reader.lines() {
        let line = line?;
        if line == "" {
            continue;
        }

        let card = Scratchcard::from_line(&line)?;

        total_points += card.points();
    }

    println!("{total_points}");

    Ok(())
}

fn part2<R: io::Read>(reader: io::BufReader<R>) -> io::Result<()> {
    let mut total_num_cards: i64 = 0;

    let mut card_counts = Vec::<i64>::new();

    for (card_index, line) in reader.lines().enumerate() {
        let line = line?;
        if line == "" {
            continue;
        }

        let card = Scratchcard::from_line(&line)?;

        if card_index >= card_counts.len() {
            card_counts.push(1);
            assert!(card_index == card_counts.len() - 1);
        }

        let count_of_this_card = card_counts[card_index];
        total_num_cards += count_of_this_card;

        let num_matches = card.num_matches();

        for i in card_index + 1..card_index + 1 + num_matches {
            if i >= card_counts.len() {
                card_counts.push(1);
                assert!(i == card_counts.len() - 1);
            }

            card_counts[i] += count_of_this_card;
        }
    }

    println!("{total_num_cards}");

    Ok(())
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    (match part {
        Part::Part1 => part1,
        Part::Part2 => part2,
    })(reader)
}
