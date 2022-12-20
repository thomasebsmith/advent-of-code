use std::cmp::max;
use std::collections::HashMap;
use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::part::Part;

// TODO: Make this solution faster

const NUM_RESOURCE_TYPES: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ResourceType {
    Ore = 0,
    Clay = 1,
    Obsidian = 2,
    Geode = 3,
}

impl ResourceType {
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "ore" => Some(Self::Ore),
            "clay" => Some(Self::Clay),
            "obsidian" => Some(Self::Obsidian),
            "geode" => Some(Self::Geode),
            _ => None,
        }
    }

    fn from_number(number: usize) -> Option<Self> {
        match number {
            0 => Some(Self::Ore),
            1 => Some(Self::Clay),
            2 => Some(Self::Obsidian),
            3 => Some(Self::Geode),
            _ => None,
        }
    }
}

struct Blueprint {
    robot_costs: [usize; NUM_RESOURCE_TYPES * NUM_RESOURCE_TYPES],
}

impl Blueprint {
    pub fn costs(&self, robot_type: ResourceType) -> &[usize] {
        let start_index = (robot_type as usize) * NUM_RESOURCE_TYPES;
        let end_index = start_index + NUM_RESOURCE_TYPES;
        &self.robot_costs[start_index..end_index]
    }

    pub fn max_geodes(&self, minutes: usize) -> usize {
        let mut state_manager = StateManager {
            cache: HashMap::new(),
        };
        state_manager.max_geodes(&self, minutes)
    }
}

struct StateManager {
    cache: HashMap<StateCacheEntry, usize>,
}

impl StateManager {
    pub fn max_geodes(
        &mut self,
        blueprint: &Blueprint,
        minutes: usize,
    ) -> usize {
        let mut state = State {
            minutes_remaining: minutes,
            resources: [0; NUM_RESOURCE_TYPES],
            robots: [1, 0, 0, 0],
            pending_robots: [0; NUM_RESOURCE_TYPES],
            maximum_geodes_seen: 0,
        };

        self.max_geodes_cached(blueprint, &mut state).unwrap_or(0)
    }

    fn max_geodes_cached(
        &mut self,
        blueprint: &Blueprint,
        state: &mut State,
    ) -> Option<usize> {
        let cache_entry = state.cache_entry();

        let cache_value = self.cache.get(&cache_entry);

        match cache_value {
            None => {
                let result_to_cache =
                    self.max_geodes_uncached(blueprint, state);
                if let Some(value) = result_to_cache {
                    self.cache.insert(cache_entry, value);
                }

                result_to_cache
            }
            Some(value) => Some(*value),
        }
    }

    // Call this right after ticking a minute
    fn max_geodes_upper_bound(
        &self,
        blueprint: &Blueprint,
        state: &State,
    ) -> usize {
        if state.minutes_remaining == 0 {
            return 0;
        }

        // Best case: build a geode robot every minute
        let mut geodes = 0;
        let mut num_geode_robots = state.robots[ResourceType::Geode as usize];
        // Wait a minute for the first robot to be available
        geodes += num_geode_robots;
        for _ in 0..(state.minutes_remaining - 1) {
            // Build a geode robot
            num_geode_robots += 1;
            // Tick a minute
            geodes += num_geode_robots;
        }
        geodes
    }

    fn max_geodes_uncached(
        &mut self,
        blueprint: &Blueprint,
        state: &mut State,
    ) -> Option<usize> {
        let pending = state.pending_robots.clone();
        let starting_geode = state.resources[ResourceType::Geode as usize];

        state.tick_minute();

        let total_num_geode = state.resources[ResourceType::Geode as usize];
        let gain = total_num_geode - starting_geode;

        let result = (if state.minutes_remaining == 0 {
            state.maximum_geodes_seen =
                max(state.maximum_geodes_seen, total_num_geode);
            Some(0)
        } else {
            let upper_bound = self.max_geodes_upper_bound(blueprint, state);
            if total_num_geode + upper_bound <= state.maximum_geodes_seen {
                None
            } else {
                self.build_robots(blueprint, state)
            }
        })
        .map(|result| result + gain);

        state.untick_minute(pending);

        result
    }

    fn build_robots(
        &mut self,
        blueprint: &Blueprint,
        state: &mut State,
    ) -> Option<usize> {
        if state.can_build(blueprint, ResourceType::Geode) {
            let costs = blueprint.costs(ResourceType::Geode);
            for (i, cost) in costs.iter().enumerate() {
                state.resources[i] -= cost;
            }
            state.pending_robots[ResourceType::Geode as usize] += 1;

            let result = self.max_geodes_cached(blueprint, state);

            state.pending_robots[ResourceType::Geode as usize] -= 1;
            for (i, cost) in costs.iter().enumerate() {
                state.resources[i] += cost;
            }

            return result;
        }

        let mut result = self.max_geodes_cached(blueprint, state);

        for i in (0..(NUM_RESOURCE_TYPES - 1)).rev() {
            // Try building a robot of this type
            let robot_type = ResourceType::from_number(i).unwrap();

            if state.can_build(blueprint, robot_type) {
                let costs = blueprint.costs(robot_type);
                for (j, cost) in costs.iter().enumerate() {
                    state.resources[j] -= cost;
                }
                state.pending_robots[i] += 1;

                result = max(result, self.max_geodes_cached(blueprint, state));

                state.pending_robots[i] -= 1;
                for (j, cost) in costs.iter().enumerate() {
                    state.resources[j] += cost;
                }
            }
        }

        result
    }
}

