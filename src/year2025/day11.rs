use std::collections::{HashMap, HashSet};
use std::io;

use crate::errors::invalid_input;
use crate::parse::lines;
use crate::part::Part;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct DeviceID(String);

struct Device {
    connections: Vec<DeviceID>,
}

struct ServerRack {
    devices: HashMap<DeviceID, Device>,
    predecessors: HashMap<DeviceID, HashSet<DeviceID>>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ResultsTableKey {
    start: DeviceID,
    end: DeviceID,
    passing_through: Vec<DeviceID>,
}

impl ResultsTableKey {
    fn new(
        start: DeviceID,
        end: DeviceID,
        passing_through: &HashSet<DeviceID>,
    ) -> Self {
        let mut passing_through_vec =
            passing_through.iter().cloned().collect::<Vec<_>>();
        passing_through_vec.sort();
        Self {
            start,
            end,
            passing_through: passing_through_vec,
        }
    }
}

impl ServerRack {
    fn new<R: io::Read>(reader: io::BufReader<R>) -> io::Result<Self> {
        let devices = lines(reader)?
            .map(|line| {
                let &[device_name, connections_str] =
                    &line.split(": ").collect::<Vec<_>>()[..]
                else {
                    return Err(invalid_input("Expected name: connections"));
                };
                let connections = connections_str
                    .split(' ')
                    .map(|string| DeviceID(string.to_owned()))
                    .collect::<Vec<_>>();
                Ok((DeviceID(device_name.to_owned()), Device { connections }))
            })
            .collect::<io::Result<HashMap<_, _>>>()?;

        let mut predecessors = HashMap::<DeviceID, HashSet<DeviceID>>::new();
        for (device_id, device) in devices.iter() {
            for output in device.connections.iter() {
                predecessors
                    .entry(output.clone())
                    .or_default()
                    .insert(device_id.clone());
            }
        }

        Ok(Self {
            devices,
            predecessors,
        })
    }

    fn num_paths_between(
        &self,
        start: &DeviceID,
        end: &DeviceID,
        passing_through: &HashSet<DeviceID>,
    ) -> usize {
        if !self.devices.contains_key(start)
            || !self.predecessors.contains_key(end)
        {
            return 0;
        }
        for id in passing_through.iter() {
            if !self.devices.contains_key(id) {
                return 0;
            }
        }

        let mut map = HashMap::new();
        self.num_paths_between_helper(start, end, passing_through, &mut map)
    }

    fn num_paths_between_helper(
        &self,
        start: &DeviceID,
        end: &DeviceID,
        passing_through: &HashSet<DeviceID>,
        results_table: &mut HashMap<ResultsTableKey, usize>,
    ) -> usize {
        let key =
            ResultsTableKey::new(start.clone(), end.clone(), passing_through);
        if let Some(&result) = results_table.get(&key) {
            return result;
        }

        let result: usize = if start == end && passing_through.is_empty() {
            if passing_through.is_empty() { 1 } else { 0 }
        } else if !self.predecessors.contains_key(end) {
            0
        } else {
            self.predecessors
                .get(end)
                .unwrap()
                .iter()
                .map(|predecessor_id| {
                    let mut new_passing_through = passing_through.clone();
                    new_passing_through.remove(predecessor_id);
                    self.num_paths_between_helper(
                        start,
                        predecessor_id,
                        &new_passing_through,
                        results_table,
                    )
                })
                .sum()
        };

        results_table.insert(key, result);

        result
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let server_rack = ServerRack::new(reader)?;

    let (start, end, passing_through) = match part {
        Part::Part1 => ("you", "out", vec![]),
        Part::Part2 => ("svr", "out", vec!["dac", "fft"]),
    };
    let result = server_rack.num_paths_between(
        &DeviceID(start.to_owned()),
        &DeviceID(end.to_owned()),
        &passing_through
            .into_iter()
            .map(|id| DeviceID(id.to_owned()))
            .collect(),
    );
    println!("{result}");

    Ok(())
}
