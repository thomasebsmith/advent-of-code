use std::cmp::max;
use std::collections::HashMap;
use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::part::Part;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ValveID(usize);

#[derive(Debug)]
struct Valve {
    flow_rate: u64,
    tunnels: Vec<ValveID>,
    optimal_paths: Vec<usize>,
}

struct Plumbing {
    valves: Vec<Valve>,
    starting_valve: ValveID,
}

#[derive(Clone, Debug)]
struct MRPLocation {
    distance_to_location: usize,
    valve_id: ValveID,
}

#[derive(Clone, Debug)]
struct MRPState {
    minutes_remaining: u64,
    open_valves: Vec<bool>,
    pressure_per_minute: u64,
    released_pressure: u64,
    my_location: MRPLocation,
    elephant_location: Option<MRPLocation>,
    max_seen: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ValveOpener {
    Me,
    Elephant,
}

impl ValveOpener {
    pub fn next(self) -> Self {
        match self {
            Self::Me => Self::Elephant,
            Self::Elephant => Self::Me,
        }
    }
}

impl MRPState {
    pub fn is_open(&self, valve_id: ValveID) -> bool {
        self.open_valves[valve_id.0]
    }

    pub fn all_valves_open(&self) -> bool {
        self.open_valves.iter().all(|open| *open)
    }

    pub fn location(
        &mut self,
        opener: ValveOpener,
    ) -> Option<&mut MRPLocation> {
        match opener {
            ValveOpener::Me => Some(&mut self.my_location),
            ValveOpener::Elephant => self.elephant_location.as_mut(),
        }
    }
}

impl Plumbing {
    fn get_valve(&self, id: ValveID) -> &Valve {
        &self.valves[id.0]
    }

    // Should be called after pressure is processed but before action is taken.
    fn total_pressure_released_upper_bound(&self, state: &MRPState) -> u64 {
        let mut remaining_flow_rates = self
            .valves
            .iter()
            .enumerate()
            .filter(|(i, _)| !state.open_valves[*i])
            .map(|(_, valve)| valve.flow_rate)
            .collect::<Vec<_>>();

        remaining_flow_rates.sort_unstable();

        let mut max_pressure = state.released_pressure;
        let mut ppm = state.pressure_per_minute;

        let mut my_dist = state.my_location.distance_to_location;
        let mut elephant_dist = state
            .elephant_location
            .as_ref()
            .map(|loc| loc.distance_to_location)
            .unwrap_or(0);

        let is_elephant = state.elephant_location.is_some();

        for _ in 0..state.minutes_remaining {
            // Take my action
            if my_dist != 0 {
                my_dist -= 1;
            } else {
                if let Some(flow_rate) = remaining_flow_rates.pop() {
                    ppm += flow_rate;
                    my_dist = 1;
                }
            }

            // Take elephant's action (if applicable)
            if is_elephant {
                if elephant_dist != 0 {
                    elephant_dist -= 1;
                } else {
                    if let Some(flow_rate) = remaining_flow_rates.pop() {
                        ppm += flow_rate;
                        elephant_dist = 1;
                    }
                }
            }

            // Release pressure
            max_pressure += ppm;
        }

        max_pressure
    }

    pub fn most_released_pressure(
        &self,
        minutes: u64,
        is_elephant: bool,
    ) -> u64 {
        let elephant_location = if is_elephant {
            Some(MRPLocation {
                valve_id: self.starting_valve,
                distance_to_location: 0,
            })
        } else {
            None
        };

        let mut state = MRPState {
            minutes_remaining: minutes,
            open_valves: vec![false; self.valves.len()],
            pressure_per_minute: 0,
            released_pressure: 0,
            my_location: MRPLocation {
                valve_id: self.starting_valve,
                distance_to_location: 0,
            },
            elephant_location,
            max_seen: 0,
        };

        self.mrp_helper(&mut state, ValveOpener::Me);

        state.max_seen
    }

