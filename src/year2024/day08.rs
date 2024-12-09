use std::collections::HashMap;
use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::part::Part;

// From https://github.com/rust-num/num-integer/blob/03640c2a9472fad6f40845ab29c7c9502935d1d3/src/lib.rs
// Licensed under MIT license:
// https://github.com/rust-num/num-integer/blob/03640c2a9472fad6f40845ab29c7c9502935d1d3/LICENSE-MIT
fn gcd(this: isize, other: isize) -> isize {
    // Use Stein's algorithm
    let mut m = this;
    let mut n = other;
    if m == 0 || n == 0 {
        return (m | n).abs();
    }

    // find common factors of 2
    let shift = (m | n).trailing_zeros();

    // The algorithm needs positive numbers, but the minimum value
    // can't be represented as a positive one.
    // It's also a power of two, so the gcd can be
    // calculated by bitshifting in that case

    // Assuming two's complement, the number created by the shift
    // is positive for all numbers except gcd = abs(min value)
    // The call to .abs() causes a panic in debug mode
    if m == isize::min_value() || n == isize::min_value() {
        return (1isize << shift).abs();
    }

    // guaranteed to be positive now, rest like unsigned algorithm
    m = m.abs();
    n = n.abs();

    // divide n and m by 2 until odd
    m >>= m.trailing_zeros();
    n >>= n.trailing_zeros();

    while m != n {
        if m > n {
            m -= n;
            m >>= m.trailing_zeros();
        } else {
            n -= m;
            n >>= n.trailing_zeros();
        }
    }
    m << shift
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Position {
    row: isize,
    col: isize,
}

struct Map {
    antennas: HashMap<char, Vec<Position>>,
    width: isize,
    height: isize,
}

impl Map {
    fn new<R: io::Read>(reader: io::BufReader<R>) -> io::Result<Self> {
        let mut width: Option<isize> = None;
        let mut antennas = HashMap::<char, Vec<Position>>::new();

        let mut height = 0isize;
        for (row, line) in reader.lines().enumerate() {
            let line = line?;

            for (col, ch) in line.chars().enumerate() {
                if ch == '.' {
                    continue;
                }
                let position = Position {
                    row: row as isize,
                    col: col as isize,
                };
                antennas.entry(ch).or_insert_with(Vec::new).push(position);
            }

            if let Some(current_width) = width {
                if current_width != line.len() as isize {
                    return Err(invalid_input("Mismatched widths"));
                }
            } else {
                width = Some(line.len() as isize);
            }

            height += 1;
        }
        let Some(width) = width else {
            return Err(invalid_input("No lines"));
        };

        Ok(Self {
            antennas,
            width,
            height,
        })
    }

    fn in_bounds(&self, position: Position) -> bool {
        position.row >= 0
            && position.row < self.height
            && position.col >= 0
            && position.col < self.width
    }

    fn in_bounds_antinodes<const BROAD_ANTINODES: bool>(
        &self,
    ) -> HashMap<Position, HashMap<char, usize>> {
        let mut antinodes = HashMap::<Position, HashMap<char, usize>>::new();
        let mut add_antinode = |position, frequency| {
            if self.in_bounds(position) {
                *antinodes
                    .entry(position)
                    .or_insert_with(HashMap::new)
                    .entry(frequency)
                    .or_insert(0) += 1;
            }
        };
        for (&frequency, antennas) in &self.antennas {
            for i in 0..(antennas.len() - 1) {
                for j in (i + 1)..antennas.len() {
                    let antenna_1 = antennas[i];
                    let antenna_2 = antennas[j];
                    let row_diff = antenna_2.row - antenna_1.row;
                    let col_diff = antenna_2.col - antenna_1.col;

                    if BROAD_ANTINODES {
                        let divisor = gcd(row_diff, col_diff);
                        let row_diff = row_diff / divisor;
                        let col_diff = col_diff / divisor;
                        let mut position = antenna_1;
                        while self.in_bounds(position) {
                            add_antinode(position, frequency);
                            position.row -= row_diff;
                            position.col -= col_diff;
                        }
                        position = antenna_1;
                        position.row += row_diff;
                        position.col += col_diff;
                        while self.in_bounds(position) {
                            add_antinode(position, frequency);
                            position.row += row_diff;
                            position.col += col_diff;
                        }
                    } else {
                        let antinode_1 = Position {
                            row: antenna_1.row - row_diff,
                            col: antenna_1.col - col_diff,
                        };
                        let antinode_2 = Position {
                            row: antenna_2.row + row_diff,
                            col: antenna_2.col + col_diff,
                        };
                        add_antinode(antinode_1, frequency);
                        add_antinode(antinode_2, frequency);
                    }
                }
            }
        }
        antinodes
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let map = Map::new(reader)?;

    let result = match part {
        Part::Part1 => map.in_bounds_antinodes::<false>().len(),
        Part::Part2 => map.in_bounds_antinodes::<true>().len(),
    };

    println!("{result}");

    Ok(())
}
