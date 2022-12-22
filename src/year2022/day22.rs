use std::io;
use std::io::BufRead;
use std::ops::Add;

use crate::errors::invalid_input;
use crate::part::Part;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tile {
    Open,
    Wall,
    Nothing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    Right = 0,
    Down = 1,
    Left = 2,
    Up = 3,
}

impl Direction {
    pub fn new(ch: char) -> Option<Self> {
        match ch {
            'R' => Some(Self::Right),
            'D' => Some(Self::Down),
            'L' => Some(Self::Left),
            'U' => Some(Self::Up),
            _ => None,
        }
    }
}

impl Add for Direction {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self, other) {
            (_, Self::Up) => self,
            (Self::Up, _) => other,

            (Self::Right, Self::Right) => Self::Down,
            (Self::Down, Self::Right) => Self::Left,
            (Self::Left, Self::Right) => Self::Up,

            (Self::Right, Self::Left) => Self::Up,
            (Self::Down, Self::Left) => Self::Right,
            (Self::Left, Self::Left) => Self::Down,

            // TODO
            (_, Self::Down) => unimplemented!(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Instruction {
    Move(usize),
    Turn(Direction),
}

impl Tile {
    pub fn new(tile_char: char) -> Option<Self> {
        match tile_char {
            '.' => Some(Self::Open),
            '#' => Some(Self::Wall),
            ' ' => Some(Self::Nothing),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Position {
    pub row: usize,
    pub col: usize,
    pub direction: Direction,
}

impl Position {
    pub fn password(self) -> usize {
        1000 * (self.row + 1) + 4 * (self.col + 1) + self.direction as usize
    }
}

struct Map {
    tiles: Vec<Vec<Tile>>,
    pub my_position: Option<Position>,
}

impl Map {
    pub fn new() -> Self {
        Self { tiles: Vec::new(), my_position: None }
    }

    pub fn add_row(&mut self, tiles: Vec<Tile>) {
        if self.my_position.is_none() {
            self.my_position = tiles.iter().enumerate().filter(|(_, tile)| **tile == Tile::Open).map(|(col, _)| col).next().map(|col| Position { row: self.tiles.len(), col, direction: Direction::Right });
        }
        self.tiles.push(tiles);
    }

    fn wrap_up(&mut self) {
        // TODO
        let position = self.my_position.as_mut().expect("Must have position");

        let cur_row = position.row;
        for (row_num, row) in (&self.tiles[position.row..]).iter().enumerate().map(|(i, row)| (i + cur_row, row)) {
            if position.col >= row.len() {
                break;
            }
            match row[position.col] {
                Tile::Open => {
                    position.row = row_num;
                },
                Tile::Wall => {
                    position.row = cur_row;
                },
                Tile::Nothing => {
                    break;
                },
            }
        }
    }

    fn wrap_down(&mut self) {
        // TODO
        let position = self.my_position.as_mut().expect("Must have position");

        let cur_row = position.row;
        for (row_num, row) in (&self.tiles[..position.row]).iter().enumerate().rev() {
            if position.col >= row.len() {
                break;
            }
            match row[position.col] {
                Tile::Open => {
                    position.row = row_num;
                },
                Tile::Wall => {
                    position.row = cur_row;
                },
                Tile::Nothing => {
                    break;
                },
            }
        }
    }

    fn wrap_left(&mut self) {
        // TODO
        let position = self.my_position.as_mut().expect("Must have position");
        let row = &self.tiles[position.row];

        let cur_col = position.col;
        for (col, tile) in (&row[position.col..]).iter().enumerate().map(|(i, tile)| (i + cur_col, tile)) {
            match tile {
                Tile::Open => {
                    position.col = col;
                },
                Tile::Wall => {
                    position.col = cur_col;
                },
                Tile::Nothing => {
                    break;
                },
            }
        }
    }

    fn wrap_right(&mut self) {
        // TODO
        let position = self.my_position.as_mut().expect("Must have position");
        let row = &self.tiles[position.row];

        let cur_col = position.col;
        for (col, tile) in (&row[..position.col]).iter().enumerate().rev() {
            match tile {
                Tile::Open => {
                    position.col = col;
                },
                Tile::Wall => {
                    position.col = cur_col;
                },
                Tile::Nothing => {
                    break;
                },
            }
        }
    }


    fn move_one(&mut self) {
        // TODO
        let position = self.my_position.as_mut().expect("Must have position");

        match position.direction {
            // TODO dup
            Direction::Left => {
                if position.col == 0 {
                    self.wrap_left();
                } else {
                    let tile = self.tiles[position.row][position.col - 1];
                    match tile {
                        Tile::Wall => {
                            // Do nothing
                        }
                        Tile::Nothing => { self.wrap_left(); }
                        Tile::Open => { position.col -= 1; }
                    }
                }
            }
            Direction::Right => {
                let row = &self.tiles[position.row];
                if position.col + 1 >= row.len() {
                    self.wrap_right();
                } else {
                    let tile = self.tiles[position.row][position.col + 1];
                    match tile {
                        Tile::Wall => {
                            // Do nothing
                        }
                        Tile::Nothing => { self.wrap_right(); }
                        Tile::Open => { position.col += 1; }
                    }
                }
            }
            Direction::Up => {
                if position.row == 0 {
                    self.wrap_up();
                } else {
                    let row = &self.tiles[position.row - 1];
                    if position.col >= row.len() {
                        self.wrap_up();
                    } else {
                        let tile = self.tiles[position.row - 1][position.col];
                        match tile {
                            Tile::Wall => {
                                // Do nothing
                            }
                            Tile::Nothing => { self.wrap_up(); }
                            Tile::Open => { position.row -= 1; }
                        }
                    }
                }
            }
            Direction::Down => {
                if position.row + 1 >= self.tiles.len() {
                    self.wrap_down();
                } else {
                    let row = &self.tiles[position.row + 1];
                    if position.col >= row.len() {
                        self.wrap_down();
                    } else {
                        let tile = self.tiles[position.row + 1][position.col];
                        match tile {
                            Tile::Wall => {
                                // Do nothing
                            }
                            Tile::Nothing => { self.wrap_down(); }
                            Tile::Open => { position.row += 1; }
                        }
                    }
                }
            }
        }

    }

    pub fn perform(&mut self, instruction: Instruction) {
        // TODO
        let position = self.my_position.as_mut().expect("Must have position");

        match instruction {
            Instruction::Turn(turn_direction) => {
                position.direction = position.direction + turn_direction;
            }
            Instruction::Move(num_tiles) => {
                for _ in 0..num_tiles {
                    self.move_one();
                }
            }
        }
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut map = Map::new();
    let mut instructions = Vec::<Instruction>::new();

    let mut empty_line_seen = false;

    for line in reader.lines() {
        let line = line?;

        if line == "" {
            empty_line_seen = true;
            continue;
        }

        if !empty_line_seen {
            // Part of the map
            let tiles = line.chars().map(Tile::new).collect::<Option<Vec<_>>>().ok_or_else(|| invalid_input("Invalid tile character"))?;

            map.add_row(tiles);
        } else {
            // The instructions
            // TODO: use option<enum> for parse_state
            // TODO: dup
            let mut parse_state: u8 = 0;
            let mut begin_index: usize = 0;

            for (i, ch) in line.chars().enumerate() {
                let this_parse_state: u8 = if ch.is_ascii_digit() {
                    1
                } else {
                    2
                };

                if parse_state != 0 && parse_state != this_parse_state {
                    // Assume ASCII
                    let string = &line[begin_index..i];
                    match parse_state {
                        1 => {
                            instructions.push(Instruction::Move(string.parse().map_err(invalid_input)?));
                        },
                        2 => {
                            for ch in string.chars() {
                                let direction = Direction::new(ch).ok_or_else(|| invalid_input("Unexpected direction"))?;
                                instructions.push(Instruction::Turn(direction));
                            }
                        },
                        _ => unreachable!()
                    }
                    begin_index = i;
                }
                parse_state = this_parse_state;
            }

            // TODO ASCII
            let string = &line[begin_index..];
            match parse_state {
                1 => {
                    instructions.push(Instruction::Move(string.parse().map_err(invalid_input)?));
                },
                2 => {
                    for ch in string.chars() {
                        let direction = Direction::new(ch).ok_or_else(|| invalid_input("Unexpected direction"))?;
                        instructions.push(Instruction::Turn(direction));
                    }
                },
                _ => unreachable!()
            }
        }
    }

    for instruction in instructions {
        map.perform(instruction);
    }

    // TODO
    println!("{}", map.my_position.unwrap().password());

    Ok(())
}
