use std::collections::{HashMap, VecDeque};
use std::io;

use crate::errors::invalid_input;
use crate::parse::lines;
use crate::part::Part;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum Pulse {
    Low,
    High,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ModuleState {
    FlipFlop(bool),
    Conjunction(HashMap<String, Pulse>),
    Broadcast,
    SandMover(bool),
}

impl ModuleState {
    fn _debug_compare(&self, key: &str, new_state: &Self) {
        if self != new_state {
            match self {
                Self::FlipFlop(was_on) => {
                    let Self::FlipFlop(is_on) = new_state else {
                        panic!("Invalid compare");
                    };
                    println!("{key} flipped from {was_on} to {is_on}");
                }
                Self::Conjunction(ref old_inputs) => {
                    let Self::Conjunction(ref new_inputs) = new_state else {
                        panic!("Invalid compare");
                    };
                    // TODO should I print more often here??
                    let old_was_low =
                        old_inputs.values().all(|v| *v == Pulse::High);
                    let new_is_low =
                        new_inputs.values().all(|v| *v == Pulse::High);
                    if old_was_low != new_is_low {
                        println!("{key} changed from low={old_was_low} to low={new_is_low}",);
                    }
                }
                _ => panic!("Unexpected discrepancy"),
            }
        }
    }
}

#[derive(Clone, Debug)]
struct Module {
    state: ModuleState,
    destinations: Vec<String>,
}

impl Module {
    fn from_line(line: &str) -> io::Result<(String, Self)> {
        let [description, destinations_str] =
            line.split(" -> ").collect::<Vec<_>>()[..]
        else {
            return Err(invalid_input("Expected one ->"));
        };

        let (state, name) = match description.chars().next() {
            None => {
                return Err(invalid_input("Expected content left of ->"));
            }
            Some('%') => (ModuleState::FlipFlop(false), &description[1..]),
            Some('&') => {
                (ModuleState::Conjunction(HashMap::new()), &description[1..])
            }
            _ => {
                if description == "broadcaster" {
                    (ModuleState::Broadcast, description)
                } else {
                    return Err(invalid_input("Invalid module description"));
                }
            }
        };
        let name = name.to_owned();
        let destinations = destinations_str
            .split(", ")
            .map(str::to_owned)
            .collect::<Vec<_>>();

        Ok((
            name,
            Self {
                state,
                destinations,
            },
        ))
    }

    fn handle_pulse(&mut self, pulse: Pulse, source: &str) -> Option<Pulse> {
        match self.state {
            ModuleState::FlipFlop(ref mut is_on) => match pulse {
                Pulse::Low => {
                    *is_on = !*is_on;
                    Some(if *is_on { Pulse::High } else { Pulse::Low })
                }
                Pulse::High => None,
            },
            ModuleState::Conjunction(ref mut inputs) => {
                inputs.insert(source.to_owned(), pulse);
                Some(
                    if inputs
                        .values()
                        .all(|last_pulse| *last_pulse == Pulse::High)
                    {
                        Pulse::Low
                    } else {
                        Pulse::High
                    },
                )
            }
            ModuleState::Broadcast => Some(pulse),
            ModuleState::SandMover(ref mut is_on) => {
                if pulse == Pulse::Low {
                    println!("Sand mover on!"); // TODO
                    *is_on = true;
                }
                None
            }
        }
    }
}

struct Network {
    modules: HashMap<String, Module>,
    num_low_pulses: usize,
    num_high_pulses: usize,
}

impl Network {
    fn from_lines(lines: impl Iterator<Item = String>) -> io::Result<Self> {
        let mut modules = lines
            .into_iter()
            .map(|line| Module::from_line(&line))
            .collect::<io::Result<HashMap<_, _>>>()?;

        // TODO: This is a bit of a hack because conjunction modules are
        // annoying
        let mut destination_to_source_map =
            HashMap::<String, Vec<String>>::new();
        for (name, module) in &modules {
            for destination in &module.destinations {
                destination_to_source_map
                    .entry(destination.clone())
                    .or_insert_with(Vec::new)
                    .push(name.clone());
            }
        }
        for (name, module) in &mut modules {
            if let ModuleState::Conjunction(ref mut inputs) = module.state {
                if let Some(sources) = destination_to_source_map.get(name) {
                    for source in sources {
                        inputs.insert(source.clone(), Pulse::Low);
                    }
                }
            }
        }

        if modules.contains_key("rx") {
            return Err(invalid_input(
                "rx module cannot be specified in input",
            ));
        }

        modules.insert(
            "rx".to_owned(),
            Module {
                state: ModuleState::SandMover(false),
                destinations: Vec::new(),
            },
        );

        Ok(Self {
            modules,
            num_low_pulses: 0,
            num_high_pulses: 0,
        })
    }

    fn send_pulse(
        &mut self,
        source: &str,
        destination: &str,
        pulse: Pulse,
        button_presses: usize,
    ) {
        let mut queue = VecDeque::<(String, String, Pulse)>::new();
        queue.push_back((source.to_owned(), destination.to_owned(), pulse));

        while let Some((source, destination, pulse)) = queue.pop_front() {
            // println!("Processing pulse {:#?} from {source} to {destination}",
            // pulse); // TODO TODO
            if pulse == Pulse::Low && source == "zp" {
                println!("low from zp during press {button_presses}");
            }

            *match pulse {
                Pulse::Low => &mut self.num_low_pulses,
                Pulse::High => &mut self.num_high_pulses,
            } += 1;

            let Some(module) = self.modules.get_mut(&destination) else {
                // Ignore references to invalid modules
                //println!("Skipping pulse because {destination} doesn't
                // exist"); // TODO
                continue;
            };

            let Some(resulting_pulse) = module.handle_pulse(pulse, &source)
            else {
                // If no resulting pulse, move on
                //println!("No output pulse from {destination}"); // TODO
                continue;
            };

            for new_destination in &module.destinations {
                queue.push_back((
                    destination.clone(),
                    new_destination.to_owned(),
                    resulting_pulse,
                ));
            }
        }
    }

    fn sand_mover_is_on(&self) -> bool {
        // TODO refactor constant
        match self.modules.get("rx").unwrap().state {
            ModuleState::SandMover(true) => true,
            _ => false,
        }
    }
}

fn _debug_compare(
    old_modules: &HashMap<String, Module>,
    new_modules: &HashMap<String, Module>,
) {
    for key in old_modules.keys() {
        let old_mod = old_modules.get(key).unwrap();
        let new_mod = new_modules.get(key).unwrap();
        old_mod.state._debug_compare(key, &new_mod.state);
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut network = Network::from_lines(lines(reader)?)?;

    match part {
        Part::Part1 => {
            for i in 0..1000 {
                network.send_pulse("", "broadcaster", Pulse::Low, i + 1); // TODO: use a constant
            }
            println!("{}", network.num_high_pulses * network.num_low_pulses);
        }
        Part::Part2 => {
            let mut button_presses: usize = 0;
            //let mut last_pz = false;
            while !network.sand_mover_is_on() {
                // TODO
                //let old_modules = network.modules.clone();
                network.send_pulse(
                    "",
                    "broadcaster",
                    Pulse::Low,
                    button_presses + 1,
                ); // TODO dup
                button_presses += 1;
                //println!("After {button_presses} presses:");
                //_debug_compare(&old_modules, &network.modules);
                //println!();

                // TODO
                /*let check_conj = |name: &str, want: Pulse| {
                    if let Some(Module { state: ModuleState::Conjunction(ref inputs), ..}) = network.modules.get(name) {
                        let is_outputting_low = inputs.values().all(|li| *li == Pulse::High);
                        match (is_outputting_low, want) {
                            (true, Pulse::Low) => println!("{name} is outputting high after {button_presses} presses"),
                            (false, Pulse::High) => println!("{name} is outputting low after {button_presses} presses"),
                            _ => (),
                        }
                    }
                };
                check_conj("nx", Pulse::Low);*/
                // check_conj("bh", Pulse::High);
                // check_conj("dl", Pulse::High);
                // check_conj("ns", Pulse::High);
                // check_conj("vd", Pulse::High);
                /*let check_ff = |name: &str, last: &mut bool| {
                    if let Some(Module { state: ModuleState::FlipFlop(is_on), ..}) = network.modules.get(name) {
                        if last != is_on {
                            println!("{name} is_on={is_on} after {button_presses} presses");
                            *last = *is_on;
                        }
                    }
                };
                check_ff("jl", &mut last_pz);*/
            }
            println!("{button_presses}");
        }
    }

    Ok(())
}
