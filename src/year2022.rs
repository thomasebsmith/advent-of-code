mod day1;

use std::io;

use crate::part::Part;

pub fn run(day: u8, part: Part, input_file: &str) -> io::Result<()> {
    if day == 1 {
        day1::run(part, input_file)
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "Invalid day"))
    }
}
