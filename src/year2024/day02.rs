use std::collections::HashMap;
use std::io;
use std::io::BufRead;
use std::str::FromStr;

use crate::errors::invalid_input;
use crate::parse::{lines, parse_all, parse_words};
use crate::part::Part;

fn safe_count(reports: Vec<Report>, part: Part) -> usize {
    reports
        .into_iter()
        .filter(match part {
            Part::Part1 => Report::is_safe,
            Part::Part2 => Report::is_safe_tolerant,
        })
        .count()
}

struct Report {
    levels: Vec<i64>,
}

impl FromStr for Report {
    type Err = io::Error;

    fn from_str(s: &str) -> io::Result<Self> {
        Ok(Self {
            levels: parse_words(s)?,
        })
    }
}

fn is_safe(levels: &Vec<i64>, skip_index: Option<usize>) -> bool {
    let mut maybe_previous: Option<i64> = None;
    let mut maybe_increasing: Option<bool> = None;
    for (i, level) in levels.iter().enumerate() {
        if Some(i) == skip_index {
            continue;
        }
        if let Some(previous) = maybe_previous {
            if *level == previous {
                return false;
            }
            let now_increasing = *level > previous;
            if let Some(increasing) = maybe_increasing {
                if now_increasing != increasing {
                    return false;
                }
            } else {
                maybe_increasing = Some(now_increasing);
            }
            let diff = (*level - previous).abs();
            if diff > 3 {
                return false;
            }
        }
        maybe_previous = Some(*level);
    }
    return true;
}

impl Report {
    fn is_safe(&self) -> bool {
        is_safe(&self.levels, None)
    }

    fn is_safe_tolerant(&self) -> bool {
        if self.is_safe() {
            return true;
        }
        // O(n^2), unfortunately. DP would be nice.
        for i in (0..self.levels.len()).rev() {
            if is_safe(&self.levels, Some(i)) {
                return true;
            }
        }
        return false;
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let reports: Vec<Report> = parse_all(lines(reader)?)?;

    let result = safe_count(reports, part);
    println!("{result}");

    Ok(())
}
