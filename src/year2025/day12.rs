use std::io;

use crate::errors::invalid_input;
use crate::parse::{lines, paragraphs, parse_all, parse_words};
use crate::part::Part;

#[derive(Clone, Debug)]
struct Present {
    layouts: [[[bool; 3]; 3]; 8],
    num_tiles_occupied: usize,
}

impl Present {
    fn new(paragraph: &Vec<String>) -> io::Result<Self> {
        if paragraph.len() != 4 {
            return Err(invalid_input("Expected 4 lines per present"));
        }

        let mut layout = [[false; 3]; 3];
        let mut num_tiles_occupied = 0;
        for row in 0..3 {
            let row_line = &paragraph[row + 1];
            if row_line.len() != 3 {
                return Err(invalid_input("Expected a present width of 3"));
            }

            for (col, ch) in row_line.chars().enumerate() {
                layout[row][col] = match ch {
                    '#' => {
                        num_tiles_occupied += 1;
                        true
                    }
                    '.' => false,
                    _ => {
                        return Err(invalid_input(
                            "Unexpected character in present layout",
                        ));
                    }
                }
            }
        }

        let mut layouts = [[[false; 3]; 3]; 8];
        for row in 0..3 {
            for col in 0..3 {
                let value = layout[row][col];
                layouts[0][row][col] = value; // no change
                layouts[1][2 - row][col] = value; // flip
                layouts[2][col][2 - row] = value; // rotate right
                layouts[3][2 - col][2 - row] = value; // rotate right + flip
                layouts[4][2 - row][2 - col] = value; // rotate 180°
                layouts[5][row][2 - col] = value; // rotate 180° + flip
                layouts[6][2 - col][row] = value; // rotate left
                layouts[7][col][row] = value; // rotate left + flip
            }
        }

        Ok(Self {
            layouts,
            num_tiles_occupied,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Position {
    row: usize,
    col: usize,
}

#[derive(Clone, Copy, Debug)]
struct PackingInfo {
    present_id: usize,
    orientation: usize, /* in range [0, 8) to determine which present layout
                         * we're using */
    position: Position,
}

struct TreeState {
    left_to_pack: Vec<usize>,
    min_tiles_required: usize,
    tiles_available: usize,
    occupied: Vec<Vec<bool>>,
    packed: Vec<PackingInfo>,
}

impl TreeState {
    fn new(
        width: usize,
        height: usize,
        left_to_pack: Vec<usize>,
        presents: &Vec<Present>,
    ) -> Self {
        let to_pack_count = left_to_pack.iter().sum::<usize>();
        let min_tiles_required = left_to_pack
            .iter()
            .zip(presents)
            .map(|(count, present)| count * present.num_tiles_occupied)
            .sum::<usize>();
        Self {
            left_to_pack,
            min_tiles_required,
            tiles_available: width * height,
            occupied: vec![vec![false; width]; height],
            packed: Vec::with_capacity(to_pack_count),
        }
    }

    fn can_fit(&self, layout: &[[bool; 3]; 3], position: Position) -> bool {
        for row_offset in 0..3 {
            for col_offset in 0..3 {
                if !layout[row_offset][col_offset] {
                    continue;
                }

                let row = position.row + row_offset;
                let col = position.col + col_offset;
                if row >= self.occupied.len() || col >= self.occupied[row].len()
                {
                    return false;
                }

                if self.occupied[row][col] {
                    return false;
                }
            }
        }

        true
    }

    fn pack(
        &mut self,
        position: Position,
        presents: &Vec<Present>,
        packing_info: PackingInfo,
    ) {
        let present = &presents[packing_info.present_id];
        let layout = &present.layouts[packing_info.orientation];
        for row_offset in 0..3 {
            for col_offset in 0..3 {
                if !layout[row_offset][col_offset] {
                    continue;
                }

                let row = position.row + row_offset;
                let col = position.col + col_offset;
                assert!(!self.occupied[row][col]);
                self.occupied[row][col] = true;
            }
        }

        self.left_to_pack[packing_info.present_id] -= 1;
        self.min_tiles_required -= present.num_tiles_occupied;
        self.tiles_available -= present.num_tiles_occupied;
        self.packed.push(packing_info);
    }

    fn unpack(&mut self, presents: &Vec<Present>) {
        let packing_info = self.packed.pop().unwrap();
        let present = &presents[packing_info.present_id];
        let layout = &present.layouts[packing_info.orientation];
        for row_offset in 0..3 {
            for col_offset in 0..3 {
                if !layout[row_offset][col_offset] {
                    continue;
                }

                let row = packing_info.position.row + row_offset;
                let col = packing_info.position.col + col_offset;
                assert!(self.occupied[row][col]);
                self.occupied[row][col] = false;
            }
        }
        self.left_to_pack[packing_info.present_id] += 1;
        self.min_tiles_required += present.num_tiles_occupied;
        self.tiles_available += present.num_tiles_occupied;
    }
}
#[derive(Clone, Debug)]
struct TreeSpace {
    width: usize,
    height: usize,
    present_amounts: Vec<usize>,
}

impl TreeSpace {
    fn new(line: &str) -> io::Result<Self> {
        let &[dimensions, presents] = &line.split(": ").collect::<Vec<_>>()[..]
        else {
            return Err(invalid_input("Expected dimension: presents"));
        };

        let &[width, height] = &parse_all(dimensions.split('x'))?[..] else {
            return Err(invalid_input("Expected dimension: presents"));
        };

        let present_amounts = parse_words(presents)?;

        Ok(Self {
            width,
            height,
            present_amounts,
        })
    }

    fn can_be_packed(&self, presents: &Vec<Present>) -> bool {
        self.can_be_packed_helper(
            presents,
            &mut TreeState::new(
                self.width,
                self.height,
                self.present_amounts.clone(),
                presents,
            ),
        )
    }

    fn can_be_packed_helper(
        &self,
        presents: &Vec<Present>,
        state: &mut TreeState,
    ) -> bool {
        let Some((next_present_id, _)) = state
            .left_to_pack
            .iter()
            .copied()
            .enumerate()
            .filter(|&(_, count)| count > 0)
            .next()
        else {
            return true;
        };

        if state.min_tiles_required > state.tiles_available {
            return false;
        }

        let present = &presents[next_present_id];

        for row in 0..self.height {
            for col in 0..self.width {
                let position = Position { row, col };
                for orientation in 0..8 {
                    let layout = &present.layouts[orientation];
                    if state.can_fit(layout, position) {
                        let packing_info = PackingInfo {
                            present_id: next_present_id,
                            orientation,
                            position,
                        };
                        state.pack(position, presents, packing_info);
                        if self.can_be_packed_helper(presents, state) {
                            return true;
                        }
                        state.unpack(presents);
                    }
                }
            }
        }

        false
    }
}

pub fn run<R: io::Read>(
    _part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let the_paragraphs: Vec<_> = paragraphs(lines(reader)?).collect();
    if the_paragraphs.is_empty() {
        return Err(invalid_input("No input given"));
    }

    let presents = (&the_paragraphs[..the_paragraphs.len() - 1])
        .iter()
        .map(Present::new)
        .collect::<io::Result<Vec<_>>>()?;
    let tree_spaces = the_paragraphs
        .last()
        .unwrap()
        .iter()
        .map(|string| TreeSpace::new(string.as_str()))
        .collect::<io::Result<Vec<_>>>()?;

    let result = tree_spaces
        .into_iter()
        .map(|space| space.can_be_packed(&presents))
        .filter(|value| *value)
        .count();

    println!("{result}");

    Ok(())
}
