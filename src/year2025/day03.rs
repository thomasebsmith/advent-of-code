use std::cmp::Reverse;
use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::part::Part;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Battery {
    voltage: i64,
}

impl Battery {
    fn new(voltage: i64) -> Self {
        Self { voltage }
    }
}

#[derive(Clone, Debug)]
struct BatteryBank {
    batteries: Vec<Battery>,
}

impl BatteryBank {
    fn from_string(string: &str) -> io::Result<Self> {
        let batteries = string
            .chars()
            .map(|ch| {
                Ok(Battery::new(
                    ch.to_digit(10)
                        .ok_or_else(|| invalid_input("Invalid digit"))?
                        as i64,
                ))
            })
            .collect::<io::Result<Vec<_>>>()?;
        Ok(Self { batteries })
    }

    fn largest_joltage(&self, num_batteries: usize) -> io::Result<i64> {
        if self.batteries.len() < num_batteries {
            return Err(invalid_input(format!(
                "Expected at least {num_batteries} batteries per bank"
            )));
        }
        let mut result = 0i64;
        let mut starting_index = 0usize;
        for battery_index in 0..num_batteries {
            let num_excluded_batteries = num_batteries - battery_index - 1;
            let ending_index = self.batteries.len() - num_excluded_batteries;
            let battery_range = &self.batteries[starting_index..ending_index];
            let (index_from_start, battery) = battery_range
                .iter()
                .enumerate()
                .max_by_key(|(i, battery)| (*battery, Reverse(*i)))
                .unwrap();
            let index = index_from_start + starting_index;
            starting_index = index + 1;
            result *= 10;
            result += battery.voltage;
        }
        Ok(result)
    }
}

fn solve(banks: Vec<BatteryBank>, part: Part) -> io::Result<i64> {
    let num_batteries = match part {
        Part::Part1 => 2,
        Part::Part2 => 12,
    };
    Ok(banks
        .iter()
        .map(|bank| bank.largest_joltage(num_batteries))
        .collect::<io::Result<Vec<_>>>()?
        .into_iter()
        .sum())
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let banks = reader
        .lines()
        .map(|line| BatteryBank::from_string(&line?))
        .collect::<io::Result<Vec<_>>>()?;

    let result = solve(banks, part)?;
    println!("{result}");

    Ok(())
}
