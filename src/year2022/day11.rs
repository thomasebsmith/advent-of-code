use std::collections::BTreeMap;
use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::iter::n_elements;
use crate::part::Part;

enum Operator {
    Plus,
    Times,
}

impl Operator {
    pub fn new(string: &str) -> io::Result<Self> {
        match string {
            "+" => Ok(Self::Plus),
            "*" => Ok(Self::Times),
            _ => Err(invalid_input("Unknown operator")),
        }
    }
}

enum Operand {
    Constant(u64),
    Old,
}

impl Operand {
    pub fn new(string: &str) -> io::Result<Self> {
        match string {
            "old" => Ok(Self::Old),
            _ => Ok(Self::Constant(
                string.parse::<u64>().map_err(invalid_input)?,
            )),
        }
    }
}

struct Operation {
    operand1: Operand,
    operator: Operator,
    operand2: Operand,
}

impl Operation {
    pub fn new(string: &str) -> io::Result<Self> {
        let words = n_elements(
            3,
            string
                .strip_prefix("  Operation: new = ")
                .ok_or_else(|| {
                    invalid_input(
                        "Operation must begin with \"  Operation: new = \"",
                    )
                })?
                .split(' '),
        )
        .ok_or_else(|| invalid_input("Expected new = _ _ _"))?;

        Ok(Self {
            operand1: Operand::new(words[0])?,
            operator: Operator::new(words[1])?,
            operand2: Operand::new(words[2])?,
        })
    }

    pub fn apply(&self, value: u64) -> u64 {
        let value1 = match self.operand1 {
            Operand::Constant(val) => val.into(),
            Operand::Old => value,
        };
        let value2 = match self.operand2 {
            Operand::Constant(val) => val.into(),
            Operand::Old => value,
        };

        match self.operator {
            Operator::Plus => value1 + value2,
            Operator::Times => value1 * value2,
        }
    }
}

struct Test {
    pub divisibility_check: u64,
    pub monkey_if_true: usize,
    pub monkey_if_false: usize,
}

impl Test {
    pub fn new(lines: (&str, &str, &str)) -> io::Result<Self> {
        let divisibility_check = lines
            .0
            .strip_prefix("  Test: divisible by ")
            .ok_or_else(|| {
                invalid_input("Line must begin with \"  Test: divisible by \"")
            })?
            .parse::<u64>()
            .map_err(invalid_input)?;
        let monkey_if_true = lines
            .1
            .strip_prefix("    If true: throw to monkey ")
            .ok_or_else(|| {
                invalid_input(
                    "Line must begin with \"    If true: throw to monkey \"",
                )
            })?
            .parse::<usize>()
            .map_err(invalid_input)?;
        let monkey_if_false = lines
            .2
            .strip_prefix("    If false: throw to monkey ")
            .ok_or_else(|| {
                invalid_input(
                    "Line must begin with \"    If false: throw to monkey \"",
                )
            })?
            .parse::<usize>()
            .map_err(invalid_input)?;
        Ok(Self {
            divisibility_check,
            monkey_if_true,
            monkey_if_false,
        })
    }
}

struct Monkey {
    items: Vec<u64>,
    operation: Operation,
    pub test: Test,
    pub inspected_items: u64,
}

fn parse_monkey(lines: &[String]) -> io::Result<(usize, Monkey)> {
    if lines.len() != 6 {
        Err(invalid_input("Expected 6 lines"))?
    }

    let monkey_num = lines[0]
        .strip_prefix("Monkey ")
        .map(|string| string.strip_suffix(':'))
        .flatten()
        .ok_or_else(|| invalid_input("Expected monkey title"))?
        .parse::<usize>()
        .map_err(invalid_input)?;

    let items = lines[1]
        .strip_prefix("  Starting items: ")
        .ok_or_else(|| invalid_input("Expected starting items"))?
        .split(", ")
        .map(|string| string.parse::<u64>().map_err(invalid_input))
        .try_collect::<Vec<_>>()?;

    let operation = Operation::new(&lines[2])?;

    let test = Test::new((&lines[3], &lines[4], &lines[5]))?;

    Ok((
        monkey_num,
        Monkey {
            items,
            operation,
            test,
            inspected_items: 0,
        },
    ))
}

fn run_round(
    monkeys: &mut BTreeMap<usize, Monkey>,
    divide_worry: bool,
    worry_modulus: u64,
) {
    let monkey_nums = monkeys.keys().map(|i| *i).collect::<Vec<_>>();
    for i in monkey_nums.into_iter() {
        let items = monkeys[&i].items.clone();
        for item in items.into_iter() {
            let (new_item, new_monkey) = {
                let monkey = monkeys.get_mut(&i).unwrap();

                let mut new_item = monkey.operation.apply(item);

                monkey.inspected_items += 1;

                if divide_worry {
                    new_item /= 3;
                } else {
                    new_item %= worry_modulus;
                }

                let new_monkey =
                    if new_item % monkey.test.divisibility_check == 0 {
                        monkey.test.monkey_if_true
                    } else {
                        monkey.test.monkey_if_false
                    };

                (new_item, new_monkey)
            };

            monkeys.get_mut(&new_monkey).unwrap().items.push(new_item);
        }
        monkeys.get_mut(&i).unwrap().items.clear();
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut recent_lines = Vec::<String>::new();
    let mut monkeys = BTreeMap::<usize, Monkey>::new();

    for line in reader.lines() {
        let line = line?;
        if line == "" {
            let (monkey_num, monkey) = parse_monkey(&recent_lines[..])?;
            monkeys.insert(monkey_num, monkey);
            recent_lines.clear();
        } else {
            recent_lines.push(line);
        }
    }

    if !recent_lines.is_empty() {
        let (monkey_num, monkey) = parse_monkey(&recent_lines[..])?;
        monkeys.insert(monkey_num, monkey);
        recent_lines.clear();
    }

    let worry_modulus = monkeys
        .values()
        .map(|monkey| monkey.test.divisibility_check)
        .product::<u64>();

    let num_rounds: usize = match part {
        Part::Part1 => 20,
        Part::Part2 => 10_000,
    };

    for _ in 0..num_rounds {
        run_round(&mut monkeys, matches!(part, Part::Part1), worry_modulus);
    }

    let mut max_inspected_items_1: Option<u64> = None;
    let mut max_inspected_items_2: Option<u64> = None;
    for inspected_items in monkeys.values().map(|m| m.inspected_items) {
        match (max_inspected_items_1, max_inspected_items_2) {
            (None, _) => {
                max_inspected_items_1 = Some(inspected_items);
            }
            (Some(max_1), None) => {
                if inspected_items > max_1 {
                    max_inspected_items_1 = Some(inspected_items);
                    max_inspected_items_2 = Some(max_1);
                } else {
                    max_inspected_items_2 = Some(inspected_items);
                }
            }
            (Some(max_1), Some(max_2)) => {
                if inspected_items > max_1 {
                    max_inspected_items_1 = Some(inspected_items);
                    max_inspected_items_2 = Some(max_1);
                } else if inspected_items > max_2 {
                    max_inspected_items_2 = Some(inspected_items);
                }
            }
        }
    }

    println!(
        "{}",
        max_inspected_items_1
            .ok_or_else(|| invalid_input("Not enough monkeys"))?
            * max_inspected_items_2
                .ok_or_else(|| invalid_input("Not enough monkeys"))?
    );

    Ok(())
}
