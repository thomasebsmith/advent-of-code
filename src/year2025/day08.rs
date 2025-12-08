use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::io;
use std::mem::swap;

use crate::errors::invalid_input;
use crate::parse::{lines, parse_all};
use crate::part::Part;

type Num = i64;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
struct BoxID(usize);

type Circuit = HashSet<BoxID>;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
struct CircuitID(usize);

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
struct Position3D {
    pub x: Num,
    pub y: Num,
    pub z: Num,
}

impl Position3D {
    pub fn squared_dist(self, other: Self) -> Num {
        (self.x - other.x).pow(2)
            + (self.y - other.y).pow(2)
            + (self.z - other.z).pow(2)
    }
}

#[derive(Clone, PartialEq, Eq)]
struct HeapEntry {
    squared_distance: Num,
    id_1: BoxID,
    id_2: BoxID,
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        other.squared_distance.cmp(&self.squared_distance)
    }
}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct Playground {
    box_positions: Vec<Position3D>,
    shortest_maybe_unconnected: BinaryHeap<HeapEntry>,
    circuits: Vec<Circuit>,
    num_circuits: usize,
    box_to_circuit: HashMap<BoxID, CircuitID>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum ConnectResult {
    AlreadyConnected,
    Connection(BoxID, BoxID),
    EverythingConnected,
}

impl Playground {
    fn new<R: io::Read>(reader: io::BufReader<R>) -> io::Result<Self> {
        let box_positions = lines(reader)?
            .map(|line| {
                let components = parse_all::<_, Num>(line.split(','))?;
                let &[x, y, z] = &components[..] else {
                    return Err(invalid_input("Expected x,y,z coordinate"));
                };
                Ok(Position3D { x, y, z })
            })
            .collect::<io::Result<Vec<_>>>()?;

        if box_positions.len() <= 1 {
            return Err(invalid_input("Expected at least 2 circuit boxes"));
        }

        let mut shortest_maybe_unconnected =
            BinaryHeap::<HeapEntry>::with_capacity(
                box_positions.len() * (box_positions.len() - 1) / 2,
            );
        for i in 0..box_positions.len() {
            for j in i + 1..box_positions.len() {
                let squared_distance =
                    box_positions[i].squared_dist(box_positions[j]);
                let entry = HeapEntry {
                    squared_distance,
                    id_1: BoxID(i),
                    id_2: BoxID(j),
                };
                shortest_maybe_unconnected.push(entry);
            }
        }

        let mut circuits = Vec::<Circuit>::with_capacity(box_positions.len());
        let mut box_to_circuit =
            HashMap::<BoxID, CircuitID>::with_capacity(box_positions.len());
        for i in 0..box_positions.len() {
            let mut circuit = Circuit::with_capacity(1);
            circuit.insert(BoxID(i));
            circuits.push(circuit);
            box_to_circuit.insert(BoxID(i), CircuitID(i));
        }

        let num_circuits = circuits.len();

        Ok(Self {
            box_positions,
            shortest_maybe_unconnected,
            circuits,
            num_circuits,
            box_to_circuit,
        })
    }

    fn merge_circuits(&mut self, id_1: CircuitID, id_2: CircuitID) -> bool {
        if id_1 == id_2 {
            return false;
        }

        // Move everything from id_2 to id_1
        let mut moving_from_circuit_2 = Circuit::new();
        swap(&mut moving_from_circuit_2, &mut self.circuits[id_2.0]);

        for &box_id in moving_from_circuit_2.iter() {
            self.box_to_circuit.insert(box_id, id_1);
        }

        if !moving_from_circuit_2.is_empty() {
            self.num_circuits -= 1;
        }

        self.circuits[id_1.0].extend(moving_from_circuit_2);

        true
    }

    fn connect_closest_pair(&mut self) -> ConnectResult {
        if self.num_circuits <= 1 {
            return ConnectResult::EverythingConnected;
        }

        let Some(entry) = self.shortest_maybe_unconnected.pop() else {
            return ConnectResult::EverythingConnected;
        };

        let circuit_1 = *self.box_to_circuit.get(&entry.id_1).unwrap();
        let circuit_2 = *self.box_to_circuit.get(&entry.id_2).unwrap();
        if self.merge_circuits(circuit_1, circuit_2) {
            ConnectResult::Connection(entry.id_1, entry.id_2)
        } else {
            ConnectResult::AlreadyConnected
        }
    }

    fn circuit_product(&self) -> usize {
        let mut circuit_sizes = self
            .circuits
            .iter()
            .map(|circuit| circuit.len())
            .collect::<Vec<_>>();
        circuit_sizes.sort();
        if circuit_sizes.len() < 3 {
            0
        } else {
            circuit_sizes[circuit_sizes.len() - 3]
                * circuit_sizes[circuit_sizes.len() - 2]
                * circuit_sizes[circuit_sizes.len() - 1]
        }
    }

    fn box_position(&self, id: BoxID) -> Option<Position3D> {
        self.box_positions.get(id.0).map(|position| *position)
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut playground = Playground::new(reader)?;

    let result = match part {
        Part::Part1 => {
            for _ in 0..1000 {
                playground.connect_closest_pair();
            }
            playground.circuit_product() as i64
        }
        Part::Part2 => {
            let mut last_connection: Option<(BoxID, BoxID)> = None;
            loop {
                match playground.connect_closest_pair() {
                    ConnectResult::AlreadyConnected => { /* ignore */ }
                    ConnectResult::Connection(id_1, id_2) => {
                        last_connection = Some((id_1, id_2));
                    }
                    ConnectResult::EverythingConnected => {
                        break;
                    }
                }
            }

            let Some((id_1, id_2)) = last_connection else {
                return Err(invalid_input("No connections could be made"));
            };

            playground.box_position(id_1).unwrap().x
                * playground.box_position(id_2).unwrap().x
        }
    };

    println!("{result}");

    Ok(())
}
