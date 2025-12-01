use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::part::Part;

const DIAL_LIMIT: i64 = 100;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Dial {
    position: i64,
}

impl Dial {
    fn new() -> Self {
        Self { position: 50 }
    }

    fn turn(self, diff: i64) -> Self {
        let new_position = (self.position + diff).rem_euclid(DIAL_LIMIT);
        Self {
            position: new_position,
        }
    }

    fn turn_and_count_zeros(self, diff: i64) -> (Self, i64) {
        let new_dial = self.turn(diff);
        let mut zeroes = (self.position + diff).abs() / DIAL_LIMIT;
        if self.position + diff <= 0 && self.position > 0 {
            zeroes += 1;
        }

        (new_dial, zeroes)
    }

    fn is_zero(self) -> bool {
        self.position == 0
    }
}

fn part1(moves: &Vec<i64>) -> i64 {
    moves
        .iter()
        .fold((Dial::new(), 0), |(dial, password), amount| {
            let new_dial = dial.turn(*amount);
            (
                new_dial,
                password + (if new_dial.is_zero() { 1 } else { 0 }),
            )
        })
        .1
}

fn part2(moves: &Vec<i64>) -> i64 {
    moves
        .iter()
        .fold((Dial::new(), 0), |(dial, password), amount| {
            let (new_dial, zeroes) = dial.turn_and_count_zeros(*amount);
            (new_dial, password + zeroes)
        })
        .1
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut moves = Vec::<i64>::new();
    for line in reader.lines() {
        let line = &line?;
        if line.len() == 0 {
            return Err(invalid_input("Lines must not be empty"));
        }
        let mut amount = (&line[1..]).parse::<i64>().map_err(invalid_input)?;
        if line.starts_with('L') {
            amount = -amount;
        } else if !line.starts_with('R') {
            return Err(invalid_input("Lines must start with L or R"));
        }

        moves.push(amount);
    }

    let result = match part {
        Part::Part1 => part1(&moves),
        Part::Part2 => part2(&moves),
    };
    println!("{result}");

    Ok(())
}
