mod day1;
mod day10;
mod day11;
mod day12;
mod day2;
mod day3;
mod day4;
mod day5;
mod day6;
mod day7;
mod day8;
mod day9;

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
        7 => day7::run,
        8 => day8::run,
        9 => day9::run,
        10 => day10::run,
        11 => day11::run,
        12 => day12::run,
        _ => Err(invalid_input("Invalid day"))?,
    };

    run_func(part, reader)
}
