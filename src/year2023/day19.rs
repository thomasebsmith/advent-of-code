use std::collections::{HashMap, VecDeque};
use std::io;

use crate::errors::invalid_input;
use crate::parse::{lines, paragraphs};
use crate::part::Part;

type Category = char;
type Value = i64;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Range {
    start: Value,
    end: Value, // exclusive
}

impl Range {
    fn size(&self) -> Value {
        self.end - self.start
    }

    fn split_at(&self, value: Value) -> (Self, Self) {
        if value > self.start {
            if value >= self.end {
                (
                    *self,
                    Self {
                        start: value,
                        end: value,
                    },
                )
            } else {
                (
                    Self {
                        start: self.start,
                        end: value,
                    },
                    Self {
                        start: value,
                        end: self.end,
                    },
                )
            }
        } else {
            (
                Self {
                    start: value,
                    end: value,
                },
                *self,
            )
        }
    }
}

// It would be faster to use an array/struct instead of a hashmap, but a hashmap
// is fast enough.
struct MachinePart {
    ratings: HashMap<Category, Value>,
}

impl MachinePart {
    fn from_string(string: &str) -> io::Result<Self> {
        let Some(string) = string.strip_prefix('{') else {
            return Err(invalid_input("Expected { in part string"));
        };
        let Some(string) = string.strip_suffix('}') else {
            return Err(invalid_input("Expected } in part string"));
        };

        let ratings = string
            .split(",")
            .map(|pair| {
                let [category, value] = pair.split('=').collect::<Vec<_>>()[..]
                else {
                    return Err(invalid_input("Expected = in part value"));
                };
                if category.len() != 1 {
                    return Err(invalid_input("Category must be 1 char"));
                }
                Ok((
                    category.chars().next().unwrap(),
                    value.parse::<Value>().map_err(invalid_input)?,
                ))
            })
            .collect::<io::Result<HashMap<Category, Value>>>()?;

        Ok(Self { ratings })
    }

    fn ratings_sum(&self) -> Value {
        self.ratings.values().sum()
    }
}

#[derive(Clone)]
struct MachinePartSet {
    ratings: HashMap<Category, Range>,
}

impl MachinePartSet {
    fn new() -> Self {
        let starting_range = Range {
            start: 1,
            end: 4000 + 1,
        };
        Self {
            ratings: HashMap::from([
                ('x', starting_range),
                ('m', starting_range),
                ('a', starting_range),
                ('s', starting_range),
            ]),
        }
    }

    fn empty() -> Self {
        let empty_range = Range { start: 1, end: 1 };
        Self {
            ratings: HashMap::from([
                ('x', empty_range),
                ('m', empty_range),
                ('a', empty_range),
                ('s', empty_range),
            ]),
        }
    }

