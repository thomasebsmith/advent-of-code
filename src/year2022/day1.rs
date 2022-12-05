use std::collections::BTreeSet;
use std::io;
use std::io::BufRead;

use crate::part::Part;

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let num_top_elves: usize = match part {
        Part::Part1 => 1,
        Part::Part2 => 3,
    };

    let mut top_calories = BTreeSet::<u64>::new();
    let mut this_elfs_calories: u64 = 0;
    for line in reader.lines() {
        let line = line?;
        if line == "" {
            top_calories.insert(this_elfs_calories);
            if top_calories.len() > num_top_elves {
                top_calories.pop_first();
            }
            this_elfs_calories = 0;
        } else {
            this_elfs_calories += line
                .parse::<u64>()
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        }
    }

    let max_calories: u64 = top_calories.iter().sum();
    println!("{}", max_calories);

    Ok(())
}
