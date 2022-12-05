use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::part::Part;

#[derive(Clone, Copy)]
struct Movement {
    count: usize,
    one_indexed_stack_from: usize,
    one_indexed_stack_to: usize,
}

struct Stacks {
    stacks: Vec<Vec<char>>,
}

impl Stacks {
    pub fn new(stack_lines: &[String]) -> io::Result<Self> {
        let mut by_height: Vec<Vec<char>> = Vec::new();
        let mut num_stacks: Option<usize> = None;
        for line in stack_lines {
            let this_height: Vec<_> = line.chars().skip(1).step_by(4).collect();

            match num_stacks {
                None => num_stacks = Some(this_height.len()),
                Some(len) => {
                    if this_height.len() != len {
                        Err(invalid_input("Differing numbers of stacks"))?
                    }
                }
            }

            by_height.push(this_height);
        }

        let Some(num_stacks) = num_stacks else {
            return Err(invalid_input("Empty input"));
        };

        let mut stacks = Vec::<Vec<char>>::with_capacity(num_stacks);
        stacks.resize(num_stacks, Vec::new());
        for row in by_height.iter().rev() {
            for (i, ch) in row.iter().enumerate() {
                if *ch != ' ' {
                    stacks[i].push(*ch);
                }
            }
        }

        Ok(Self { stacks })
    }

    fn get_stacks(
        &mut self,
        movement: Movement,
    ) -> io::Result<(&mut Vec<char>, &mut Vec<char>)> {
        if movement.one_indexed_stack_from == 0 {
            Err(invalid_input("from stack is #0"))?
        }
        if movement.one_indexed_stack_to == 0 {
            Err(invalid_input("to stack is #0"))?
        }

        let from_idx = movement.one_indexed_stack_from - 1;
        let to_idx = movement.one_indexed_stack_to - 1;
        if from_idx >= self.stacks.len() {
            Err(invalid_input(format!(
                "from stack #{} out of bounds for size {}",
                movement.one_indexed_stack_from,
                self.stacks.len(),
            )))?
        }
        if to_idx >= self.stacks.len() {
            Err(invalid_input(format!(
                "to stack #{} out of bounds for size {}",
                movement.one_indexed_stack_to,
                self.stacks.len(),
            )))?
        }
        if from_idx == to_idx {
            Err(invalid_input(format!(
                "to and from are both stack #{}",
                movement.one_indexed_stack_from,
            )))?;
        }

        let (stack_from, stack_to) = if from_idx > to_idx {
            let (part_to, part_from) = self.stacks.split_at_mut(from_idx);
            let stack_from = &mut part_from[0];
            let stack_to = &mut part_to[to_idx];
            (stack_from, stack_to)
        } else { // to_idx > from_idx
            let (part_from, part_to) = self.stacks.split_at_mut(to_idx);
            (&mut part_from[from_idx], &mut part_to[0])
        };

        if stack_from.len() < movement.count {
            Err(invalid_input("count is too large"))?
        }

        Ok((stack_from, stack_to))
    }

    pub fn apply_one_by_one(&mut self, movement: Movement) -> io::Result<()> {
        let (stack_from, stack_to) = self.get_stacks(movement)?;
        
        for _i in 0..movement.count {
            stack_to.push(stack_from.pop().unwrap());
        }

        Ok(())
    }

    pub fn apply_at_once(&mut self, movement: Movement) -> io::Result<()> {
        let (stack_from, stack_to) = self.get_stacks(movement)?;

        let mut to_move = stack_from.split_off(
            stack_from.len() - movement.count
        );
        stack_to.append(&mut to_move);

        Ok(())
    }

    fn print_tops(&self) {
        let summary = self.stacks.iter().map(
            |stack| stack.last().map(|ch| *ch).unwrap_or(' ')
        ).collect::<String>();

        println!("{}", summary);
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut stack_lines = Vec::<String>::new();
    let mut is_stack_line = true;
    let mut stacks: Option<Stacks> = None;
    let mut movements = Vec::<Movement>::new();

    for line in reader.lines() {
        let line = line?;

        if is_stack_line && line == "" {
            // If at the end of the drawing, parse it.
            is_stack_line = false;

            let drawing_lines = stack_lines.split_last().ok_or_else(
                || invalid_input("Starting blank line")
            )?.1;

            stacks = Some(Stacks::new(drawing_lines)?);
            continue;
        }

        if is_stack_line {
            // Drawing
            stack_lines.push(line);
        } else {
            // move _ from _ to _
            let words = line.split(' ');
            let numbers: Result<Vec<_>, _> = words.skip(1).step_by(2).map(
                |word| word.parse::<usize>().map_err(invalid_input)
            ).collect();

            let numbers = numbers?;
            if numbers.len() != 3 {
                Err(invalid_input("Invalid movement"))?
            }

            movements.push(Movement {
                count: numbers[0],
                one_indexed_stack_from: numbers[1],
                one_indexed_stack_to: numbers[2],
            });
        }
    }

    let mut stacks = stacks.ok_or_else(|| invalid_input("Missing drawing"))?;
    for movement in movements {
        match part {
            Part::Part1 => stacks.apply_one_by_one(movement)?,
            Part::Part2 => stacks.apply_at_once(movement)?,
        }
    }

    stacks.print_tops();
    Ok(())
}
