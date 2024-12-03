use std::io;
use std::io::BufRead;

use crate::part::Part;

#[derive(Debug)]
enum State {
    Skipping,
    M,
    U,
    L,
    FirstDigits(i64),
    SecondDigits(i64, i64),
    D,
    O,
    DoParen,
    N,
    Apostrophe,
    T,
    DontParen,
}

struct Program {
    state: State,
    total: i64,
    enabled: bool,
}

impl Program {
    fn new() -> Self {
        Self {
            state: State::Skipping,
            total: 0,
            enabled: true,
        }
    }

    fn parse_muls<R: io::Read>(
        &mut self,
        reader: io::BufReader<R>,
        part: Part,
    ) -> io::Result<()> {
        for line in reader.lines() {
            for ch in line?.chars() {
                self.update(ch, part);
            }
        }
        Ok(())
    }

    fn update(&mut self, ch: char, part: Part) {
        let (new_state, addition) = match self.state {
            State::Skipping => {
                if ch == 'm' {
                    (State::M, 0)
                } else if part == Part::Part2 && ch == 'd' {
                    (State::D, 0)
                } else {
                    (State::Skipping, 0)
                }
            }
            State::M => {
                if ch == 'u' {
                    (State::U, 0)
                } else {
                    (State::Skipping, 0)
                }
            }
            State::U => {
                if ch == 'l' {
                    (State::L, 0)
                } else {
                    (State::Skipping, 0)
                }
            }
            State::L => {
                if ch == '(' {
                    (State::FirstDigits(0), 0)
                } else {
                    (State::Skipping, 0)
                }
            }
            State::FirstDigits(digits) => {
                if ch.is_ascii_digit() {
                    (
                        State::FirstDigits(
                            digits * 10 + ch.to_digit(10).unwrap() as i64,
                        ),
                        0,
                    )
                } else if ch == ',' {
                    (State::SecondDigits(digits, 0), 0)
                } else {
                    (State::Skipping, 0)
                }
            }
            State::SecondDigits(first_digits, digits) => {
                if ch.is_ascii_digit() {
                    (
                        State::SecondDigits(
                            first_digits,
                            digits * 10 + ch.to_digit(10).unwrap() as i64,
                        ),
                        0,
                    )
                } else if ch == ')' {
                    (State::Skipping, first_digits * digits)
                } else {
                    (State::Skipping, 0)
                }
            }
            State::D => {
                if ch == 'o' {
                    (State::O, 0)
                } else {
                    (State::Skipping, 0)
                }
            }
            State::O => {
                if ch == '(' {
                    (State::DoParen, 0)
                } else if ch == 'n' {
                    (State::N, 0)
                } else {
                    (State::Skipping, 0)
                }
            }
            State::DoParen => {
                if ch == ')' {
                    self.enabled = true;
                }
                (State::Skipping, 0)
            }
            State::N => {
                if ch == '\'' {
                    (State::Apostrophe, 0)
                } else {
                    (State::Skipping, 0)
                }
            }
            State::Apostrophe => {
                if ch == 't' {
                    (State::T, 0)
                } else {
                    (State::Skipping, 0)
                }
            }
            State::T => {
                if ch == '(' {
                    (State::DontParen, 0)
                } else {
                    (State::Skipping, 0)
                }
            }
            State::DontParen => {
                if ch == ')' {
                    self.enabled = false;
                }
                (State::Skipping, 0)
            }
        };
        self.state = new_state;
        if self.enabled {
            self.total += addition;
        }
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut program = Program::new();
    program.parse_muls(reader, part)?;
    let result = program.total;
    println!("{result}");

    Ok(())
}
