mod day01;

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
        _ => Err(invalid_input("Invalid day"))?,
    };

    run_func(part, reader)
}
