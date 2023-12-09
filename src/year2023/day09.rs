use std::io;

use crate::parse::{lines, parse_words};
use crate::part::Part;

struct History {
    values: Vec<i64>,
}

impl History {
    fn from_line(line: &str) -> io::Result<Self> {
        let values = parse_words::<i64>(line)?;
        Ok(Self { values })
    }

    fn derivative(&self) -> Self {
        let mut results = Vec::<i64>::new();

        if self.values.is_empty() {
            return Self { values: results };
        }

        let mut previous_value = self.values[0];
        for value in self.values.iter().skip(1) {
            results.push(*value - previous_value);
            previous_value = *value;
        }

        Self { values: results }
    }

    fn predict_one_forwards(&self) -> i64 {
        if self.values.iter().all(|value| *value == 0) {
            return 0;
        }

        assert!(!self.values.is_empty());
        self.values.last().unwrap() + self.derivative().predict_one_forwards()
    }

    fn predict_one_backwards(&self) -> i64 {
        if self.values.iter().all(|value| *value == 0) {
            return 0;
        }

        assert!(!self.values.is_empty());
        self.values.first().unwrap() - self.derivative().predict_one_backwards()
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let histories = lines(reader)?
        .map(|line| History::from_line(&line))
        .collect::<io::Result<Vec<_>>>()?;

    let result: i64 = histories
        .into_iter()
        .map(|history| match part {
            Part::Part1 => history.predict_one_forwards(),
            Part::Part2 => history.predict_one_backwards(),
        })
        .sum();
    println!("{result}");

    Ok(())
}
