use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::part::Part;

type FileID = i64;

#[derive(Debug)]
struct Disk {
    blocks: Vec<Option<FileID>>,
}

impl Disk {
    fn new(line: &str) -> io::Result<Self> {
        let mut reading_file_length = true;
        let mut file_id: FileID = 0;
        let mut blocks: Vec<Option<FileID>> = Vec::new();
        for ch in line.chars() {
            if !ch.is_ascii_digit() {
                return Err(invalid_input("Non-ascii digit found"));
            }

            let fill_value = if reading_file_length {
                let this_file_id = file_id;
                file_id += 1;
                Some(this_file_id)
            } else {
                None
            };

            blocks.resize(
                blocks.len() + ch.to_digit(10).unwrap() as usize,
                fill_value,
            );
            reading_file_length = !reading_file_length;
        }

        Ok(Self { blocks })
    }

    fn compact(&mut self) {
        if self.blocks.is_empty() {
            return;
        }

        let mut i: usize = 0;
        let mut j: usize = self.blocks.len() - 1;

        while j > i && self.blocks[j].is_none() {
            j -= 1;
        }
        while i < j && self.blocks[i].is_some() {
            i += 1;
        }

        while i < j {
            self.blocks.swap(i, j);

            while j > i && self.blocks[j].is_none() {
                j -= 1;
            }
            while i < j && self.blocks[i].is_some() {
                i += 1;
            }
        }
    }

    fn move_range(
        &mut self,
        empty_space_indices_by_length: &mut HashMap<
            usize,
            BinaryHeap<Reverse<usize>>,
        >,
        start_index: usize,
        length: usize,
    ) {
        let Some((size, Reverse(dest_index))) = empty_space_indices_by_length
            .iter()
            .filter_map(|(&size, heap)| {
                if size >= length {
                    heap.peek().map(|index| (size, *index))
                } else {
                    None
                }
            })
            .min_by_key(|(_, Reverse(index))| *index)
        else {
            return;
        };

        if dest_index + length > start_index {
            return;
        }

        let (before_start, start) = self.blocks.split_at_mut(start_index);
        let start = &mut start[..length];
        let dest = &mut before_start[dest_index..(dest_index + length)];

        start.swap_with_slice(dest);

        let new_size = size - length;
        let new_index = dest_index + length;
        empty_space_indices_by_length.get_mut(&size).unwrap().pop();
        if new_size > 0 {
            empty_space_indices_by_length
                .entry(new_size)
                .or_insert_with(BinaryHeap::new)
                .push(Reverse(new_index));
        }

        // No need to account for the new space at the end
    }

    fn compact_no_fragmentation(&mut self) {
        if self.blocks.is_empty() {
            return;
        }

        let mut empty_space_indices_by_length =
            HashMap::<usize, BinaryHeap<Reverse<usize>>>::new();
        let mut current_start_index = 0usize;
        for (i, &block) in self.blocks.iter().enumerate() {
            if block.is_some() {
                if current_start_index < i {
                    empty_space_indices_by_length
                        .entry(i - current_start_index)
                        .or_insert_with(BinaryHeap::new)
                        .push(Reverse(current_start_index));
                }
                current_start_index = i + 1;
            }
        }

        let mut current_end_index_inclusive = self.blocks.len() - 1;
        let mut current_file: Option<FileID> = None;
        for i in (0..self.blocks.len()).rev() {
            if self.blocks[i] != current_file {
                if current_file.is_some() {
                    self.move_range(
                        &mut empty_space_indices_by_length,
                        i + 1,
                        current_end_index_inclusive - i,
                    );
                }
                current_end_index_inclusive = i;
            }
            current_file = self.blocks[i];
        }
    }

    fn checksum(&self) -> i64 {
        self.blocks
            .iter()
            .enumerate()
            .map(|(i, block)| i as i64 * block.unwrap_or(0))
            .sum()
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let lines = reader.lines().collect::<Result<Vec<_>, _>>()?;
    if lines.len() != 1 {
        return Err(invalid_input("Expected one line"));
    }

    let mut disk = Disk::new(&lines[0])?;
    match part {
        Part::Part1 => disk.compact(),
        Part::Part2 => disk.compact_no_fragmentation(),
    }

    println!("{}", disk.checksum());

    Ok(())
}
