use std::collections::{BTreeSet, HashMap};
use std::io;

use crate::errors::invalid_input;
use crate::parse::{lines, paragraphs};
use crate::part::Part;

type WireName = String;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum GateType {
    And,
    Or,
    Xor,
}

#[derive(Clone, Debug)]
struct Gate {
    gate_type: GateType,
    input1: WireName,
    input2: WireName,
}

impl Gate {
    fn compute(&self, input1: bool, input2: bool) -> bool {
        match self.gate_type {
            GateType::And => input1 & input2,
            GateType::Or => input1 | input2,
            GateType::Xor => input1 ^ input2,
        }
    }
}

#[derive(Clone, Debug)]
enum WireSource {
    Gate(Gate),
    Constant(bool),
}

struct Simulation {
    wire_sources: HashMap<WireName, WireSource>,
    wire_value_cache: HashMap<WireName, bool>,
    z_wire_names: BTreeSet<WireName>,
}

impl Simulation {
    fn new() -> Self {
        Self {
            wire_sources: HashMap::new(),
            wire_value_cache: HashMap::new(),
            z_wire_names: BTreeSet::new(),
        }
    }

    fn add_wire(&mut self, name: WireName, source: WireSource) {
        if name.starts_with('z') {
            self.z_wire_names.insert(name.clone());
        }
        self.wire_sources.insert(name, source);
    }

    fn compute(&mut self, wire_name: &WireName) -> bool {
        if let Some(&value) = self.wire_value_cache.get(wire_name) {
            return value;
        }

        let result = match self.wire_sources.get(wire_name).unwrap() {
            WireSource::Gate(gate) => {
                let gate = gate.clone();
                gate.compute(
                    self.compute(&gate.input1),
                    self.compute(&gate.input2),
                )
            }
            WireSource::Constant(value) => *value,
        };
        self.wire_value_cache.insert(wire_name.to_owned(), result);
        result
    }

