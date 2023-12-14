use std::collections::HashMap;
use std::io;

use crate::errors::invalid_input;
use crate::parse::lines;
use crate::part::Part;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum Tile {
    RoundRock,
    CubeRock,
    Empty,
}

impl Tile {
    fn from_char(ch: char) -> io::Result<Self> {
        match ch {
            'O' => Ok(Self::RoundRock),
            '#' => Ok(Self::CubeRock),
            '.' => Ok(Self::Empty),
            _ => Err(invalid_input("Invalid tile character")),
        }
    }
}

struct Platform {
    map: Vec<Vec<Tile>>,
    width: usize,
    height: usize,
}

impl Platform {
    fn from_reader<R: io::Read>(reader: io::BufReader<R>) -> io::Result<Self> {
        let mut map = Vec::<Vec<Tile>>::new();
        let mut width: Option<usize> = None;

        for line in lines(reader)? {
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

        let height = map.len();
        if height == 0 || width.unwrap() == 0 {
            return Err(invalid_input("Empty map"));
        }

        let width = width.unwrap();

        Ok(Self { map, width, height })
    }

    fn tile_at<const IS_UP_DOWN: bool, const IS_UP_OR_LEFT: bool>(
        &mut self,
        main_index: usize,
        falling_index: usize,
    ) -> &mut Tile {
        if IS_UP_DOWN && IS_UP_OR_LEFT {
            &mut self.map[falling_index][main_index]
        } else if IS_UP_DOWN && !IS_UP_OR_LEFT {
            &mut self.map[self.height - falling_index - 1][main_index]
        } else if IS_UP_OR_LEFT {
            assert!(!IS_UP_DOWN);
            &mut self.map[main_index][falling_index]
        } else {
            assert!(!IS_UP_DOWN && !IS_UP_OR_LEFT);
            &mut self.map[main_index][self.width - falling_index - 1]
        }
    }

    fn tilt<const IS_UP_DOWN: bool, const IS_UP_OR_LEFT: bool>(&mut self) {
        let (main_iter_limit, falling_iter_limit) = if IS_UP_DOWN {
            (self.width, self.height)
        } else {
            (self.height, self.width)
        };

        for main_index in 0..main_iter_limit {
            let mut available_falling_index: usize = 0;
            for falling_index in 0..falling_iter_limit {
                match self.tile_at::<IS_UP_DOWN, IS_UP_OR_LEFT>(
                    main_index,
                    falling_index,
                ) {
                    Tile::RoundRock => {
                        if available_falling_index < falling_index {
                            *self.tile_at::<IS_UP_DOWN, IS_UP_OR_LEFT>(
                                main_index,
                                available_falling_index,
                            ) = Tile::RoundRock;
                            *self.tile_at::<IS_UP_DOWN, IS_UP_OR_LEFT>(
                                main_index,
                                falling_index,
                            ) = Tile::Empty;
                        }
                        available_falling_index += 1;
                    }
                    Tile::CubeRock => {
                        available_falling_index = falling_index + 1;
                    }
                    Tile::Empty => {
                        // Do nothing - available falling index stays the same
                    }
                }
            }
        }
    }

    fn tilt_north(&mut self) {
        self.tilt::<true, true>();
    }

    fn tilt_west(&mut self) {
        self.tilt::<false, true>();
    }

    fn tilt_south(&mut self) {
        self.tilt::<true, false>();
    }

    fn tilt_east(&mut self) {
        self.tilt::<false, false>();
    }

    fn tilt_cycle(&mut self) {
        self.tilt_north();
        self.tilt_west();
        self.tilt_south();
        self.tilt_east();
    }

    fn tilt_n_cycles(&mut self, num_cycles: usize) {
        // The idea behind this function is that eventually, we will enter some
        // state were n cycles has no effect. I call this a "cycle of cycles."
        // Once we discover n, we can skip forward in chunks of n knowing that
        // those chunks of cycles have no effect.

        // This map could get pretty big pretty quickly, but for the inputs I've
        // seen, it works fine.
        let mut seen_maps = HashMap::<Vec<Vec<Tile>>, usize>::new();
        for i in 0..num_cycles {
            if let Some(cycle_of_cycles_start_index) = seen_maps.get(&self.map)
            {
                // We've found a cycle of cycles!
                let cycle_of_cycles_length = i - cycle_of_cycles_start_index;
                let cycles_remaining_before_skip = num_cycles - i;
                let cycles_remaining_after_skip =
                    cycles_remaining_before_skip % cycle_of_cycles_length;
                for _ in 0..cycles_remaining_after_skip {
                    self.tilt_cycle();
                }
                return;
            } else {
                seen_maps.insert(self.map.clone(), i);
            }
            self.tilt_cycle();
        }
    }

    fn total_load(&self) -> usize {
        let mut load: usize = 0;
        for (row_index, row) in self.map.iter().enumerate() {
            for tile in row {
                if *tile == Tile::RoundRock {
                    load += self.height - row_index;
                }
            }
        }
        load
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut platform = Platform::from_reader(reader)?;

    match part {
        Part::Part1 => {
            platform.tilt_north();
        }
        Part::Part2 => {
            platform.tilt_n_cycles(1_000_000_000);
        }
    }

    let result = platform.total_load();

    println!("{result}");

    Ok(())
}
