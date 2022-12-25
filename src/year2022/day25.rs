use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::part::Part;

const BASE: i64 = 5;

fn parse_snafu(snafu: &str) -> Option<i64> {
    let mut place_val: i64 = 1;

    let mut result: i64 = 0;

    for ch in snafu.chars().rev() {
        result += place_val
            * (match ch {
                '2' => 2,
                '1' => 1,
                '0' => 0,
                '-' => -1,
                '=' => -2,
                _ => None?,
            });
        place_val *= BASE;
    }

    Some(result)
}

fn to_snafu(mut number: i64) -> String {
    let mut rev_digits = Vec::<char>::new();
    while number != 0 {
        let remainder = number % BASE;
        if remainder <= 2 {
            rev_digits.push(match remainder {
                0 => '0',
                1 => '1',
                2 => '2',
                _ => unreachable!(),
            });
        } else {
            rev_digits.push(match remainder {
                3 => '=',
                4 => '-',
                _ => unreachable!(),
            });
            number += BASE;
        }
        number /= BASE;
    }

    let mut result = String::new();
    for ch in rev_digits.into_iter().rev() {
        result.push(ch);
    }
    result
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    match part {
        Part::Part1 => {
            let mut sum: i64 = 0;
            for line in reader.lines() {
                let line = line?;
                let number = parse_snafu(&line).ok_or_else(|| {
                    invalid_input("Could not parse snafu number")
                })?;
                sum += number;
            }
            println!("{}", to_snafu(sum));
        }
        Part::Part2 => {
            println!("Merry Christmas!");
        }
    }

    Ok(())
}
