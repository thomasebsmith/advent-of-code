use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::parse::parse_all;
use crate::part::Part;

#[derive(Clone, Debug)]
struct PosetNode<T: Hash> {
    points_to: HashSet<T>,
    pointed_to_by: HashSet<T>,
}

impl<T: Hash + Copy + Eq> PosetNode<T> {
    fn new(value: T) -> Self {
        let mut points_to = HashSet::new();
        points_to.insert(value);
        let pointed_to_by = points_to.clone();
        Self {
            points_to,
            pointed_to_by,
        }
    }
}

struct Poset<T: Hash> {
    graph: HashMap<T, PosetNode<T>>,
}

impl<T: Hash + Copy + Eq + Debug> Poset<T> {
    fn new() -> Self {
        Self {
            graph: HashMap::new(),
        }
    }

    fn node_at(&mut self, value: T) -> &mut PosetNode<T> {
        self.graph
            .entry(value)
            .or_insert_with(|| PosetNode::new(value))
    }

    fn add_link(&mut self, from: T, to: T) {
        if from != to && self.node_at(to).points_to.contains(&from) {
            panic!("{to:?} points to {from:?}");
        }

        self.node_at(from).points_to.insert(to);
        self.node_at(to).pointed_to_by.insert(from);
    }

    fn has_link(&self, from: T, to: T) -> bool {
        self.graph
            .get(&from)
            .is_some_and(|node| node.points_to.contains(&to))
    }

    fn add_comparison(&mut self, less: T, greater: T) {
        for greater_than_greater in self.node_at(greater).points_to.clone() {
            for less_than_less in self.node_at(less).pointed_to_by.clone() {
                self.add_link(less_than_less, greater_than_greater);
            }
        }
    }

    fn follows_order(&self, list: &Vec<T>) -> bool {
        for slice in list.windows(2) {
            let first = slice[0];
            let second = slice[1];
            if second != first && self.has_link(second, first) {
                return false;
            }
        }
        return true;
    }

    fn sort(&self, list: &mut Vec<T>) {
        list.sort_by(|first, second| {
            let first = *first;
            let second = *second;
            if first == second {
                Ordering::Equal
            } else if self.has_link(first, second) {
                Ordering::Less
            } else if self.has_link(second, first) {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });
    }
}

#[derive(Clone, Copy, Debug)]
struct Rule {
    first: i64,
    second: i64,
}

#[derive(Debug)]
struct RuleSet {
    rules: Vec<Rule>,
}

impl RuleSet {
    fn new() -> Self {
        Self { rules: Vec::new() }
    }

    fn add_rule(&mut self, first: i64, second: i64) {
        self.rules.push(Rule { first, second });
    }

    fn create_poset(&self, update: &Vec<i64>) -> Poset<i64> {
        let all_values_in_update: HashSet<i64> =
            update.iter().map(|x| *x).collect();
        let mut poset = Poset::<i64>::new();
        for rule in self.rules.iter() {
            if all_values_in_update.contains(&rule.first)
                && all_values_in_update.contains(&rule.second)
            {
                poset.add_comparison(rule.first, rule.second);
            }
        }
        poset
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut ruleset = RuleSet::new();
    let mut reading_comparisons = true;

    let mut result: i64 = 0;

    for line in reader.lines() {
        let line = line?;

        if line.is_empty() {
            reading_comparisons = false;
            continue;
        }

        if reading_comparisons {
            let &[less, greater] = &parse_all::<_, i64>(line.split("|"))?[..]
            else {
                return Err(invalid_input("Cannot parse comparison"));
            };
            ruleset.add_rule(less, greater);
        } else {
            let mut update: Vec<i64> = parse_all(line.split(","))?;
            if update.is_empty() {
                return Err(invalid_input("Empty update"));
            }
            let poset = ruleset.create_poset(&update);
            let follows_rules = poset.follows_order(&update);
            match part {
                Part::Part1 => {
                    if follows_rules {
                        result += update[update.len() / 2];
                    }
                }
                Part::Part2 => {
                    if !follows_rules {
                        poset.sort(&mut update);
                        result += update[update.len() / 2];
                    }
                }
            }
        }
    }
    println!("{result}");

    Ok(())
}
