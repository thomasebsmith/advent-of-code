use std::io;
use std::str::FromStr;

use crate::errors::invalid_input;
use crate::parse::{lines_vec, parse_words};
use crate::part::Part;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum MathOp {
    Add,
    Multiply,
}

impl MathOp {
    fn identity(self) -> i64 {
        match self {
            Self::Add => 0,
            Self::Multiply => 1,
        }
    }

    fn compute(self, value1: i64, value2: i64) -> i64 {
        match self {
            Self::Add => value1 + value2,
            Self::Multiply => value1 * value2,
        }
    }
}

impl FromStr for MathOp {
    type Err = io::Error;

    fn from_str(string: &str) -> io::Result<Self> {
        match string {
            "+" => Ok(Self::Add),
            "*" => Ok(Self::Multiply),
            _ => Err(invalid_input(format!("Invalid math op '{string}'"))),
        }
    }
}

struct ProblemSheet {
    values: Vec<Vec<i64>>,
    operations: Vec<MathOp>,
}

impl ProblemSheet {
    fn new<R: io::Read>(reader: io::BufReader<R>) -> io::Result<Self> {
        let lines = lines_vec(reader)?;
        if lines.len() < 1 {
            return Err(invalid_input("Expected at least 1 line"));
        }
        let mut width: Option<usize> = None;
        let mut row_vectors = Vec::<Vec<i64>>::with_capacity(lines.len() - 1);
        for line in &lines[..lines.len() - 1] {
            let parsed_line = parse_words::<i64>(line)?;
            if let Some(the_width) = width {
                if parsed_line.len() != the_width {
                    return Err(invalid_input("Varying line widths"));
                }
            } else {
                width = Some(parsed_line.len());
            }
            row_vectors.push(parsed_line);
        }

        let operations = parse_words::<MathOp>(lines.last().unwrap())?;
        if let Some(the_width) = width
            && operations.len() != the_width
        {
            return Err(invalid_input("Varying line widths"));
        }

        let values = (0..operations.len())
            .map(|column_index| {
                row_vectors
                    .iter()
                    .map(|row_vector| row_vector[column_index])
                    .collect()
            })
            .collect::<Vec<Vec<i64>>>();

        Ok(Self { values, operations })
    }

    fn new_with_complex_parsing<R: io::Read>(
        reader: io::BufReader<R>,
    ) -> io::Result<Self> {
        let lines = lines_vec(reader)?;
        if lines.len() < 2 {
            return Err(invalid_input("Expected at least 2 lines"));
        }

        let op_line = lines.last().unwrap();

        let mut values = Vec::<Vec<i64>>::new();
        let mut cur_value_vec = Vec::<i64>::new();
        let mut operations = Vec::<MathOp>::new();

        let mut column_index = 0usize;
        let mut operation_found = false;
        loop {
            let mut any_char_found = false;
            let mut any_digit_found = false;
            let mut result = 0i64;
            for line in &lines[..lines.len() - 1] {
                let Some(ch) = line.chars().nth(column_index) else {
                    continue;
                };

                any_char_found = true;

                if ch == ' ' {
                    continue;
                }

                let Some(digit) = ch.to_digit(10) else {
                    return Err(invalid_input("Invalid digit"));
                };
                any_digit_found = true;
                result *= 10;
                result += digit as i64;
            }

            if let Some(op_char) = op_line.chars().nth(column_index) {
                if op_char != ' ' {
                    if operation_found {
                        return Err(invalid_input(format!(
                            "Unexpected duplicate operation in op column {}, char column {}",
                            operations.len() - 1,
                            column_index
                        )));
                    }
                    operation_found = true;
                    operations.push(op_char.to_string().parse()?);
                }
            }

            if any_digit_found {
                cur_value_vec.push(result);
            } else {
                if !operation_found {
                    return Err(invalid_input("Missing operation"));
                }
                operation_found = false;
                values.push(cur_value_vec.clone());
                cur_value_vec.clear();
            }

            column_index += 1;

            if !any_char_found {
                break;
            }
        }

        Ok(Self { values, operations })
    }

    fn compute_results(&self) -> Vec<i64> {
        let mut results = self
            .operations
            .iter()
            .map(|op| op.identity())
            .collect::<Vec<_>>();
        for (result_index, value_vec) in self.values.iter().enumerate() {
            for &value in value_vec {
                results[result_index] = self.operations[result_index]
                    .compute(results[result_index], value);
            }
        }
        results
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let problems = match part {
        Part::Part1 => ProblemSheet::new(reader),
        Part::Part2 => ProblemSheet::new_with_complex_parsing(reader),
    }?;

    let result = problems.compute_results().into_iter().sum::<i64>();
    println!("{result}");

    Ok(())
}
