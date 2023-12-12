use std::io;

use crate::errors::invalid_input;
use crate::parse::lines;
use crate::part::Part;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Cell {
    Empty,
    Galaxy,
}

impl Cell {
    fn from_char(ch: char) -> io::Result<Self> {
        match ch {
            '.' => Ok(Self::Empty),
            '#' => Ok(Self::Galaxy),
            _ => Err(invalid_input("Invalid cell character")),
        }
    }
}

struct Image {
    cells: Vec<Vec<Cell>>,
    width: usize,
    height: usize,
    expanded_row_offsets: Vec<usize>,
    expanded_col_offsets: Vec<usize>,
}

impl Image {
    fn from_reader<R: io::Read>(reader: io::BufReader<R>) -> io::Result<Self> {
        let mut width: Option<usize> = None;
        let mut cells = Vec::<Vec<Cell>>::new();
        for line in lines(reader)? {
            let row_of_cells = line
                .chars()
                .map(Cell::from_char)
                .collect::<io::Result<Vec<_>>>()?;
            if let Some(the_width) = width {
                if row_of_cells.len() != the_width {
                    return Err(invalid_input("Differing widths"));
                }
            } else {
                width = Some(row_of_cells.len());
            }
            cells.push(row_of_cells);
        }

        let height = cells.len();
        if height == 0 {
            return Err(invalid_input("Empty image"));
        }

        let width = width.unwrap();
        if width == 0 {
            return Err(invalid_input("Empty image"));
        }

        let expanded_row_offsets = (0..width).collect::<Vec<usize>>();
        let expanded_col_offsets = (0..height).collect::<Vec<usize>>();

        Ok(Self {
            cells,
            width,
            height,
            expanded_row_offsets,
            expanded_col_offsets,
        })
    }

    fn expand(&mut self, factor: usize) {
        assert!(factor != 0);
        assert!(self.width > 0);
        assert!(self.height > 0);

        let to_add_per_empty_space = factor - 1;

        // Expand rows
        let mut expanded_rows_so_far: usize = 0;
        for (row_index, row) in self.cells.iter().enumerate() {
            self.expanded_row_offsets[row_index] += expanded_rows_so_far;
            if row.iter().all(|cell| *cell == Cell::Empty) {
                expanded_rows_so_far += to_add_per_empty_space;
            }
        }

        // Expand columns
        let mut expanded_cols_so_far: usize = 0;
        for col_index in 0..self.width {
            self.expanded_col_offsets[col_index] += expanded_cols_so_far;

            let mut should_duplicate = true;
            for row in &self.cells {
                if row[col_index] != Cell::Empty {
                    should_duplicate = false;
                    break;
                }
            }

            if should_duplicate {
                expanded_cols_so_far += to_add_per_empty_space;
            }
        }
    }

    fn galaxy_locations(&self) -> Vec<(usize, usize)> {
        let mut galaxy_locations = Vec::<(usize, usize)>::new();
        for (row_idx, row) in self.cells.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                if *cell == Cell::Galaxy {
                    galaxy_locations.push((row_idx, col_idx));
                }
            }
        }

        galaxy_locations
    }

    fn expanded(&self, (row, col): (usize, usize)) -> (usize, usize) {
        (
            self.expanded_row_offsets[row],
            self.expanded_col_offsets[col],
        )
    }

    fn min_distance_pairwise_sum(&self) -> usize {
        let mut sum: usize = 0;

        let galaxy_locations = self.galaxy_locations();
        for i in 0..galaxy_locations.len() {
            for j in (i + 1)..galaxy_locations.len() {
                let (row_one, col_one) = self.expanded(galaxy_locations[i]);
                let (row_two, col_two) = self.expanded(galaxy_locations[j]);
                sum += row_one.abs_diff(row_two) + col_one.abs_diff(col_two);
            }
        }

        sum
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let expansion_factor = match part {
        Part::Part1 => 2,
        Part::Part2 => 1_000_000,
    };
    let mut image = Image::from_reader(reader)?;
    image.expand(expansion_factor);

    let result = image.min_distance_pairwise_sum();

    println!("{result}");

    Ok(())
}
