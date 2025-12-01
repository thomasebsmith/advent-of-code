use std::collections::{HashMap, HashSet};
use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::part::Part;

fn narrow(
    connections: &HashMap<String, HashSet<String>>,
    groups: HashSet<Vec<String>>,
) -> HashSet<Vec<String>> {
    let mut result = HashSet::<Vec<String>>::new();
    for group in groups {
        let mut new_set: Option<HashSet<String>> = None;
        for member in &group {
            if let Some(ref mut new_set) = new_set {
                *new_set = &*new_set & connections.get(member).unwrap();
            } else {
                new_set = Some(connections.get(member).unwrap().clone());
            }
        }
        for new_group_member in new_set.unwrap() {
            let mut new_vec = group.clone();
            new_vec.push(new_group_member);
            new_vec.sort();
            result.insert(new_vec);
        }
    }
    result
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut connections = HashMap::<String, HashSet<String>>::new();
    for line in reader.lines() {
        let line = line?;
        let comps = line.split("-").collect::<Vec<_>>();
        if comps.len() != 2 {
            return Err(invalid_input("Expected comp1-comp2"));
        };
        connections
            .entry(comps[0].to_owned())
            .or_insert_with(HashSet::new)
            .insert(comps[1].to_owned());
        connections
            .entry(comps[1].to_owned())
            .or_insert_with(HashSet::new)
            .insert(comps[0].to_owned());
    }

    let mut groups = HashSet::<Vec<String>>::new();
    for (comp1, connected_to_comp) in connections.iter() {
        for comp2 in connected_to_comp {
            let mut pair = vec![comp1.to_owned(), comp2.to_owned()];
            pair.sort();
            groups.insert(pair);
        }
    }

    match part {
        Part::Part1 => {
            groups = narrow(&connections, groups);
            let result = groups
                .into_iter()
                .filter(|trio| trio.iter().any(|name| name.starts_with('t')))
                .count();
            println!("{result}");
        }
        Part::Part2 => {
            while groups.len() > 1 {
                groups = narrow(&connections, groups);
            }
            if groups.is_empty() {
                return Err(invalid_input("No single LAN party found"));
            }
            let group = groups.into_iter().next().unwrap();
            let password = group.join(",");
            println!("{password}");
        }
    }

    Ok(())
}
