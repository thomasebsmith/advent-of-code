use std::collections::{HashMap, HashSet};
use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::part::Part;

#[derive(Clone, Copy)]
enum Bit {
    Digit(u64),
    Space,
    Symbol(char),
    PotentialGear,
}

impl Bit {
    fn from_char(ch: char) -> Self {
        if ch == '.' {
            Self::Space
        } else if ch == '*' {
            Self::PotentialGear
        } else if ch.is_ascii_digit() {
            Self::Digit(u64::from(ch) - u64::from('0'))
        } else {
            Self::Symbol(ch)
        }
    }

    fn is_symbol(self) -> bool {
        matches!(self, Self::Symbol(_) | Self::PotentialGear)
    }
}

struct Map {
    cells: Vec<Vec<Bit>>,
}

impl Map {
    fn from_reader<R: io::Read>(reader: io::BufReader<R>) -> io::Result<Self> {
        let mut cells = Vec::<Vec<Bit>>::new();
        let mut width: Option<usize> = None;
        for line in reader.lines() {
            let line = line?;
            if line == "" {
                continue;
            }

            let map_line = line.chars().map(Bit::from_char).collect::<Vec<_>>();
            assert!(map_line.len() != 0); // We should have continue;ed if this were the case
            if let Some(width) = width {
                if map_line.len() != width {
                    return Err(invalid_input(format!(
                        "Differing line lengths: expected {width}, saw {}",
                        map_line.len()
                    )));
                }
            } else {
                width = Some(map_line.len());
            }

            cells.push(map_line);
        }

        if cells.len() == 0 {
            Err(invalid_input("Empty map"))
        } else {
            Ok(Self { cells })
        }
    }

    fn width(&self) -> usize {
        self.cells[0].len()
    }

    fn height(&self) -> usize {
        self.cells.len()
    }

    fn neighbor_locations(
        &self,
        row: usize,
        col: usize,
    ) -> Vec<(usize, usize)> {
        assert!(row < self.height() && col < self.width());

        let mut results = Vec::<(usize, usize)>::new();
        if row != 0 {
            if col != 0 {
                results.push((row - 1, col - 1));
            }
            results.push((row - 1, col));
            if col != self.width() - 1 {
                results.push((row - 1, col + 1));
            }
        }

        if col != 0 {
            results.push((row, col - 1));
        }
        if col != self.width() - 1 {
            results.push((row, col + 1));
        }

        if row != self.height() - 1 {
            if col != 0 {
                results.push((row + 1, col - 1));
            }
            results.push((row + 1, col));
            if col != self.width() - 1 {
                results.push((row + 1, col + 1));
            }
        }

        results
    }

    fn neighbors(&self, row: usize, col: usize) -> Vec<Bit> {
        self.neighbor_locations(row, col)
            .into_iter()
            .map(|(row, col)| self.cells[row][col])
            .collect::<_>()
    }

    fn neighbor_matches<F: Fn(Bit) -> bool>(
        &self,
        row: usize,
        col: usize,
        func: F,
    ) -> bool {
        self.neighbors(row, col).into_iter().any(func)
    }

    // TODO: This and gear_ratio_sum duplicate code to calculate part numbers
    fn part_number_sum(&self) -> u64 {
        let mut sum: u64 = 0;
        for (row, row_of_cells) in self.cells.iter().enumerate() {
            let mut digit_acc: u64 = 0;
            let mut is_symbol_adjacent = false;

            for (col, bit) in row_of_cells.iter().enumerate() {
                if let Bit::Digit(digit) = bit {
                    digit_acc *= 10;
                    digit_acc += digit;
                    if !is_symbol_adjacent
                        && self.neighbor_matches(row, col, Bit::is_symbol)
                    {
                        is_symbol_adjacent = true;
                    }
                    continue;
                }

                if is_symbol_adjacent {
                    sum += digit_acc;
                }

                is_symbol_adjacent = false;
                digit_acc = 0;
            }

            if is_symbol_adjacent {
                sum += digit_acc;
            }
        }

        sum
    }

    fn gear_ratio_sum(&self) -> u64 {
        let mut gear_adjacent_part_map =
            HashMap::<(usize, usize), Vec<u64>>::new();

        let mut add_part_number = |gear_locations: &HashSet<(usize, usize)>,
                                   num: u64| {
            for loc in gear_locations {
                gear_adjacent_part_map
                    .entry(*loc)
                    .or_insert(Vec::new())
                    .push(num);
            }
        };

        for (row, row_of_cells) in self.cells.iter().enumerate() {
            let mut digit_acc: u64 = 0;
            let mut is_symbol_adjacent = false;
            let mut neighboring_gears = HashSet::<(usize, usize)>::new();

            for (col, bit) in row_of_cells.iter().enumerate() {
                if let Bit::Digit(digit) = bit {
                    digit_acc *= 10;
                    digit_acc += digit;
                    if !is_symbol_adjacent
                        && self.neighbor_matches(row, col, Bit::is_symbol)
                    {
                        is_symbol_adjacent = true;
                    }

                    for (neighbor_row, neighbor_col) in
                        self.neighbor_locations(row, col)
                    {
                        if matches!(
                            self.cells[neighbor_row][neighbor_col],
                            Bit::PotentialGear
                        ) {
                            neighboring_gears
                                .insert((neighbor_row, neighbor_col));
                        }
                    }

                    continue;
                }

                if is_symbol_adjacent {
                    add_part_number(&neighboring_gears, digit_acc);
                }

                neighboring_gears.clear();
                is_symbol_adjacent = false;
                digit_acc = 0;
            }

            if is_symbol_adjacent {
                add_part_number(&neighboring_gears, digit_acc);
            }
        }

        let mut sum: u64 = 0;
        for (_, neighboring_parts) in &gear_adjacent_part_map {
            if neighboring_parts.len() == 2 {
                sum += neighboring_parts[0] * neighboring_parts[1];
            }
        }
        sum
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let map = Map::from_reader(reader)?;

    let result = match part {
        Part::Part1 => map.part_number_sum(),
        Part::Part2 => map.gear_ratio_sum(),
    };

    println!("{result}");

    Ok(())
}
