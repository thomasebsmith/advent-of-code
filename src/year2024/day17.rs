use std::io;

use crate::errors::invalid_input;
use crate::parse::lines;
use crate::part::Part;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum OperandType {
    Literal,
    Combo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Instruction {
    DivideIntoA,
    BitwiseXORLiteral,
    OperandToRegister,
    JumpIfNotZero,
    BitwiseXORRegisters,
    Output,
    DivideIntoB,
    DivideIntoC,
}

impl Instruction {
    fn operand_type(self) -> OperandType {
        match self {
            Self::DivideIntoA
            | Self::OperandToRegister
            | Self::Output
            | Self::DivideIntoB
            | Self::DivideIntoC => OperandType::Combo,
            Self::BitwiseXORLiteral
            | Self::JumpIfNotZero
            | Self::BitwiseXORRegisters => OperandType::Literal,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Bits {
    value: i64,
}

impl Bits {
    fn new(value: i64) -> io::Result<Self> {
        const MAX_VALUE: i64 = 7;
        if value > MAX_VALUE {
            Err(invalid_input("Number exceeded 3 bits"))
        } else {
            Ok(Self { value })
        }
    }

    fn as_instruction(self) -> Instruction {
        match self.value {
            0 => Instruction::DivideIntoA,
            1 => Instruction::BitwiseXORLiteral,
            2 => Instruction::OperandToRegister,
            3 => Instruction::JumpIfNotZero,
            4 => Instruction::BitwiseXORRegisters,
            5 => Instruction::Output,
            6 => Instruction::DivideIntoB,
            7 => Instruction::DivideIntoC,
            _ => unreachable!(),
        }
    }

    fn raw_value(self) -> i64 {
        self.value
    }
}

fn skip_then_parse_i64(value: &str, prefix: &str) -> io::Result<i64> {
    let Some(to_parse) = value.strip_prefix(prefix) else {
        return Err(invalid_input("Prefix not matched"));
    };
    to_parse.parse().map_err(invalid_input)
}

#[derive(Clone)]
struct Computer {
    memory: Vec<Bits>,
    instruction_pointer: i64,
    register_a: i64,
    register_b: i64,
    register_c: i64,
    output: Vec<i64>,
}

impl Computer {
    fn new<R: io::Read>(reader: io::BufReader<R>) -> io::Result<Self> {
        let all_lines = lines(reader)?.collect::<Vec<_>>();
        if all_lines.len() != 5 {
            return Err(invalid_input("Expected five lines"));
        }

        let register_a = skip_then_parse_i64(&all_lines[0], "Register A: ")?;
        let register_b = skip_then_parse_i64(&all_lines[1], "Register B: ")?;
        let register_c = skip_then_parse_i64(&all_lines[2], "Register C: ")?;

        if !all_lines[3].is_empty() {
            return Err(invalid_input("Expected line 4 to be empty"));
        }

        let Some(memory_str) = all_lines[4].strip_prefix("Program: ") else {
            return Err(invalid_input("Expected prefix \"Program: \""));
        };

        let memory = memory_str
            .split(",")
            .map(|number_str| {
                Bits::new(number_str.parse().map_err(invalid_input)?)
            })
            .collect::<io::Result<Vec<_>>>()?;

        let instruction_pointer = 0;

        let output = Vec::<i64>::new();

        Ok(Self {
            memory,
            instruction_pointer,
            register_a,
            register_b,
            register_c,
            output,
        })
    }

    fn run_instruction(&mut self) -> bool {
        if self.instruction_pointer < 0
            || self.memory.is_empty()
            || self.instruction_pointer >= self.memory.len() as i64 - 1
        {
            return false;
        }

        let instruction =
            self.memory[self.instruction_pointer as usize].as_instruction();
        let operand_type = instruction.operand_type();
        let raw_operand =
            self.memory[self.instruction_pointer as usize + 1].raw_value();
        let operand_value = match (operand_type, raw_operand) {
            (OperandType::Literal, _) | (_, 0) | (_, 1) | (_, 2) | (_, 3) => {
                raw_operand
            }
            (OperandType::Combo, 4) => self.register_a,
            (OperandType::Combo, 5) => self.register_b,
            (OperandType::Combo, 6) => self.register_c,
            (OperandType::Combo, 7) => return false,
            _ => return false,
        };

        const INSTRUCTION_POINTER_INCREMENT: i64 = 2;

        match instruction {
            Instruction::DivideIntoA => {
                self.register_a /= 1 << operand_value;
            }
            Instruction::BitwiseXORLiteral => {
                self.register_b ^= operand_value;
            }
            Instruction::OperandToRegister => {
                self.register_b = operand_value % 8;
            }
            Instruction::JumpIfNotZero => {
                if self.register_a != 0 {
                    self.instruction_pointer =
                        operand_value - INSTRUCTION_POINTER_INCREMENT;
                }
            }
            Instruction::BitwiseXORRegisters => {
                self.register_b ^= self.register_c;
            }
            Instruction::Output => {
                self.output.push(operand_value % 8);
            }
            Instruction::DivideIntoB => {
                self.register_b = self.register_a / (1 << operand_value);
            }
            Instruction::DivideIntoC => {
                self.register_c = self.register_a / (1 << operand_value);
            }
        }

        self.instruction_pointer += INSTRUCTION_POINTER_INCREMENT;

        true
    }

    fn run_until_halt(&mut self) {
        while self.run_instruction() {}
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut computer = Computer::new(reader)?;

    let result = match part {
        Part::Part1 => {
            computer.run_until_halt();
            let string_vec = computer
                .output
                .iter()
                .map(i64::to_string)
                .collect::<Vec<_>>();
            string_vec.join(",")
        }
        Part::Part2 => {
            // This solution is specific to the type of program in the sample
            // output.
            let mut components = Vec::<i64>::new();
            components.push(0);
            let final_value = loop {
                let mut register_a_value = 0i64;
                for &component in components.iter() {
                    register_a_value <<= 3;
                    register_a_value |= component;
                }
                let mut new_computer = computer.clone();
                new_computer.register_a = register_a_value;
                new_computer.run_until_halt();
                let mut num_same: usize = 0;
                let size_diff = computer.memory.len() - components.len();
                if new_computer.output.len() != components.len() {
                    panic!();
                }
                for i in (0..new_computer.output.len()).rev() {
                    if new_computer.output[i]
                        != computer.memory[size_diff + i].raw_value()
                    {
                        break;
                    }
                    num_same += 1;
                }

                if num_same == components.len() {
                    if size_diff == 0 {
                        break register_a_value;
                    }
                    components.push(0);
                } else if num_same + 1 == components.len() {
                    loop {
                        *components.last_mut().unwrap() += 1;
                        if *components.last().unwrap() > 7 {
                            components.pop();
                            if components.is_empty() {
                                panic!("Ran out of possibilities");
                            }
                        } else {
                            break;
                        }
                    }
                } else {
                    panic!("Multiple differing components: {num_same} same with {} components (size diff={size_diff})", components.len());
                }
            };
            /*
            // Generic solution:
            let mut register_a_value = 0i64;
            'outer: loop {
                register_a_value += 1;
                if register_a_value % 1_000_000 == 0 {
                    println!("A={register_a_value}");
                }
                let mut new_computer = computer.clone();
                new_computer.register_a = register_a_value;
                new_computer.run_until_halt();
                if new_computer.output.len() == computer.memory.len() {
                    for i in 0..new_computer.output.len() {
                        if new_computer.output[i] != computer.memory[i].raw_value() {
                            continue 'outer;
                        }
                    }
                    break;
                }
            }*/
            final_value.to_string()
        }
    };

    println!("{result}");

    Ok(())
}