    fn z_number(&mut self) -> u64 {
        let mut result = 0u64;
        for name in self.z_wire_names.clone().into_iter().rev() {
            result <<= 1;
            result += u64::from(self.compute(&name));
        }
        result
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let all_paragraphs = paragraphs(lines(reader)?).collect::<Vec<_>>();
    if all_paragraphs.len() != 2 {
        return Err(invalid_input("Expected two sections"));
    }

    let mut simulation = Simulation::new();

    for line in &all_paragraphs[0] {
        let &[name, value_str] = &line.split(": ").collect::<Vec<_>>()[..]
        else {
            return Err(invalid_input("Expected name: value"));
        };
        let value_u64: u64 = value_str.parse().map_err(invalid_input)?;
        let value = value_u64 != 0;

        simulation.add_wire(name.to_owned(), WireSource::Constant(value));
    }

    for line in &all_paragraphs[1] {
        let &[input1, gate_type, input2, _, output] =
            &line.split(" ").collect::<Vec<_>>()[..]
        else {
            return Err(invalid_input("Expected name OP name -> name"));
        };
        let gate_type = match gate_type {
            "AND" => GateType::And,
            "OR" => GateType::Or,
            "XOR" => GateType::Xor,
            _ => return Err(invalid_input("Unknown gate type")),
        };
        simulation.add_wire(
            output.to_owned(),
            WireSource::Gate(Gate {
                gate_type,
                input1: input1.to_owned(),
                input2: input2.to_owned(),
            }),
        );
    }

    if part == Part::Part2 {
        /*
        for x_num in 0..45 {
            let name = format!("x{x_num:02}");

            simulation.wire_value_cache.clear();
            for (wire_name, source) in &mut simulation.wire_sources {
                let WireSource::Constant(ref mut value) = source else {
                    continue;
                };

                *value = false;
            }
            simulation.wire_sources.insert(name, WireSource::Constant(true));
            let actual = simulation.z_number();
            let expected = 1 << x_num;
            if actual != expected {
                println!("x bit {x_num} is off. expected {expected:b} but saw {actual:b}");
            }
        }

        for y_num in 0..45 {
            let name = format!("y{y_num:02}");

            simulation.wire_value_cache.clear();
            for (wire_name, source) in &mut simulation.wire_sources {
                let WireSource::Constant(ref mut value) = source else {
                    continue;
                };

                *value = false;
            }
            simulation.wire_sources.insert(name, WireSource::Constant(true));
            let actual = simulation.z_number();
            let expected = 1 << y_num;
            if actual != expected {
                println!("y bit {y_num} is off. expected {expected:b} but saw {actual:b}");
            }
        }*/
        // TODO: check 45 and 0 and 1
        let mut expected_basic_add: Option<WireName> = None;
        let mut expected_prev_carry: Option<WireName> = None;

        for z_num in (2..45).rev() {
            let name = format!("z{z_num:02}");
            let x_name = format!("x{z_num:02}");
            let y_name = format!("y{z_num:02}");

            let WireSource::Gate(gate) =
                simulation.wire_sources.get(&name).unwrap()
            else {
                println!("Expected {name} to be gate-sourced");
                expected_basic_add = None;
                expected_prev_carry = None;
                continue;
            };
            if gate.gate_type != GateType::Xor {
                println!("Expected _ XOR _ -> {name}");
                expected_basic_add = None;
                expected_prev_carry = None;
                continue;
            }

            let WireSource::Gate(gate1) =
                simulation.wire_sources.get(&gate.input1).unwrap()
            else {
                println!("Expected {name} to have gates nested");
                expected_basic_add = None;
                expected_prev_carry = None;
                continue;
            };
            let WireSource::Gate(gate2) =
                simulation.wire_sources.get(&gate.input2).unwrap()
            else {
                println!("Expected {name} to have gates nested");
                expected_basic_add = None;
                expected_prev_carry = None;
                continue;
            };
            let (gate_add, gate_prev_carry) =
                if gate1.gate_type == GateType::Xor {
                    (gate1, gate2)
                } else {
                    (gate2, gate1)
                };
            let (gate_add_name, gate_prev_carry_name) =
                if gate1.gate_type == GateType::Xor {
                    (&gate.input1, &gate.input2)
                } else {
                    (&gate.input2, &gate.input1)
                };
            if let Some(ref expected_basic_add) = expected_basic_add {
                if *gate_add_name != *expected_basic_add {
                    println!(
                        "Failed expectation from {} that {expected_basic_add} is the basic add for {z_num}",
                        z_num + 1
                    );
                    continue;
                }
            }
            if let Some(ref expected_prev_carry) = expected_prev_carry {
                if *gate_prev_carry_name != *expected_prev_carry {
                    println!(
                        "Failed expectation from {} that {expected_prev_carry} is the previous carry for {z_num}",
                        z_num + 1
                    );
                    continue;
                }
            }
            expected_basic_add = None;
            expected_prev_carry = None;
            if gate_add.gate_type != GateType::Xor {
                println!("Expected GA {gate_add_name} to be XOR");
                continue;
            }
            if gate_prev_carry.gate_type != GateType::Or {
                println!(
                    "Expected GPC {gate_prev_carry_name} to be OR (on bit {z_num})"
                );
                continue;
            }
            if gate_add.input1 != x_name && gate_add.input2 != x_name {
                println!("Expected GA {gate_add_name} to have {x_name} input");
                continue;
            }
            if gate_add.input1 != y_name && gate_add.input2 != y_name {
                println!("Expected GA {gate_add_name} to have {y_name} input");
                continue;
            }
            let WireSource::Gate(gate3) = simulation
                .wire_sources
                .get(&gate_prev_carry.input1)
                .unwrap()
            else {
                println!(
                    "Expected GPC {gate_prev_carry_name} to have gates nested"
                );
                continue;
            };
            let WireSource::Gate(gate4) = simulation
                .wire_sources
                .get(&gate_prev_carry.input2)
                .unwrap()
            else {
                println!(
                    "Expected GPC {gate_prev_carry_name} to have gates nested"
                );
                continue;
            };
            let (gate_prev_basic_carry, gate_prev_fancy_carry) =
                if gate3.input1.starts_with('x')
                    || gate3.input1.starts_with('y')
                {
                    (gate3, gate4)
                } else {
                    (gate4, gate3)
                };
            let (gpbc_name, gpfc_name) = if gate3.input1.starts_with('x')
                || gate3.input1.starts_with('y')
            {
                (&gate_prev_carry.input1, &gate_prev_carry.input2)
            } else {
                (&gate_prev_carry.input2, &gate_prev_carry.input1)
            };
            if gate_prev_basic_carry.gate_type != GateType::And {
                println!(
                    "Expected GPBC {gpbc_name} to be AND (on bit {z_num})"
                );
                continue;
            }
            if gate_prev_fancy_carry.gate_type != GateType::And {
                println!(
                    "Expected GPFC {gpfc_name} to be AND (on bit {z_num})"
                );
                continue;
            }
            let prev_x_name = format!("x{:02}", z_num - 1);
            let prev_y_name = format!("y{:02}", z_num - 1);
            if gate_prev_basic_carry.input1 != prev_x_name
                && gate_prev_basic_carry.input2 != prev_x_name
            {
                println!(
                    "Expected {gate_prev_basic_carry:?} to have {prev_x_name} input"
                );
                continue;
            }
            if gate_prev_basic_carry.input1 != prev_y_name
                && gate_prev_basic_carry.input2 != prev_y_name
            {
                println!(
                    "Expected {gate_prev_basic_carry:?} to have {prev_y_name} input"
                );
                continue;
            }

            let WireSource::Gate(gate5) = simulation
                .wire_sources
                .get(&gate_prev_fancy_carry.input1)
                .unwrap()
            else {
                println!(
                    "Expected {gate_prev_fancy_carry:?} to have nested gates"
                );
                continue;
            };
            if gate5.input1.ends_with(&format!("{:02}", z_num - 1)) {
                expected_basic_add =
                    Some(gate_prev_fancy_carry.input1.to_owned());
                expected_prev_carry =
                    Some(gate_prev_fancy_carry.input2.to_owned());
            } else {
                expected_prev_carry =
                    Some(gate_prev_fancy_carry.input1.to_owned());
                expected_basic_add =
                    Some(gate_prev_fancy_carry.input2.to_owned());
            }
        }
    }

    let result = simulation.z_number();

    println!("{result}");

    Ok(())
}
