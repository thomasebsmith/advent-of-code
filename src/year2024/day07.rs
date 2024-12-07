use std::collections::HashSet;
use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::parse::parse_words;
use crate::part::Part;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Operator {
    Plus,
    Times,
    Concat,
}

fn num_digits(positive_number: i64) -> u32 {
    if positive_number == 0 {
        1
    } else {
        positive_number.ilog10() + 1
    }
}

impl Operator {
    fn undo_operation(self, result: i64, second_operand: i64) -> Option<i64> {
        match self {
            Self::Plus => {
                // No negative numbers
                if second_operand > result {
                    None
                } else {
                    Some(result - second_operand)
                }
            }
            Self::Times => {
                if result % second_operand != 0 {
                    None
                } else {
                    Some(result / second_operand)
                }
            }
            Self::Concat => {
                let num_digits_result = num_digits(result);
                let num_digits_second_operand = num_digits(second_operand);
                if num_digits_second_operand >= num_digits_result {
                    None
                } else {
                    let mask = 10i64.pow(num_digits_second_operand);
                    if (result - second_operand) % mask != 0 {
                        None
                    } else {
                        Some((result - second_operand) / mask)
                    }
                }
            }
        }
    }
}

struct Equation {
    target: i64,
    operands: Vec<i64>,
}

impl Equation {
    fn from_line(line: &str) -> io::Result<Self> {
        let &[target_str, operands_str] =
            &line.split(": ").collect::<Vec<_>>()[..]
        else {
            return Err(invalid_input("\": \" not found"));
        };
        let operands: Vec<i64> = parse_words(operands_str)?;
        if operands.is_empty() {
            return Err(invalid_input("No operands"));
        }
        Ok(Self {
            target: target_str.parse().map_err(invalid_input)?,
            operands,
        })
    }

    fn is_possible<const ALLOW_CONCAT: bool>(&self) -> bool {
        let operators: Vec<Operator> = if ALLOW_CONCAT {
            vec![Operator::Plus, Operator::Times, Operator::Concat]
        } else {
            vec![Operator::Plus, Operator::Times]
        };

        let mut possible_starting_points = HashSet::<i64>::new();
        possible_starting_points.insert(self.target);

        let final_target = self.operands[0];

        for i in (1..self.operands.len()).rev() {
            let operand = self.operands[i];
            let is_last_undo = i == 1;

            let mut new_starting_points = HashSet::<i64>::new();
            for starting_point in possible_starting_points {
                for ref operator in operators.iter() {
                    let Some(new_result) =
                        operator.undo_operation(starting_point, operand)
                    else {
                        continue;
                    };
                    if is_last_undo && new_result == final_target {
                        return true;
                    }
                    new_starting_points.insert(new_result);
                }
            }

            if new_starting_points.is_empty() {
                return false;
            }

            possible_starting_points = new_starting_points;
        }

        // We return early if we hit the target, so we didn't hit it if we get
        // here.
        return false;
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut result: i64 = 0;

    for line in reader.lines() {
        let line = line?;
        let equation = Equation::from_line(&line)?;
        let is_possible = match part {
            Part::Part1 => equation.is_possible::<false>(),
            Part::Part2 => equation.is_possible::<true>(),
        };
        if is_possible {
            result += equation.target;
        }
    }

    println!("{result}");

    Ok(())
}
