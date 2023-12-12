use std::collections::HashMap;
use std::io;

use crate::errors::invalid_input;
use crate::parse::{lines, parse_all};
use crate::part::Part;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Spring {
    Operational,
    Damaged,
    Unknown,
}

impl Spring {
    fn from_char(ch: char) -> io::Result<Self> {
        match ch {
            '.' => Ok(Self::Operational),
            '#' => Ok(Self::Damaged),
            '?' => Ok(Self::Unknown),
            _ => Err(invalid_input("Invalid spring character")),
        }
    }
}

// TODO: rename
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct State {
    next_spring_index: usize,
    next_group_index: usize,
}

type Cache = HashMap<State, usize>;

struct RowOfSprings {
    springs: Vec<Spring>,
    groups: Vec<usize>,
}

impl RowOfSprings {
    fn from_line(line: &str) -> io::Result<Self> {
        let [springs_chars, groups_chars] =
            line.split_whitespace().collect::<Vec<_>>()[..]
        else {
            return Err(invalid_input("Expected <springs> <groups>"));
        };

        let springs = springs_chars
            .chars()
            .map(Spring::from_char)
            .collect::<io::Result<Vec<_>>>()?;
        let groups = parse_all::<_, usize>(groups_chars.split(','))?;

        Ok(Self { springs, groups })
    }

    fn possible_arrangements(&self) -> usize {
        let mut cache = Cache::new();
        self.possible_arrangements_helper(
            State {
                next_spring_index: 0,
                next_group_index: 0,
            },
            &mut cache,
        )
    }

    fn possible_arrangements_if_operational(
        &self,
        mut state: State,
        cache: &mut Cache,
    ) -> usize {
        state.next_spring_index += 1;
        self.cached_possible_arrangements_helper(state, cache)
    }

    fn possible_arrangements_if_damaged(
        &self,
        mut state: State,
        cache: &mut Cache,
    ) -> usize {
        for offset in 1..self.groups[state.next_group_index] {
            let i = state.next_spring_index + offset;
            if i >= self.springs.len() || self.springs[i] == Spring::Operational
            {
                return 0;
            }
        }

        let operational_spring_index =
            state.next_spring_index + self.groups[state.next_group_index];
        if operational_spring_index < self.springs.len() {
            if self.springs[operational_spring_index] == Spring::Damaged {
                0
            } else {
                state.next_spring_index +=
                    self.groups[state.next_group_index] + 1;
                state.next_group_index += 1;
                self.cached_possible_arrangements_helper(state, cache)
            }
        } else {
            state.next_spring_index += self.groups[state.next_group_index];
            state.next_group_index += 1;
            self.cached_possible_arrangements_helper(state, cache)
        }
    }

    fn cached_possible_arrangements_helper(
        &self,
        state: State,
        cache: &mut Cache,
    ) -> usize {
        if let Some(result) = cache.get(&state) {
            *result
        } else {
            let result = self.possible_arrangements_helper(state, cache);
            cache.insert(state, result);
            result
        }
    }

    fn possible_arrangements_helper(
        &self,
        state: State,
        cache: &mut Cache,
    ) -> usize {
        if state.next_spring_index >= self.springs.len() {
            return if state.next_group_index >= self.groups.len() {
                1
            } else {
                0
            };
        }

        if state.next_group_index >= self.groups.len() {
            for i in state.next_spring_index..self.springs.len() {
                if self.springs[i] == Spring::Damaged {
                    return 0;
                }
            }
            return 1;
        }

        match self.springs[state.next_spring_index] {
            Spring::Operational => {
                self.possible_arrangements_if_operational(state, cache)
            }
            Spring::Damaged => {
                self.possible_arrangements_if_damaged(state, cache)
            }
            Spring::Unknown => {
                let operational_state = state.clone();
                self.possible_arrangements_if_operational(
                    operational_state,
                    cache,
                ) + self.possible_arrangements_if_damaged(state, cache)
            }
        }
    }

    fn unfold(&self) -> Self {
        const REPEATS: usize = 5;

        let mut new_springs = Vec::<Spring>::new();
        let mut new_groups = Vec::<usize>::new();

        for i in 0..REPEATS {
            new_springs.extend(self.springs.iter());
            new_groups.extend(self.groups.iter());
            if i + 1 != REPEATS {
                new_springs.push(Spring::Unknown);
            }
        }

        Self {
            springs: new_springs,
            groups: new_groups,
        }
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut result: usize = 0;

    for line in lines(reader)? {
        let row = RowOfSprings::from_line(&line)?;
        let arrangements = match part {
            Part::Part1 => row,
            Part::Part2 => row.unfold(),
        }
        .possible_arrangements();
        result += arrangements;
    }

    println!("{result}");

    Ok(())
}
