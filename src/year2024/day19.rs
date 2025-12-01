use std::collections::{BTreeMap, HashMap, HashSet};
use std::io;

use crate::errors::invalid_input;
use crate::parse::{lines, paragraphs};
use crate::part::Part;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Color {
    name: char,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Pattern {
    colors: Vec<Color>,
}

impl Pattern {
    const EMPTY: Self = Self { colors: Vec::new() };

    fn prefix(&self, count: usize) -> Self {
        if count >= self.colors.len() {
            self.clone()
        } else {
            Self {
                colors: (&self.colors[..count]).to_vec(),
            }
        }
    }

    fn suffix(&self, after_skipping: usize) -> Self {
        if after_skipping >= self.colors.len() {
            Self::EMPTY
        } else {
            Self {
                colors: (&self.colors[after_skipping..]).to_vec(),
            }
        }
    }
}

type Design = Pattern;

struct PatternSet {
    patterns: BTreeMap<usize, HashSet<Pattern>>,
    design_cache: HashMap<Design, usize>,
}

impl PatternSet {
    fn new(line: &str) -> Self {
        let mut patterns = BTreeMap::<usize, HashSet<Pattern>>::new();
        for pattern_str in line.split(", ") {
            let pattern = Pattern {
                colors: pattern_str
                    .chars()
                    .map(|name| Color { name })
                    .collect(),
            };
            patterns
                .entry(pattern.colors.len())
                .or_insert_with(HashSet::new)
                .insert(pattern);
        }

        Self {
            patterns,
            design_cache: HashMap::new(),
        }
    }

    fn ways_possible_helper(&mut self, design: &Design) -> usize {
        let design_length = design.colors.len();
        if design_length == 0 {
            return 1;
        }

        let max_pattern_length = self
            .patterns
            .last_key_value()
            .map(|pair| *pair.0)
            .unwrap_or(0)
            .min(design_length);
        let mut result = 0usize;
        for pattern_length in 1..=max_pattern_length {
            let Some(pattern_set) = self.patterns.get(&pattern_length) else {
                continue;
            };
            let prefix = design.prefix(pattern_length);
            if pattern_set.contains(&prefix) {
                result += self.ways_possible(&design.suffix(pattern_length));
            }
        }

        result
    }

    fn ways_possible(&mut self, design: &Design) -> usize {
        if let Some(&cached_value) = self.design_cache.get(&design) {
            cached_value
        } else {
            let computed_value = self.ways_possible_helper(design);
            self.design_cache.insert(design.clone(), computed_value);
            computed_value
        }
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let paragraphs = paragraphs(lines(reader)?).collect::<Vec<_>>();
    if paragraphs.len() != 2 {
        return Err(invalid_input("Expected two paragraphs"));
    }
    if paragraphs[0].len() != 1 {
        return Err(invalid_input("Expected first paragraph to be one line"));
    }

    let mut pattern_set = PatternSet::new(&paragraphs[0][0]);

    let design_ways_iterator = paragraphs[1].iter().map(|design_str| {
        let design = Design {
            colors: design_str.chars().map(|name| Color { name }).collect(),
        };
        pattern_set.ways_possible(&design)
    });

    let result = match part {
        Part::Part1 => design_ways_iterator.filter(|&ways| ways > 0).count(),
        Part::Part2 => design_ways_iterator.sum(),
    };

    println!("{result}");

    Ok(())
}
