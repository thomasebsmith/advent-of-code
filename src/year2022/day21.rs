use std::collections::HashMap;
use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::part::Part;

type Num = i64;

// TODO: be more efficient instead of cloning
#[derive(Clone)]
enum Operation {
    Add(String, String),
    Subtract(String, String),
    Multiply(String, String),
    Divide(String, String),
    CheckEquality(String, String),
    Constant(Num),
}

struct Monkey {
    operation: Operation,
    cached_result: Option<i64>,
    cached_depends_on_human: Option<bool>,
}

struct Monkeys {
    monkeys: HashMap<String, Monkey>,
}

impl Monkeys {
    pub fn new() -> Self {
        Self {
            monkeys: HashMap::new(),
        }
    }

    pub fn add_monkey(&mut self, name: String, operation: Operation) {
        self.monkeys.insert(
            name,
            Monkey {
                operation,
                cached_result: None,
                cached_depends_on_human: None,
            },
        );
    }

    pub fn modify_root(&mut self) {
        // TODO
        let monkey = self.monkeys.get_mut("root").unwrap();

        // TODO
        let (m1, m2) = match &monkey.operation {
            Operation::Add(m1, m2) => (m1, m2),
            _ => panic!("TODO"),
        };

        self.monkeys.get_mut("root").unwrap().operation =
            Operation::CheckEquality(String::from(m1), String::from(m2));
    }

    pub fn get_result(&mut self, monkey_name: &str) -> Num {
        // TODO: panics
        let monkey = self.monkeys.get_mut(monkey_name).unwrap();
        if let Some(result) = monkey.cached_result {
            return result;
        }

        // TODO: inf recursion check??
        let result = match &monkey.operation.clone() {
            Operation::Add(m1, m2) => self.get_result(m1) + self.get_result(m2),
            Operation::Subtract(m1, m2) => {
                self.get_result(m1) - self.get_result(m2)
            }
            Operation::Multiply(m1, m2) => {
                self.get_result(m1) * self.get_result(m2)
            }
            Operation::Divide(m1, m2) => {
                self.get_result(m1) / self.get_result(m2)
            }
            Operation::CheckEquality(m1, m2) => {
                if self.get_result(m1) == self.get_result(m2) {
                    1
                } else {
                    0
                }
            }
            Operation::Constant(num) => *num,
        };

        self.monkeys.get_mut(monkey_name).unwrap().cached_result = Some(result);
        result
    }

    fn depends_on_human(&mut self, monkey_name: &str) -> bool {
        let monkey = self.monkeys.get(monkey_name).unwrap();
        if let Some(result) = monkey.cached_depends_on_human {
            return result;
        }

        // TODO: use constant or similar for "humn"
        let result = if monkey_name == "humn" {
            true
        } else {
            match &monkey.operation.clone() {
                Operation::Add(m1, m2) => {
                    self.depends_on_human(m1) || self.depends_on_human(m2)
                }
                Operation::Subtract(m1, m2) => {
                    self.depends_on_human(m1) || self.depends_on_human(m2)
                }
                Operation::Multiply(m1, m2) => {
                    self.depends_on_human(m1) || self.depends_on_human(m2)
                }
                Operation::Divide(m1, m2) => {
                    self.depends_on_human(m1) || self.depends_on_human(m2)
                }
                Operation::CheckEquality(m1, m2) => {
                    self.depends_on_human(m1) || self.depends_on_human(m2)
                }
                Operation::Constant(_) => false,
            }
        };

        self.monkeys
            .get_mut(monkey_name)
            .unwrap()
            .cached_depends_on_human = Some(result);
        result
    }

    pub fn get_human(
        &mut self,
        monkey_name: &str,
        desired_result: Num,
    ) -> Option<Num> {
        if monkey_name == "humn" {
            return Some(desired_result);
        }

        if !self.depends_on_human(monkey_name) {
            return None;
        }

        let operation =
            self.monkeys.get(monkey_name).unwrap().operation.clone();
        let (m1, m2) = match &operation {
            Operation::Add(m1, m2) => (m1, m2),
            Operation::Subtract(m1, m2) => (m1, m2),
            Operation::Multiply(m1, m2) => (m1, m2),
            Operation::Divide(m1, m2) => (m1, m2),
            Operation::CheckEquality(m1, m2) => (m1, m2),
            _ => panic!("Constant depends on input"),
        };

        if self.depends_on_human(m1) {
            assert!(!self.depends_on_human(m2));
            let result2 = self.get_result(m2);
            let desired_subresult = match operation {
                Operation::Add(_, _) => desired_result - result2,
                Operation::Subtract(_, _) => desired_result + result2,
                Operation::Multiply(_, _) => desired_result / result2,
                Operation::Divide(_, _) => desired_result * result2,
                Operation::CheckEquality(_, _) => {
                    assert!(desired_result == 1);
                    result2
                }
                _ => panic!("TODO"),
            };
            self.get_human(m1, desired_subresult)
        } else {
            assert!(self.depends_on_human(m2));
            let result1 = self.get_result(m1);
            let desired_subresult = match operation {
                Operation::Add(_, _) => desired_result - result1,
                Operation::Subtract(_, _) => result1 - desired_result,
                Operation::Multiply(_, _) => desired_result / result1,
                Operation::Divide(_, _) => result1 / desired_result,
                Operation::CheckEquality(_, _) => {
                    assert!(desired_result == 1);
                    result1
                }
                _ => panic!("TODO"),
            };
            self.get_human(m2, desired_subresult)
        }
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut monkeys = Monkeys::new();
    for line in reader.lines() {
        let line = line?;

        let &[name, operation_str] = &line.split(": ").collect::<Vec<_>>()[..] else {
            Err(invalid_input("Expected \": \" separator"))?
        };

        let operation_words = operation_str.split(' ').collect::<Vec<_>>();
        let operation = match &operation_words[..] {
            &[number] => Operation::Constant(
                number.parse::<Num>().map_err(invalid_input)?,
            ),
            &[m1, "+", m2] => {
                Operation::Add(String::from(m1), String::from(m2))
            }
            &[m1, "-", m2] => {
                Operation::Subtract(String::from(m1), String::from(m2))
            }
            &[m1, "*", m2] => {
                Operation::Multiply(String::from(m1), String::from(m2))
            }
            &[m1, "/", m2] => {
                Operation::Divide(String::from(m1), String::from(m2))
            }
            _ => Err(invalid_input("Invalid operation"))?,
        };

        monkeys.add_monkey(String::from(name), operation);
    }

    match part {
        Part::Part1 => {
            println!("{}", monkeys.get_result("root"));
        }
        Part::Part2 => {
            monkeys.modify_root();
            println!("{}", monkeys.get_human("root", 1).unwrap());
        }
    }

    Ok(())
}
