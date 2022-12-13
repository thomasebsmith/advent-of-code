use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::iter::{all_unique, consecutive_sequences, only_element};
use crate::part::Part;

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let marker_len: usize = match part {
        Part::Part1 => 4,
        Part::Part2 => 14,
    };

    let datastream = only_element(reader.lines())
        .ok_or_else(|| invalid_input("More than one line"))??;

    if datastream.len() < marker_len {
        Err(invalid_input("No possible markers - too short"))?
    }

    for (i, subsequence) in
        consecutive_sequences(marker_len, datastream.chars()).enumerate()
    {
        if all_unique(subsequence.iter()) {
            println!("{}", i + marker_len);
            return Ok(());
        }
    }

    Err(invalid_input("No marker found"))
}
