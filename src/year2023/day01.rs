use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::part::Part;

struct DigitPattern {
    pattern: &'static str,
    digit: u64,
}

impl DigitPattern {
    const fn new(pattern: &'static str, digit: u64) -> Self {
        Self { pattern, digit }
    }
}

const NUMERIC_PATTERNS: [DigitPattern; 10] = [
    DigitPattern::new("0", 0),
    DigitPattern::new("1", 1),
    DigitPattern::new("2", 2),
    DigitPattern::new("3", 3),
    DigitPattern::new("4", 4),
    DigitPattern::new("5", 5),
    DigitPattern::new("6", 6),
    DigitPattern::new("7", 7),
    DigitPattern::new("8", 8),
    DigitPattern::new("9", 9),
];

const WORD_PATTERNS: [DigitPattern; 9] = [
    DigitPattern::new("one", 1),
    DigitPattern::new("two", 2),
    DigitPattern::new("three", 3),
    DigitPattern::new("four", 4),
    DigitPattern::new("five", 5),
    DigitPattern::new("six", 6),
    DigitPattern::new("seven", 7),
    DigitPattern::new("eight", 8),
    DigitPattern::new("nine", 9),
];

fn find_calibration_value<'a, I>(line: &str, patterns: I) -> Option<u64>
where
    I: Iterator<Item = &'a DigitPattern>,
{
    // Not the most efficient, but sufficient

    let mut first_digit: Option<u64> = None;
    let mut first_digit_index: usize = 0;
    let mut last_digit: Option<u64> = None;
    let mut last_digit_index: usize = 0;

    for pattern in patterns {
        if let Some(first_index) = line.find(pattern.pattern) {
            if first_digit.is_none() || first_index < first_digit_index {
                first_digit_index = first_index;
                first_digit = Some(pattern.digit);
            }
        }

        if let Some(last_index) = line.rfind(pattern.pattern) {
            if last_digit.is_none() || last_index > last_digit_index {
                last_digit_index = last_index;
                last_digit = Some(pattern.digit);
            }
        }
    }

    if let (Some(first_digit), Some(last_digit)) = (first_digit, last_digit) {
        Some(first_digit * 10 + last_digit)
    } else {
        None
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut sum: u64 = 0;

    for line in reader.lines() {
        let line = line?;
        if line == "" {
            continue;
        }

        let calibration_value = match part {
            Part::Part1 => {
                find_calibration_value(&line, NUMERIC_PATTERNS.iter())
            }
            Part::Part2 => find_calibration_value(
                &line,
                NUMERIC_PATTERNS.iter().chain(WORD_PATTERNS.iter()),
            ),
        };

        let Some(calibration_value) = calibration_value else {
            return Err(invalid_input("Could not find calibration value"));
        };

        sum += calibration_value;
    }

    println!("{sum}");

    Ok(())
}
