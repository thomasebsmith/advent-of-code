use std::io;

use num_rational::Rational64;

use crate::errors::invalid_input;
use crate::parse::{lines, paragraphs, parse_all};
use crate::part::Part;

type Num = Rational64;

struct Machine {
    a_x: Num,
    a_y: Num,
    b_x: Num,
    b_y: Num,
    prize_x: Num,
    prize_y: Num,
}

impl Machine {
    fn from_paragraph(paragraph: Vec<String>, part: Part) -> io::Result<Self> {
        let parse =
            |line: &str, prefix: &str, split: &str| -> io::Result<(Num, Num)> {
                let suffix = line
                    .strip_prefix(prefix)
                    .ok_or_else(|| invalid_input("Missing line prefix"))?;
                let &[first, second] =
                    &parse_all::<_, Num>(suffix.split(split))?[..]
                else {
                    return Err(invalid_input("Expected two parts on line"));
                };
                Ok((first, second))
            };

        if paragraph.len() != 3 {
            return Err(invalid_input("Expected 3 lines per machine"));
        }
        let (a_x, a_y) = parse(&paragraph[0], "Button A: X+", ", Y+")?;
        let (b_x, b_y) = parse(&paragraph[1], "Button B: X+", ", Y+")?;
        let (mut prize_x, mut prize_y) =
            parse(&paragraph[2], "Prize: X=", ", Y=")?;
        if part == Part::Part2 {
            let part_2_diff = Num::from_integer(10000000000000);
            prize_x += part_2_diff;
            prize_y += part_2_diff;
        }
        if a_x == Num::ZERO
            || a_y == Num::ZERO
            || b_x == Num::ZERO
            || b_y == Num::ZERO
            || prize_x == Num::ZERO
            || prize_y == Num::ZERO
        {
            return Err(invalid_input("Values cannot be 0"));
        }
        Ok(Self {
            a_x,
            a_y,
            b_x,
            b_y,
            prize_x,
            prize_y,
        })
    }

    fn minimum_tokens(&self) -> Option<Num> {
        let a_tokens = Num::from_integer(3);
        let b_tokens = Num::from_integer(1);
        if self.a_y * self.b_x == self.a_x * self.b_y {
            unimplemented!();
        } else {
            let a_presses = (self.prize_y
                - (self.b_y * self.prize_x / self.b_x))
                / (self.a_y - self.a_x * self.b_y / self.b_x);
            let b_presses = (self.prize_x - self.a_x * a_presses) / self.b_x;
            if !a_presses.is_integer() || !b_presses.is_integer() {
                None
            } else {
                Some(a_presses * a_tokens + b_presses * b_tokens)
            }
        }
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut total_min_tokens = Num::ZERO;
    for paragraph in paragraphs(lines(reader)?) {
        let machine = Machine::from_paragraph(paragraph, part)?;
        if let Some(min_tokens) = machine.minimum_tokens() {
            total_min_tokens += min_tokens;
        }
    }

    println!("{total_min_tokens}");

    Ok(())
}
