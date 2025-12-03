mod day01;
mod day02;
mod day03;

use std::io;

use crate::errors::invalid_input;
use crate::part::Part;

pub fn run<R: io::Read>(
    day: u8,
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let run_func = match day {
        1 => day01::run,
        2 => day02::run,
        3 => day03::run,
        _ => Err(invalid_input("Invalid day"))?,
    };

    run_func(part, reader)
}
