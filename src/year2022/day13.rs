use std::cmp::Ordering;
use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::part::Part;

#[derive(Clone)]
enum PacketData {
    Integer(u64),
    List(Vec<Self>),
}

impl PacketData {
    pub fn from_packet_line(line: &str) -> io::Result<Self> {
        if line.is_empty() {
            Err(invalid_input("Empty line"))?
        }

        // Add extra depth to the stack to make top-level parsing easier
        let mut stack: Vec<Vec<Box<Self>>> = vec![vec![]];
        let mut number_start_index: Option<usize> = None;

        for (i, ch) in line.chars().enumerate() {
            match ch {
                '[' => {
                    stack.push(Vec::new());
                }
                ']' | ',' => {
                    if let Some(start_index) = number_start_index {
                        let text = &line[start_index..i];
                        stack
                            .last_mut()
                            .ok_or_else(|| {
                                invalid_input(
                                    "Cannot finish number - nothing to add to",
                                )
                            })?
                            .push(Box::new(Self::Integer(
                                text.parse().map_err(invalid_input)?,
                            )));
                        number_start_index = None;
                    }

                    if ch == ']' {
                        let Some(list) = stack.pop() else {
                            Err(invalid_input("Very unmatched ]"))?
                        };
                        let Some(to_add_to) = stack.last_mut() else {
                            Err(invalid_input("Unmatched ]"))?
                        };
                        to_add_to.push(Box::new(Self::List(
                            list.into_iter().map(|x| *x).collect(),
                        )));
                    }
                }
                _ => {
                    if number_start_index.is_none() {
                        number_start_index = Some(i);
                    }
                }
            }
        }

        if stack.len() != 1 {
            Err(invalid_input("Bad stack length at end"))?
        }

        if stack[0].len() != 1 {
            Err(invalid_input("Multiple packets on same line"))?
        }

        Ok(*stack[0].pop().unwrap())
    }
}

impl Ord for PacketData {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Integer(i1), Self::Integer(i2)) => i1.cmp(&i2),
            (Self::List(v1), Self::List(v2)) => v1.cmp(v2),
            (Self::List(v1), Self::Integer(_)) => {
                v1.iter().cmp(std::iter::once(other))
            }
            (Self::Integer(_), Self::List(v2)) => {
                std::iter::once(self).cmp(v2.iter())
            }
        }
    }
}

impl PartialOrd for PacketData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PacketData {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for PacketData {}

fn part_1<R: io::Read>(reader: io::BufReader<R>) -> io::Result<()> {
    let mut packets = Vec::<PacketData>::new();
    let mut index: usize = 1;
    let mut indices_sum: usize = 0;

    for line in reader.lines() {
        let line = line?;

        if line == "" {
            if packets.len() != 2 {
                Err(invalid_input("More than two unseparated packets"))?
            }
            if packets[0] <= packets[1] {
                indices_sum += index;
            }
            packets.clear();
            index += 1;
        } else {
            packets.push(PacketData::from_packet_line(&line)?);
        }
    }

    println!("{}", indices_sum);

    Ok(())
}

fn part_2<R: io::Read>(reader: io::BufReader<R>) -> io::Result<()> {
    let mut packets = Vec::<PacketData>::new();

    let divider_packet_1 =
        PacketData::List(vec![PacketData::List(vec![PacketData::Integer(2)])]);
    packets.push(divider_packet_1.clone());
    let divider_packet_2 =
        PacketData::List(vec![PacketData::List(vec![PacketData::Integer(6)])]);
    packets.push(divider_packet_2.clone());

    for line in reader.lines() {
        let line = line?;

        if line != "" {
            packets.push(PacketData::from_packet_line(&line)?);
        }
    }

    packets.sort();

    let key = packets
        .into_iter()
        .enumerate()
        .filter(|(_, packet)| {
            *packet == divider_packet_1 || *packet == divider_packet_2
        })
        .map(|(i, _)| i + 1)
        .product::<usize>();

    println!("{}", key);

    Ok(())
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let func = match part {
        Part::Part1 => part_1,
        Part::Part2 => part_2,
    };
    func(reader)
}
