use std::io;
use std::io::BufRead;
use std::ops::Range;

use crate::errors::invalid_input;
use crate::iter::n_elements;
use crate::part::Part;

struct Assignment {
    sections: Range<u64>,
}

impl Assignment {
    fn new(assignment_str: &str) -> Option<Self> {
        let limits = n_elements(
            2,
            assignment_str.split('-').map(str::parse::<u64>)
        )?;
        let begin = *limits[0].as_ref().ok()?;
        let end = *limits[1].as_ref().ok()? + 1;
        Some(Self { sections: begin..end })
    }

    fn fully_contains(&self, other: &Assignment) -> bool {
        self.sections.start <= other.sections.start &&
            self.sections.end >= other.sections.end
    }

    fn begins_in(&self, other: &Assignment) -> bool {
        self.sections.start >= other.sections.start &&
            self.sections.start < other.sections.end
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut count: u64 = 0;
    for line in reader.lines() {
        let assignments = n_elements(
            2,
            line?.split(',').map(Assignment::new)
        ).ok_or_else(|| invalid_input("number of assignments is not 2"))?;

        let first_assignment = assignments[0].as_ref().ok_or_else(
            || invalid_input("invalid first assignment")
        )?;
        let second_assignment = assignments[1].as_ref().ok_or_else(
            || invalid_input("invalid second assignment")
        )?;

        let check_func = match part {
            Part::Part1 => Assignment::fully_contains,
            Part::Part2 => Assignment::begins_in,
        };

        if check_func(first_assignment, second_assignment) ||
            check_func(second_assignment, first_assignment) {
            count += 1;
        }
    }

    println!("{}", count);
    Ok(())
}
