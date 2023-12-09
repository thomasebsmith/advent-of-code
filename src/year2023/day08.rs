use std::cell::RefCell;
use std::collections::HashMap;
use std::io;

use crate::errors::invalid_input;
use crate::parse::{lines, paragraphs};
use crate::part::Part;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Step {
    Left,
    Right,
}

impl Step {
    fn from_char(ch: char) -> io::Result<Self> {
        match ch {
            'L' => Ok(Self::Left),
            'R' => Ok(Self::Right),
            _ => Err(invalid_input("Invalid step character")),
        }
    }
}

struct Paths {
    left_path: String,
    right_path: String,
}

struct Map {
    paths_by_source: HashMap<String, Paths>,
}

impl Map {
    fn from_lines(lines: &Vec<String>) -> io::Result<Self> {
        let mut paths_by_source = HashMap::<String, Paths>::new();
        for line in lines {
            let [source, _, left, right] =
                line.split_whitespace().collect::<Vec<_>>()[..]
            else {
                return Err(invalid_input("Expected source = (left, right)"));
            };

            let left = left
                .strip_prefix("(")
                .and_then(|string| string.strip_suffix(","))
                .ok_or_else(|| invalid_input("Invalid left punctuation"))?;

            let right = right
                .strip_suffix(")")
                .ok_or_else(|| invalid_input("Invalid right punctuation"))?;

            if paths_by_source.contains_key(source) {
                return Err(invalid_input("Duplicate source"));
            }
            paths_by_source.insert(
                source.to_owned(),
                Paths {
                    left_path: left.to_owned(),
                    right_path: right.to_owned(),
                },
            );
        }

        Ok(Self { paths_by_source })
    }

    fn contains_source(&self, source: &str) -> bool {
        self.paths_by_source.contains_key(source)
    }

    fn travel(&self, start: &str, step: Step) -> &str {
        assert!(self.contains_source(start));
        let paths = self.paths_by_source.get(start).unwrap();
        match step {
            Step::Left => &paths.left_path,
            Step::Right => &paths.right_path,
        }
    }

    fn sources(&self) -> impl Iterator<Item = &str> {
        self.paths_by_source.keys().map(|key| key.as_ref())
    }
}

// This traverser-based algorithm works generically. It handles multiple ends
// in a cycle, starting points that are not part of the cycle, etc. However, it
// is much smaller than an LCM-based solution, which would work for contrived
// inputs such as my AOC input.
struct Traverser<'a, const IS_PART_2: bool> {
    map: Map,
    steps: Vec<Step>,
    cache: RefCell<HashMap<(usize, &'a str), (usize, &'a str)>>,
}

const PART_1_START_LOCATION: &str = "AAA";

impl<'a, const IS_PART_2: bool> Traverser<'a, IS_PART_2> {
    fn new(map: Map, steps: Vec<Step>) -> io::Result<Self> {
        if steps.is_empty() {
            return Err(invalid_input("No steps"));
        }

        if !IS_PART_2 && !map.contains_source(PART_1_START_LOCATION) {
            return Err(invalid_input("No start location"));
        }

        Ok(Self {
            map,
            steps,
            cache: RefCell::new(HashMap::new()),
        })
    }

    fn is_end(location: &str) -> bool {
        if IS_PART_2 {
            location.ends_with("Z")
        } else {
            location == "ZZZ"
        }
    }

    fn start_locations(&self) -> Vec<&str> {
        if IS_PART_2 {
            self.map
                .sources()
                .filter(|source| source.ends_with("A"))
                .collect()
        } else {
            vec![PART_1_START_LOCATION]
        }
    }

    fn distance_to_next_end(
        &'a self,
        current_offset: usize,
        current_location: &'a str,
    ) -> (usize, &'a str) {
        let cache_key = (current_offset % self.steps.len(), current_location);
        {
            if let Some(result) = self.cache.borrow().get(&cache_key) {
                return *result;
            }
        }

        // We don't cache results that aren't explicitly asked for, because it won't help much and just takes extra memory.
        let mut distance: usize = 0;
        let mut location = current_location;

        loop {
            location = self.map.travel(
                location,
                self.steps[(current_offset + distance) % self.steps.len()],
            );
            distance += 1;

            if Self::is_end(location) {
                break;
            }
        }

        self.cache
            .borrow_mut()
            .insert(cache_key, (distance, location));
        (distance, location)
    }

    fn distance_until_all_at_end(&'a self) -> usize {
        let mut locations = self.start_locations();

        if locations.is_empty() {
            return 0;
        }

        let mut offsets = Vec::<usize>::new();
        offsets.reserve(locations.len());
        for location in locations.iter_mut() {
            let (distance, next_location) =
                self.distance_to_next_end(0, *location);
            *location = next_location;
            offsets.push(distance);
        }

        loop {
            let mut min_offset = usize::MAX;
            let mut min_offset_idx: usize = 0;
            let mut max_offset = usize::MIN;
            for (i, offset) in offsets.iter().enumerate() {
                let offset = *offset;
                if offset < min_offset {
                    min_offset_idx = i;
                    min_offset = offset;
                }
                if offset > max_offset {
                    max_offset = offset;
                }
            }

            if min_offset == max_offset {
                return min_offset;
            }

            let (distance, next_location) = self.distance_to_next_end(
                offsets[min_offset_idx],
                locations[min_offset_idx],
            );
            offsets[min_offset_idx] += distance;
            locations[min_offset_idx] = next_location;
        }
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let [steps, map] = &paragraphs(lines(reader)?).collect::<Vec<_>>()[..]
    else {
        return Err(invalid_input("Expected steps and map"));
    };

    let [steps] = &steps[..] else {
        return Err(invalid_input("Expected one line of steps"));
    };

    let steps = steps
        .chars()
        .map(Step::from_char)
        .collect::<io::Result<Vec<_>>>()?;

    let map = Map::from_lines(map)?;

    let result = match part {
        Part::Part1 => {
            Traverser::<false>::new(map, steps)?.distance_until_all_at_end()
        }
        Part::Part2 => {
            Traverser::<true>::new(map, steps)?.distance_until_all_at_end()
        }
    };

    println!("{result}");

    Ok(())
}
