use std::io;
use std::num::Wrapping;

use crate::errors::invalid_input;
use crate::parse::lines;
use crate::part::Part;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Operation {
    RemoveLens,
    AddLens(u8),
}

impl Operation {
    fn from_string(string: &str) -> io::Result<Self> {
        match string.chars().next() {
            Some('-') => Ok(Self::RemoveLens),
            Some('=') => string[1..]
                .parse::<u8>()
                .map_err(invalid_input)
                .map(Self::AddLens),
            _ => Err(invalid_input("Invalid operation char")),
        }
    }
}

struct Step {
    label: String,
    operation: Operation,
    label_hash: u8,
    overall_hash: u8,
}

impl Step {
    fn from_string(string: &str) -> io::Result<Self> {
        let Some(operation_index) = string.find(|ch| ch == '-' || ch == '=')
        else {
            return Err(invalid_input("Could not find operation char in step"));
        };
        let label = string[..operation_index].to_owned();
        let operation = Operation::from_string(&string[operation_index..])?;
        let label_hash = Self::compute_hash(&label);
        let overall_hash = Self::compute_hash(string);
        Ok(Self {
            label,
            operation,
            label_hash,
            overall_hash,
        })
    }

    fn compute_hash(string: &str) -> u8 {
        let mut result = Wrapping(0u8);
        for byte in string.bytes() {
            result += byte;
            result *= 17;
        }
        result.0
    }
}

struct LensBox {
    number: u64,

    // Ideally we would use a linked hash map here
    lenses: Vec<(String, u8)>,
}

impl LensBox {
    fn new(number: u64) -> Self {
        Self {
            number,
            lenses: Vec::new(),
        }
    }

    fn remove(&mut self, name: &str) {
        if let Some(index) = self.find(name) {
            self.lenses.remove(index);
        }
    }

    fn add(&mut self, name: String, focal_length: u8) {
        if let Some(index) = self.find(&name) {
            self.lenses[index].1 = focal_length;
        } else {
            self.lenses.push((name, focal_length));
        }
    }

    fn find(&self, name: &str) -> Option<usize> {
        for (i, (lens_name, _)) in self.lenses.iter().enumerate() {
            if lens_name == name {
                return Some(i);
            }
        }
        None
    }

    fn focusing_power(&self) -> u64 {
        self.lenses
            .iter()
            .enumerate()
            .map(|(i, (_, focal_length))| {
                (1 + self.number) * (1 + i as u64) * *focal_length as u64
            })
            .sum()
    }
}

struct LensBoxes {
    boxes: Vec<LensBox>,
}

impl LensBoxes {
    fn new() -> Self {
        Self {
            boxes: (0..256).map(|number| LensBox::new(number)).collect(),
        }
    }

    fn run_step(&mut self, step: Step) {
        let the_box = &mut self.boxes[step.label_hash as usize];
        match step.operation {
            Operation::RemoveLens => {
                the_box.remove(&step.label);
            }
            Operation::AddLens(focal_length) => {
                the_box.add(step.label, focal_length);
            }
        }
    }

    fn focusing_power(&self) -> u64 {
        self.boxes.iter().map(LensBox::focusing_power).sum()
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let [line] = &lines(reader)?.collect::<Vec<_>>()[..] else {
        return Err(invalid_input("Expected only 1 line"));
    };

    let steps = line
        .split(',')
        .map(Step::from_string)
        .collect::<io::Result<Vec<_>>>()?;

    let result: u64 = match part {
        Part::Part1 => {
            steps.into_iter().map(|step| step.overall_hash as u64).sum()
        }
        Part::Part2 => {
            let mut boxes = LensBoxes::new();
            for step in steps {
                boxes.run_step(step);
            }
            boxes.focusing_power()
        }
    };

    println!("{result}");

    Ok(())
}
