use std::collections::HashMap;
use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::part::Part;

struct MachineState {
    x: i64,
    cycle_count: i64,
    x_history: HashMap<i64, i64>,
}

impl MachineState {
    pub fn new() -> Self {
        let mut x_history = HashMap::new();
        x_history.insert(1, 1);
        Self {
            x: 1,
            cycle_count: 1,
            x_history,
        }
    }

    pub fn signal_strength(&self, cycle_count: i64) -> Option<i64> {
        self.x_history.get(&cycle_count).map(|x| x * cycle_count)
    }

    pub fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::NoOp => {
                // Do nothing
                // 1 cycle
                self.cycle_count += 1;
                self.x_history.insert(self.cycle_count, self.x);
            }
            Instruction::AddX(num) => {
                // Add
                // 2 cycles
                self.cycle_count += 2;
                self.x_history.insert(self.cycle_count - 1, self.x);
                self.x += num;
                self.x_history.insert(self.cycle_count, self.x);
            }
        }
    }

    pub fn print_crt(&self) {
        const WIDTH: i64 = 40;

        for cycle_count in 1..=self.cycle_count {
            let x_position = (cycle_count - 1) % WIDTH;

            if cycle_count != 0 && x_position == 0 {
                println!();
            }

            let x_register_value = self.x_history[&cycle_count];

            if (x_register_value - x_position).abs() <= 1 {
                print!("#");
            } else {
                print!(".");
            }
        }
        println!();
    }
}

#[derive(Clone, Copy)]
enum Instruction {
    NoOp,
    AddX(i64),
}

impl Instruction {
    pub fn new(line: &str) -> io::Result<Self> {
        let words = line.split(' ').collect::<Vec<_>>();
        if words.is_empty() {
            Err(invalid_input("Empty line"))?
        }

        let instruction = words[0];
        match instruction {
            "noop" => {
                if words.len() == 1 {
                    Ok(Self::NoOp)
                } else {
                    Err(invalid_input("Extra data after noop"))
                }
            }
            "addx" => {
                if words.len() == 2 {
                    Ok(Self::AddX(
                        words[1].parse::<i64>().map_err(invalid_input)?,
                    ))
                } else {
                    Err(invalid_input("addx must have exactly 1 argument"))
                }
            }
            _ => Err(invalid_input("Unknown instruction")),
        }
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut state = MachineState::new();

    for line in reader.lines() {
        let line = line?;
        let instruction = Instruction::new(&line)?;
        state.execute(instruction);
    }

    match part {
        Part::Part1 => {
            let signal_strengths = [20, 60, 100, 140, 180, 220]
                .into_iter()
                .map(|cycle| state.signal_strength(cycle))
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| invalid_input("Not enough cycles in input"))?;
            println!("{}", signal_strengths.into_iter().sum::<i64>());
        }
        Part::Part2 => {
            state.print_crt();
        }
    }

    Ok(())
}
