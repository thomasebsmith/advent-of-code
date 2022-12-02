mod day1;
mod day2;

use std::io;

use crate::part::Part;

pub fn run(day: u8, part: Part, input_file: &str) -> io::Result<()> {
    let run_func = match day {
        1 => day1::run,
        2 => day2::run,
        _ => Err(io::Error::new(io::ErrorKind::Other, "Invalid day"))?,
    };
    run_func(part, input_file)
}
