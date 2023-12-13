use std::collections::HashSet;
use std::io;

use crate::errors::invalid_input;
use crate::parse::{lines, paragraphs};
use crate::part::Part;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum Tile {
    Ash,
    Rocks,
}

impl Tile {
    fn from_char(ch: char) -> io::Result<Self> {
        match ch {
            '.' => Ok(Self::Ash),
            '#' => Ok(Self::Rocks),
            _ => Err(invalid_input("Invalid tile character")),
        }
    }

    fn reverse(self) -> Self {
        match self {
            Self::Ash => Self::Rocks,
            Self::Rocks => Self::Ash,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
struct Reflection {
    orientation: Orientation,
    offset: usize,
}

impl Reflection {
    fn summary(self) -> usize {
        match self.orientation {
            Orientation::Horizontal => 100 * self.offset,
            Orientation::Vertical => self.offset,
        }
    }
}

struct Pattern {
    map: Vec<Vec<Tile>>,
    width: usize,
    height: usize,
}

impl Pattern {
    fn from_lines(lines: Vec<String>) -> io::Result<Self> {
        let mut width: Option<usize> = None;
        let mut map = Vec::<Vec<Tile>>::new();
        for line in lines {
            let row = line
                .chars()
                .map(Tile::from_char)
                .collect::<io::Result<Vec<_>>>()?;
            if let Some(the_width) = width {
                if row.len() != the_width {
                    return Err(invalid_input("Differing row widths"));
                }
            } else {
                width = Some(row.len());
            }
            map.push(row);
        }

        if map.len() == 0 || width.unwrap() == 0 {
            return Err(invalid_input("Empty map"));
        }
        let width = width.unwrap();
        let height = map.len();

        Ok(Self { map, width, height })
    }

    fn fix_smudge(&mut self) -> bool {
        // This is fairly slow, but it works fine with smaller input sizes.
        let original_reflections = self.find_reflections();
        for row_index in 0..self.height {
            for col_index in 0..self.width {
                let original_value = self.map[row_index][col_index];
                self.map[row_index][col_index] = original_value.reverse();
                if self
                    .find_reflections()
                    .difference(&original_reflections)
                    .count()
                    > 0
                {
                    return true;
                }
                self.map[row_index][col_index] = original_value;
            }
        }
        false
    }

    fn find_reflections(&self) -> HashSet<Reflection> {
        let mut reflections = HashSet::<Reflection>::new();

        // Look for horizontal reflections
        for row_index in 1..self.height {
            let mut is_symmetrical = true;
            for mirror_offset in 0..row_index {
                let mirror_index_small = row_index - mirror_offset - 1;
                let mirror_index_big = row_index + mirror_offset;
                if mirror_index_big >= self.height {
                    break;
                }
                if self.map[mirror_index_small] != self.map[mirror_index_big] {
                    is_symmetrical = false;
                    break;
                }
            }
            if is_symmetrical {
                reflections.insert(Reflection {
                    orientation: Orientation::Horizontal,
                    offset: row_index,
                });
            }
        }

        // Look for vertical reflections
        for col_index in 1..self.width {
            let mut is_symmetrical = true;
            for mirror_offset in 0..col_index {
                let mirror_index_small = col_index - mirror_offset - 1;
                let mirror_index_big = col_index + mirror_offset;
                if mirror_index_big >= self.width {
                    break;
                }

                for row_index in 0..self.height {
                    if self.map[row_index][mirror_index_small]
                        != self.map[row_index][mirror_index_big]
                    {
                        is_symmetrical = false;
                        break;
                    }
                }
            }
            if is_symmetrical {
                reflections.insert(Reflection {
                    orientation: Orientation::Vertical,
                    offset: col_index,
                });
            }
        }

        reflections
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut result: usize = 0;

    for paragraph in paragraphs(lines(reader)?) {
        let mut pattern = Pattern::from_lines(paragraph)?;
        let sum: usize = match part {
            Part::Part1 => pattern
                .find_reflections()
                .into_iter()
                .map(Reflection::summary)
                .sum(),
            Part::Part2 => {
                let original_reflections = pattern.find_reflections();
                if !pattern.fix_smudge() {
                    return Err(invalid_input("Could not fix smudge"));
                }
                let new_reflections = pattern.find_reflections();
                new_reflections
                    .difference(&original_reflections)
                    .map(|reflection| reflection.summary())
                    .sum()
            }
        };
        result += sum;
    }

    println!("{result}");

    Ok(())
}
