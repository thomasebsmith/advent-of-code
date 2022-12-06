mod day1;
mod day2;
mod day3;
mod day4;
mod day5;
mod day6;

use std::io;

use crate::errors::invalid_input;
use crate::part::Part;

pub fn run<R: io::Read>(
    day: u8,
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let run_func = match day {
        1 => day1::run,
        2 => day2::run,
        3 => day3::run,
        4 => day4::run,
        5 => day5::run,
        6 => day6::run,
        _ => Err(invalid_input("Invalid day"))?,
    };

    run_func(part, reader)
}