#[derive(Debug, PartialEq, Eq)]
struct State {
    minutes_remaining: usize,
    resources: [usize; NUM_RESOURCE_TYPES],
    robots: [usize; NUM_RESOURCE_TYPES],
    pending_robots: [usize; NUM_RESOURCE_TYPES],
    maximum_geodes_seen: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct StateCacheEntry {
    minutes_remaining: usize,
    resources: [usize; NUM_RESOURCE_TYPES - 1],
    robots: [usize; NUM_RESOURCE_TYPES],
    pending_robots: [usize; NUM_RESOURCE_TYPES],
}

impl State {
    fn cache_entry(&self) -> StateCacheEntry {
        StateCacheEntry {
            minutes_remaining: self.minutes_remaining,
            resources: [
                self.resources[0],
                self.resources[1],
                self.resources[2],
            ],
            robots: self.robots.clone(),
            pending_robots: self.pending_robots.clone(),
        }
    }

    fn can_build(
        &self,
        blueprint: &Blueprint,
        robot_type: ResourceType,
    ) -> bool {
        let costs = blueprint.costs(robot_type);
        for (i, cost) in costs.iter().enumerate() {
            if self.resources[i] < *cost {
                return false;
            }
        }
        true
    }

    fn tick_minute(&mut self) {
        for i in 0..NUM_RESOURCE_TYPES {
            self.resources[i] += self.robots[i];
            self.robots[i] += self.pending_robots[i];
            self.pending_robots[i] = 0;
        }
        self.minutes_remaining -= 1;
    }

    fn untick_minute(&mut self, pending_robots: [usize; NUM_RESOURCE_TYPES]) {
        self.minutes_remaining += 1;
        self.pending_robots = pending_robots;
        for i in 0..NUM_RESOURCE_TYPES {
            self.robots[i] -= self.pending_robots[i];
            self.resources[i] -= self.robots[i];
        }
    }
}

fn parse_costs(destination: &mut [usize], costs_text: &str) -> io::Result<()> {
    for i in 0..NUM_RESOURCE_TYPES {
        destination[i] = 0;
    }

    for cost_text in costs_text.split(" and ") {
        let [num, name] = &cost_text.split(' ').collect::<Vec<_>>()[..] else {
            Err(invalid_input("Expected <number> <name> for resource"))?
        };

        let cost = num.parse::<usize>().map_err(invalid_input)?;
        let resource_type = ResourceType::from_name(name).ok_or_else(|| {
            invalid_input(format!("Invalid resource name \"{}\"", name))
        })?;
        destination[resource_type as usize] += cost;
    }

    Ok(())
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut total_score: usize = match part {
        Part::Part1 => 0,
        Part::Part2 => 1,
    };

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        let [blueprint_name, recipes] = &line.split(": ").collect::<Vec<_>>()[..] else {
            Err(invalid_input("Expected colon"))?
        };

        let Some(blueprint_number_text) = blueprint_name.split(' ').skip(1).next() else {
            Err(invalid_input("Expected space in blueprint name"))?
        };

        let blueprint_number = blueprint_number_text
            .parse::<usize>()
            .map_err(invalid_input)?;

        let [ore_text, clay_text, obsidian_text, geode_text] = &recipes.strip_suffix('.').ok_or_else(|| invalid_input("Expected ending ."))?.split(". ").collect::<Vec<_>>()[..] else {
            Err(invalid_input("Expected recipes"))?
        };

        let mut costs: [usize; NUM_RESOURCE_TYPES * NUM_RESOURCE_TYPES] =
            [0; NUM_RESOURCE_TYPES * NUM_RESOURCE_TYPES];

        for (robot_type, text) in [
            (ResourceType::Ore, ore_text),
            (ResourceType::Clay, clay_text),
            (ResourceType::Obsidian, obsidian_text),
            (ResourceType::Geode, geode_text),
        ] {
            let start_index = (robot_type as usize) * NUM_RESOURCE_TYPES;
            let end_index = start_index + NUM_RESOURCE_TYPES;
            parse_costs(
                &mut costs[start_index..end_index],
                text.split(" costs ").skip(1).next().ok_or_else(|| {
                    invalid_input("Invalid ore robot recipe prefix")
                })?,
            )?;
        }

        let blueprint = Blueprint { robot_costs: costs };

        match part {
            Part::Part1 => {
                let max_geodes = blueprint.max_geodes(24);
                let score = blueprint_number * max_geodes;
                println!(
                    "Blueprint {}: max_geodes={} (score={})",
                    blueprint_number, max_geodes, score
                );
                total_score += score;
            }
            Part::Part2 => {
                let max_geodes = blueprint.max_geodes(32);
                println!(
                    "Blueprint {}: max_geodes={}",
                    blueprint_number, max_geodes
                );
                total_score *= max_geodes;

                if i >= 2 {
                    break;
                }
            }
        }
    }

    println!("{}", total_score);

    Ok(())
}
