use std::cmp::{max, min};
use std::collections::{BTreeSet, HashSet};
use std::io;
use std::ops::Range;

use crate::errors::invalid_input;
use crate::parse::{lines, paragraphs};
use crate::part::Part;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct MappingRange {
    source_start: i64,
    destination_start: i64,
    length: i64,
}

#[derive(Debug)]
struct TryMapResult {
    mapped: Range<i64>,
    left_before: Range<i64>,
    left_after: Range<i64>,
}

fn intersect<T: Ord>(r1: Range<T>, r2: Range<T>) -> Range<T> {
    max(r1.start, r2.start)..min(r1.end, r2.end)
}

impl MappingRange {
    fn from_line(line: &str) -> io::Result<Self> {
        let [destination_start, source_start, length] = line
            .split_whitespace()
            .map(|s| s.parse::<i64>().map_err(invalid_input))
            .collect::<io::Result<Vec<_>>>()?[..]
        else {
            return Err(invalid_input("Expected 3 numbers"));
        };

        Ok(Self {
            destination_start,
            source_start,
            length,
        })
    }

    fn try_map(&self, source_range: Range<i64>) -> TryMapResult {
        let left_before = intersect(
            source_range.clone(),
            source_range.start..self.source_start,
        );
        let left_after = intersect(
            source_range.clone(),
            (self.source_start + self.length)..source_range.end,
        );
        let overlap = intersect(
            source_range.clone(),
            self.source_start..(self.source_start + self.length),
        );
        let mapped = (overlap.start - self.source_start
            + self.destination_start)
            ..(overlap.end - self.source_start + self.destination_start);
        TryMapResult {
            mapped,
            left_before,
            left_after,
        }
    }
}

struct Mapping {
    ranges: BTreeSet<MappingRange>,
}

impl Mapping {
    fn new() -> Self {
        Self {
            ranges: BTreeSet::new(),
        }
    }

    fn add_range(&mut self, range: MappingRange) {
        self.ranges.insert(range);
    }

    fn apply(&self, source: Range<i64>) -> HashSet<Range<i64>> {
        let mut remaining = source.clone(); // TODO
        let mut result = HashSet::<Range<i64>>::new();
        for range in &self.ranges {
            let map_result = range.try_map(remaining);
            //println!("{:?}", map_result);
            if !map_result.left_before.is_empty() {
                result.insert(map_result.left_before);
            }
            if !map_result.mapped.is_empty() {
                result.insert(map_result.mapped);
            }
            if map_result.left_after.is_empty() {
                break;
            }
            remaining = map_result.left_after;
        }

        result
    }
}

struct Input {
    seeds: Vec<i64>,
    mappings: Vec<Mapping>,
}

impl Input {
    fn from_reader<R: io::Read>(reader: io::BufReader<R>) -> io::Result<Self> {
        let mut seeds = Vec::<i64>::new();
        let mut mappings = Vec::<Mapping>::new();

        for paragraph in paragraphs(lines(reader)?) {
            if seeds.is_empty() {
                if paragraph.len() != 1 {
                    return Err(invalid_input(
                        "Expected 1 line in first paragraph",
                    ));
                }

                seeds = paragraph[0]
                    .strip_prefix("seeds: ")
                    .ok_or_else(|| invalid_input("No seeds: "))?
                    .split_whitespace()
                    .map(|s| s.parse::<i64>().map_err(invalid_input))
                    .collect::<io::Result<_>>()?;
                if seeds.is_empty() {
                    return Err(invalid_input("Expected at least one seed"));
                }
                continue;
            }

            let mut current_mapping = Mapping::new();
            for line in paragraph {
                if line.ends_with(":") {
                    continue;
                }
                current_mapping.add_range(MappingRange::from_line(&line)?);
            }
            mappings.push(current_mapping);
        }

        Ok(Self { seeds, mappings })
    }
}

fn get_locations(
    seeds: HashSet<Range<i64>>,
    mappings: &Vec<Mapping>,
) -> HashSet<Range<i64>> {
    let mut result = seeds;
    for mapping in mappings {
        let mut next_set = HashSet::<Range<i64>>::new();
        for num in result {
            next_set.extend(mapping.apply(num));
        }
        result = next_set;
    }
    result
}

fn part1<R: io::Read>(reader: io::BufReader<R>) -> io::Result<()> {
    let input = Input::from_reader(reader)?;

    let seeds = input
        .seeds
        .into_iter()
        .map(|seed| seed..(seed + 1))
        .collect::<HashSet<_>>();

    let locations = get_locations(seeds, &input.mappings);

    let min_location = locations
        .into_iter()
        .map(|r| r.start)
        .min()
        .ok_or_else(|| invalid_input("No locations"))?;

    println!("{min_location}");

    Ok(())
}

fn part2<R: io::Read>(reader: io::BufReader<R>) -> io::Result<()> {
    let input = Input::from_reader(reader)?;

    if input.seeds.len() % 2 != 0 {
        return Err(invalid_input("Invalid seed ranges"));
    }

    let mut seeds = HashSet::<Range<i64>>::new();
    for [start, len] in input.seeds.into_iter().array_chunks::<2>() {
        seeds.insert(start..(start + len));
    }

    let locations = get_locations(seeds, &input.mappings);

    let min_location = locations
        .iter()
        .map(|r| r.start)
        .min()
        .ok_or_else(|| invalid_input("No locations"))?;

    println!("{min_location}");

    Ok(())
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    (match part {
        Part::Part1 => part1,
        Part::Part2 => part2,
    })(reader)
}