    fn mrp_helper(&self, state: &mut MRPState, current_actor: ValveOpener) {
        // TODO: Use a proper defer or something like that.

        // I act first. Before I act, process a minute's worth of pressure.
        if current_actor == ValveOpener::Me {
            state.minutes_remaining -= 1;
            state.released_pressure += state.pressure_per_minute;

            if state.minutes_remaining == 0 {
                state.max_seen = max(state.max_seen, state.released_pressure);
                state.minutes_remaining += 1;
                state.released_pressure -= state.pressure_per_minute;
                return;
            }

            if state.all_valves_open() {
                let result = state.released_pressure
                    + state.pressure_per_minute * state.minutes_remaining;
                state.max_seen = max(state.max_seen, result);

                if current_actor == ValveOpener::Me {
                    state.minutes_remaining += 1;
                    state.released_pressure -= state.pressure_per_minute;
                }
                return;
            }
        }

        if self.total_pressure_released_upper_bound(state) <= state.max_seen {
            if current_actor == ValveOpener::Me {
                state.minutes_remaining += 1;
                state.released_pressure -= state.pressure_per_minute;
            }
            return;
        }

        let Some(location) = state.location(current_actor) else {
            self.mrp_helper(state, current_actor.next());
            if current_actor == ValveOpener::Me {
                state.minutes_remaining += 1;
                state.released_pressure -= state.pressure_per_minute;
            }
            return;
        };

        let distance = location.distance_to_location;
        let current_valve_id = location.valve_id;

        if distance != 0 {
            // In the process of moving; continue doing so.
            state.location(current_actor).unwrap().distance_to_location -= 1;

            self.mrp_helper(state, current_actor.next());

            state.location(current_actor).unwrap().distance_to_location += 1;

            if current_actor == ValveOpener::Me {
                state.minutes_remaining += 1;
                state.released_pressure -= state.pressure_per_minute;
            }
            return;
        }

        let valve = self.get_valve(current_valve_id);
        if valve.flow_rate != 0 && !state.is_open(current_valve_id) {
            // Try opening the valve.
            state.open_valves[current_valve_id.0] = true;
            state.pressure_per_minute += valve.flow_rate;

            self.mrp_helper(state, current_actor.next());

            state.pressure_per_minute -= valve.flow_rate;
            state.open_valves[current_valve_id.0] = false;
        } else {
            // Try moving to other closed, non-zero-flow-rate valves.
            for connected_valve_index in 0..self.valves.len() {
                let connected_valve_id = ValveID(connected_valve_index);
                let connected_valve = self.get_valve(connected_valve_id);
                if !state.is_open(connected_valve_id)
                    && connected_valve.flow_rate != 0
                {
                    let location = state.location(current_actor).unwrap();
                    location.valve_id = connected_valve_id;
                    location.distance_to_location =
                        valve.optimal_paths[connected_valve_index] - 1;

                    self.mrp_helper(state, current_actor.next());
                }
            }

            let location = state.location(current_actor).unwrap();
            location.distance_to_location = 0;
            location.valve_id = current_valve_id;
        }

        if current_actor == ValveOpener::Me {
            state.minutes_remaining += 1;
            state.released_pressure -= state.pressure_per_minute;
        }
    }
}

struct ValvesParser {
    name_to_id: HashMap<String, ValveID>,
    valves: Vec<Valve>,
}

impl ValvesParser {
    pub fn new() -> Self {
        Self {
            name_to_id: HashMap::new(),
            valves: Vec::new(),
        }
    }

    fn get_or_create_valve_id(&mut self, name: String) -> ValveID {
        let get_next_id = || -> ValveID {
            let id = self.valves.len();
            self.valves.push(Valve {
                flow_rate: 0,
                tunnels: Vec::new(),
                optimal_paths: Vec::new(),
            });
            ValveID(id)
        };
        *self.name_to_id.entry(name).or_insert_with(get_next_id)
    }

    pub fn valve_id(&self, name: &str) -> ValveID {
        self.name_to_id[name]
    }

    pub fn add_valve(
        &mut self,
        name: &str,
        flow_rate: u64,
        tunnel_names: Vec<&str>,
    ) {
        let this_id = self.get_or_create_valve_id(String::from(name));
        self.valves[this_id.0].flow_rate = flow_rate;

        let tunnel_ids = tunnel_names
            .iter()
            .map(|name| self.get_or_create_valve_id(String::from(*name)))
            .collect::<Vec<_>>();
        self.valves[this_id.0].tunnels = tunnel_ids;
    }

    fn floyd_warshall(&mut self) {
        let num_valves = self.valves.len();

        for (i, valve) in self.valves.iter_mut().enumerate() {
            valve.optimal_paths = vec![usize::MAX; num_valves];
            valve.optimal_paths[i] = 0;
            for tunnel in &valve.tunnels {
                valve.optimal_paths[tunnel.0] = 1;
            }
        }

        for k in 0..num_valves {
            for i in 0..num_valves {
                for j in 0..num_valves {
                    let dist_i_j = self.valves[i].optimal_paths[j];
                    let dist_i_k = self.valves[i].optimal_paths[k];
                    let dist_k_j = self.valves[k].optimal_paths[j];
                    let Some(sum) = dist_i_k.checked_add(dist_k_j) else {
                        continue;
                    };
                    if dist_i_j > sum {
                        self.valves[i].optimal_paths[j] = sum;
                    }
                }
            }
        }
    }

    pub fn into_valves(mut self) -> Vec<Valve> {
        self.floyd_warshall();
        self.valves
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut valves_parser = ValvesParser::new();

    for line in reader.lines() {
        let line = line?;

        let [valve_text, tunnels_text] = &line.split("; ").collect::<Vec<_>>()[..] else {
            Err(invalid_input("Expected \"; \""))?
        };

        let [name_text, flow_rate_text] = &valve_text.split(" has flow rate=").collect::<Vec<_>>()[..] else {
            Err(invalid_input("Expected \" has flow rate=\""))?
        };

        let valve_name = name_text.strip_prefix("Valve ").ok_or_else(|| {
            invalid_input("Expected valve name to begin with \"Valve \"")
        })?;
        let flow_rate = flow_rate_text.parse::<u64>().map_err(invalid_input)?;
        let tunnel_names = tunnels_text
            .split(' ')
            .skip(4)
            .map(|string| string.strip_suffix(',').unwrap_or(string))
            .collect::<Vec<_>>();

        valves_parser.add_valve(valve_name, flow_rate, tunnel_names);
    }

    let starting_valve = valves_parser.valve_id("AA");
    let plumbing = Plumbing {
        valves: valves_parser.into_valves(),
        starting_valve,
    };

    let mrp = match part {
        Part::Part1 => plumbing.most_released_pressure(30, false),
        Part::Part2 => plumbing.most_released_pressure(26, true),
    };

    println!("{}", mrp);

    Ok(())
}
