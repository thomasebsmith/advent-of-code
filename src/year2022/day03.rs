use std::collections::HashSet;
use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::iter::only_element;
use crate::part::Part;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Item(char);

impl Item {
    fn new(ch: char) -> Option<Self> {
        if !ch.is_ascii_alphabetic() {
            None
        } else {
            Some(Self(ch))
        }
    }

    fn priority(self) -> u64 {
        if 'a' <= self.0 && self.0 <= 'z' {
            (self.0 as u64) - ('a' as u64) + 1
        } else {
            (self.0 as u64) - ('A' as u64) + 27
        }
    }
}

fn rucksack(string: &str) -> io::Result<HashSet<Item>> {
    let mut items = HashSet::<Item>::new();
    for ch in string.chars() {
        items.insert(
            Item::new(ch)
                .ok_or(invalid_input("invalid character - not a letter"))?,
        );
    }
    Ok(items)
}

fn one_in_common<I: Iterator>(iter: I) -> io::Result<I::Item> {
    only_element(iter).ok_or(invalid_input("More than one element in common"))
}

fn part1<R: io::Read>(reader: io::BufReader<R>) -> io::Result<()> {
    let mut total_priority: u64 = 0;

    for line in reader.lines() {
        let line = line?;

        let first_items = rucksack(&line[..line.len() / 2])?;
        let second_items = rucksack(&line[line.len() / 2..])?;

        let common: Vec<_> = first_items.intersection(&second_items).collect();

        total_priority += one_in_common(common.into_iter())?.priority();
    }

    println!("{}", total_priority);

    Ok(())
}

fn part2<R: io::Read>(reader: io::BufReader<R>) -> io::Result<()> {
    let mut total_priority: u64 = 0;

    for lines in reader.lines().array_chunks::<3>() {
        let mut common: Option<HashSet<Item>> = None;
        for line in lines.into_iter() {
            let line = line?;
            let sack = rucksack(&line)?;
            common = match common {
                None => Some(sack),
                Some(ref common) => Some(common & &sack),
            };
        }

        total_priority += one_in_common(common.unwrap().drain())?.priority();
    }

    println!("{}", total_priority);

    Ok(())
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    match part {
        Part::Part1 => part1(reader),
        Part::Part2 => part2(reader),
    }
}
