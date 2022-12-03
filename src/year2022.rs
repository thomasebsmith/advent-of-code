mod day1;
mod day2;
mod day3;

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
        _ => Err(invalid_input("Invalid day"))?,
    };

    run_func(part, reader)
}