    fn size(&self) -> Value {
        self.ratings.values().map(Range::size).product()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum Operator {
    LessThan,
    GreaterThan,
}

impl Operator {
    fn from_char(ch: char) -> io::Result<Self> {
        match ch {
            '<' => Ok(Self::LessThan),
            '>' => Ok(Self::GreaterThan),
            _ => Err(invalid_input("Invalid operator char")),
        }
    }

    fn apply(self, value1: Value, value2: Value) -> bool {
        match self {
            Self::LessThan => value1 < value2,
            Self::GreaterThan => value1 > value2,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
enum Operation {
    SendTo(String),
    Accept,
    Reject,
}

impl Operation {
    fn from_string(string: &str) -> io::Result<Self> {
        match string {
            "A" => Ok(Self::Accept),
            "R" => Ok(Self::Reject),
            _ => Ok(Self::SendTo(string.to_owned())),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
struct Condition {
    category: Category,
    operator: Operator,
    constant: Value,
}

impl Condition {
    fn from_string(string: &str) -> io::Result<Self> {
        if string.len() < 3 {
            return Err(invalid_input("Condition too short"));
        }
        let [category, operator_char, ..] =
            &string.chars().collect::<Vec<_>>()[..]
        else {
            return Err(invalid_input("Condition is not of the form a=C"));
        };
        let operator = Operator::from_char(*operator_char)?;
        let constant = string[2..].parse::<Value>().map_err(invalid_input)?;
        Ok(Self {
            category: *category,
            operator,
            constant,
        })
    }

    fn matches(&self, part: &MachinePart) -> bool {
        let Some(value) = part.ratings.get(&self.category) else {
            return false;
        };
        self.operator.apply(*value, self.constant)
    }

    fn split(
        &self,
        mut set: MachinePartSet,
    ) -> (MachinePartSet, MachinePartSet) {
        let Some(existing_rating) = set.ratings.get(&self.category) else {
            return (MachinePartSet::empty(), set);
        };

        let (matching, not_matching) = match self.operator {
            Operator::LessThan => {
                let (matching, not_matching) =
                    existing_rating.split_at(self.constant);
                (matching, not_matching)
            }
            Operator::GreaterThan => {
                let (not_matching, matching) =
                    existing_rating.split_at(self.constant + 1);
                (matching, not_matching)
            }
        };
        let mut new_set = set.clone();
        set.ratings.insert(self.category, matching);
        new_set.ratings.insert(self.category, not_matching);
        (set, new_set)
    }
}

struct Rule {
    condition: Option<Condition>,
    operation: Operation,
}

impl Rule {
    fn from_string(string: &str) -> io::Result<Self> {
        let [condition_str, operation_str] =
            &string.split(':').collect::<Vec<_>>()[..]
        else {
            return Ok(Self {
                condition: None,
                operation: Operation::from_string(string)?,
            });
        };

        let condition = Condition::from_string(condition_str)?;
        let operation = Operation::from_string(operation_str)?;

        Ok(Self {
            condition: Some(condition),
            operation,
        })
    }
}

struct Workflow {
    name: String,
    rules: Vec<Rule>,
}

impl Workflow {
    fn apply(&self, part: &MachinePart) -> Option<Operation> {
        for rule in &self.rules {
            let matches = if let Some(condition) = rule.condition {
                condition.matches(part)
            } else {
                true
            };
            if matches {
                return Some(rule.operation.clone());
            }
        }
        None
    }

    fn from_string(string: &str) -> io::Result<Self> {
        let [name_str, rules_str] = string.split('{').collect::<Vec<_>>()[..]
        else {
            return Err(invalid_input("Expected { in workflow"));
        };
        let name = name_str.to_owned();

        let Some(rules_str) = rules_str.strip_suffix('}') else {
            return Err(invalid_input("Workflow did not end in }"));
        };

        let rules = rules_str
            .split(',')
            .map(Rule::from_string)
            .collect::<io::Result<Vec<_>>>()?;

        Ok(Self { name, rules })
    }
}

const STARTING_WORKFLOW_NAME: &str = "in";

struct WorkflowSet {
    workflows: HashMap<String, Workflow>,
}

impl WorkflowSet {
    fn from_lines(lines: &Vec<String>) -> io::Result<Self> {
        let workflows = lines
            .iter()
            .map(|line| {
                Workflow::from_string(line)
                    .map(|workflow| (workflow.name.clone(), workflow))
            })
            .collect::<io::Result<HashMap<String, Workflow>>>()?;
        Ok(Self { workflows })
    }

    fn is_accepted(&self, part: &MachinePart) -> bool {
        let mut current_workflow_name = STARTING_WORKFLOW_NAME.to_owned();
        loop {
            let Some(workflow) = self.workflows.get(&current_workflow_name)
            else {
                panic!("Invalid workflow referenced");
            };
            match workflow.apply(part) {
                Some(Operation::SendTo(workflow_name)) => {
                    current_workflow_name = workflow_name;
                }
                Some(Operation::Accept) => {
                    break true;
                }
                Some(Operation::Reject) => {
                    break false;
                }
                None => {
                    panic!("Invalid rule sequence");
                }
            }
        }
    }

    fn num_accepted_parts(&self) -> i64 {
        let mut to_check_queue = VecDeque::<(String, MachinePartSet)>::new();
        to_check_queue.push_back((
            STARTING_WORKFLOW_NAME.to_owned(),
            MachinePartSet::new(),
        ));

        let mut num_accepted: i64 = 0;

        while let Some((ref workflow_name, mut set)) =
            to_check_queue.pop_front()
        {
            let Some(workflow) = self.workflows.get(workflow_name) else {
                continue;
            };
            for rule in &workflow.rules {
                let (passing, new_set) = match rule.condition {
                    Some(condition) => condition.split(set),
                    None => (set, MachinePartSet::empty()),
                };
                set = new_set;
                let passing_size = passing.size();
                if passing_size != 0 {
                    match rule.operation {
                        Operation::SendTo(ref dest) => {
                            to_check_queue
                                .push_back((dest.to_owned(), passing));
                        }
                        Operation::Accept => {
                            num_accepted += passing_size;
                        }
                        Operation::Reject => { /* do nothing */ }
                    }
                }

                if set.size() == 0 {
                    break;
                }
            }
        }

        num_accepted
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let [workflows_lines, parts_lines] =
        &paragraphs(lines(reader)?).collect::<Vec<_>>()[..]
    else {
        return Err(invalid_input(
            "Expected separate workflows and parts sections",
        ));
    };

    let workflow_set = WorkflowSet::from_lines(workflows_lines)?;
    let part_set = parts_lines
        .into_iter()
        .map(|part_str| MachinePart::from_string(part_str))
        .collect::<io::Result<Vec<_>>>()?;

    match part {
        Part::Part1 => {
            let mut result: i64 = 0;
            for part in part_set {
                if workflow_set.is_accepted(&part) {
                    result += part.ratings_sum();
                }
            }
            println!("{result}");
        }
        Part::Part2 => {
            let result = workflow_set.num_accepted_parts();
            println!("{result}");
        }
    }

    Ok(())
}
