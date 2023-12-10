use std::collections::{HashSet, VecDeque};
use std::io;

use crate::errors::invalid_input;
use crate::parse::lines;
use crate::part::Part;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    fn reverse(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::East => Self::West,
            Self::West => Self::East,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Location {
    row: isize,
    col: isize,
}

impl Location {
    fn go(self, direction: Direction) -> Self {
        match direction {
            Direction::North => Self {
                row: self.row - 1,
                col: self.col,
            },
            Direction::South => Self {
                row: self.row + 1,
                col: self.col,
            },
            Direction::East => Self {
                row: self.row,
                col: self.col + 1,
            },
            Direction::West => Self {
                row: self.row,
                col: self.col - 1,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tile {
    Vertical,
    Horizontal,
    NorthAndEast,
    NorthAndWest,
    SouthAndWest,
    SouthAndEast,
    Start,
    NoPipe,
}

impl Tile {
    fn pipe_types() -> Vec<Self> {
        vec![
            Self::Vertical,
            Self::Horizontal,
            Self::NorthAndEast,
            Self::NorthAndWest,
            Self::SouthAndWest,
            Self::SouthAndEast,
        ]
    }

    fn accepts_pipe_from(self, direction: Direction) -> bool {
        self.pipe_directions().contains(&direction)
    }

    fn pipe_directions(self) -> Vec<Direction> {
        match self {
            Self::Vertical => vec![Direction::North, Direction::South],
            Self::Horizontal => vec![Direction::East, Direction::West],
            Self::NorthAndEast => vec![Direction::North, Direction::East],
            Self::NorthAndWest => vec![Direction::North, Direction::West],
            Self::SouthAndWest => vec![Direction::South, Direction::West],
            Self::SouthAndEast => vec![Direction::South, Direction::East],
            Self::Start => vec![],
            Self::NoPipe => vec![],
        }
    }

    fn from_char(ch: char) -> io::Result<Self> {
        match ch {
            '|' => Ok(Self::Vertical),
            '-' => Ok(Self::Horizontal),
            'L' => Ok(Self::NorthAndEast),
            'J' => Ok(Self::NorthAndWest),
            '7' => Ok(Self::SouthAndWest),
            'F' => Ok(Self::SouthAndEast),
            'S' => Ok(Self::Start),
            '.' => Ok(Self::NoPipe),
            _ => Err(invalid_input("Invalid tile char")),
        }
    }

    fn to_char(self) -> char {
        match self {
            Self::Vertical => '│',
            Self::Horizontal => '─',
            Self::NorthAndEast => '└',
            Self::NorthAndWest => '┘',
            Self::SouthAndWest => '┐',
            Self::SouthAndEast => '┌',
            Self::Start => 'S',
            Self::NoPipe => '.',
        }
    }
}

struct TileInfo {
    tile: Tile,
    distance: Option<usize>,
    is_part_of_loop: bool,
}

impl TileInfo {
    fn from_char(ch: char) -> io::Result<Self> {
        Ok(Self {
            tile: Tile::from_char(ch)?,
            distance: None,
            is_part_of_loop: false,
        })
    }

    fn to_char(&self) -> char {
        if self.is_part_of_loop {
            self.tile.to_char()
        } else {
            '.'
        }
    }
}

struct PipeGrid {
    width: usize,
    height: usize,
    grid: Vec<Vec<TileInfo>>,
    start_location: Location,
}

impl PipeGrid {
    fn from_reader<R: io::Read>(reader: io::BufReader<R>) -> io::Result<Self> {
        let mut width: Option<usize> = None;
        let mut grid = Vec::<Vec<TileInfo>>::new();
        let mut start_location: Option<Location> = None;
        for line in lines(reader)? {
            let mut grid_line = Vec::<TileInfo>::new();
            for ch in line.chars() {
                let tile_info = TileInfo::from_char(ch)?;
                if tile_info.tile == Tile::Start {
                    if !start_location.is_none() {
                        return Err(invalid_input("Multiple start locations"));
                    }
                    start_location = Some(Location {
                        row: grid.len() as isize,
                        col: grid_line.len() as isize,
                    });
                }
                grid_line.push(tile_info);
            }

            if let Some(the_width) = width {
                if grid_line.len() != the_width {
                    return Err(invalid_input("Differing widths"));
                }
            } else {
                width = Some(grid_line.len());
            }
            grid.push(grid_line);
        }

        let Some(width) = width else {
            return Err(invalid_input("Empty grid"));
        };

        if width == 0 || grid.is_empty() {
            return Err(invalid_input("Empty grid"));
        };

        let Some(start_location) = start_location else {
            return Err(invalid_input("No start location"));
        };

        Ok(Self {
            width,
            height: grid.len(),
            grid,
            start_location,
        })
    }

    fn check_location(&self, location: Location) -> Option<(usize, usize)> {
        if location.row < 0 || location.col < 0 {
            return None;
        }

        let row: usize = location.row.try_into().unwrap();
        let col: usize = location.col.try_into().unwrap();

        if row >= self.height || col >= self.width {
            None
        } else {
            Some((row, col))
        }
    }

    fn at(&self, location: Location) -> Option<&TileInfo> {
        self.check_location(location)
            .map(|(row, col)| &self.grid[row][col])
    }

    fn at_mut(&mut self, location: Location) -> Option<&mut TileInfo> {
        self.check_location(location)
            .map(|(row, col)| &mut self.grid[row][col])
    }

    fn close_loop(&mut self) -> bool {
        let mut success = false;
        for possible_start_pipe in Tile::pipe_types() {
            self.at_mut(self.start_location).unwrap().tile =
                possible_start_pipe;
            let mut checked = HashSet::<Location>::new();
            let mut to_check = Vec::<Location>::new();
            let mut fail = false;
            to_check.push(self.start_location);
            while let Some(checking) = to_check.pop() {
                if checked.contains(&checking) {
                    continue;
                }
                checked.insert(checking);
                let Some(checking_tile) = self.at(checking) else {
                    fail = true;
                    break;
                };
                for neighbor_dir in checking_tile.tile.pipe_directions() {
                    let neighbor_loc = checking.go(neighbor_dir);
                    let Some(neighbor_tile) = self.at(neighbor_loc) else {
                        fail = true;
                        break;
                    };
                    if !neighbor_tile
                        .tile
                        .accepts_pipe_from(neighbor_dir.reverse())
                    {
                        fail = true;
                        break;
                    }
                    to_check.push(neighbor_loc);
                }
            }

            if !fail {
                success = true;
                break;
            }
        }

        if success {
            let mut checked = HashSet::<Location>::new();
            let mut to_check = Vec::<Location>::new();
            to_check.push(self.start_location);
            while let Some(checking) = to_check.pop() {
                if checked.contains(&checking) {
                    continue;
                }
                checked.insert(checking);
                let checking_tile = self.at_mut(checking).unwrap();
                checking_tile.is_part_of_loop = true;
                for neighbor_dir in checking_tile.tile.pipe_directions() {
                    let neighbor_loc = checking.go(neighbor_dir);
                    to_check.push(neighbor_loc);
                }
            }
        } else {
            self.at_mut(self.start_location).unwrap().tile = Tile::Start;
        }

        success
    }

    fn find_max_distance(&mut self) -> usize {
        // close_loop() must be called before this function
        let mut checked = HashSet::<Location>::new();
        let mut to_check = VecDeque::<Location>::new();
        let mut max_distance: usize = 0;
        to_check.push_back(self.start_location);
        self.at_mut(self.start_location).unwrap().distance = Some(0);
        while let Some(checking) = to_check.pop_front() {
            if checked.contains(&checking) {
                continue;
            }
            checked.insert(checking);
            let checking_tile = self.at(checking).unwrap();
            let new_distance = checking_tile.distance.unwrap() + 1;

            for neighbor_dir in checking_tile.tile.pipe_directions() {
                let neighbor_loc = checking.go(neighbor_dir);
                let Some(neighbor_tile) = self.at_mut(neighbor_loc) else {
                    panic!("Call and check close_loop() first please");
                };
                if neighbor_tile.distance.is_none() {
                    to_check.push_back(neighbor_loc);
                    neighbor_tile.distance = Some(new_distance);
                    if new_distance > max_distance {
                        max_distance = new_distance;
                    }
                }
            }
        }
        max_distance
    }

    #[allow(dead_code)]
    fn print(&self) {
        for row in &self.grid {
            for tile in row {
                print!("{}", tile.to_char());
            }
            println!();
        }
    }

    fn num_can_reach_edge(&self) -> usize {
        let mut can_reach_edge = Vec::<Vec<bool>>::new();

        for _ in 0..(self.height * 2) {
            can_reach_edge.push(vec![false; self.width * 2]);
        }

        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        struct NewLoc {
            location: Location,
            north: bool,
            west: bool,
        }

        impl NewLoc {
            fn cre_row_col(self) -> (usize, usize) {
                (
                    self.location.row as usize * 2
                        + (if self.north { 0 } else { 1 }),
                    self.location.col as usize * 2
                        + (if self.west { 0 } else { 1 }),
                )
            }
        }

        let mut checked = HashSet::<NewLoc>::new();
        let mut to_check = VecDeque::<NewLoc>::new();

        fn location_from_new(row: usize, col: usize) -> Location {
            Location {
                row: (row / 2) as isize,
                col: (col / 2) as isize,
            }
        }

        for row in 0..(self.height * 2) {
            to_check.push_back(NewLoc {
                location: location_from_new(row, 0),
                north: row % 2 == 0,
                west: true,
            });
            to_check.push_back(NewLoc {
                location: location_from_new(row, self.width * 2 - 1),
                north: row % 2 == 0,
                west: false,
            });
        }

        for col in 0..(self.width * 2) {
            to_check.push_back(NewLoc {
                location: location_from_new(0, col),
                north: true,
                west: col % 2 == 0,
            });
            to_check.push_back(NewLoc {
                location: location_from_new(self.height * 2 - 1, col),
                north: false,
                west: col % 2 == 0,
            });
        }

        while let Some(checking_loc) = to_check.pop_front() {
            if checked.contains(&checking_loc) {
                continue;
            }
            checked.insert(checking_loc);
            let (cre_row, cre_col) = checking_loc.cre_row_col();
            can_reach_edge[cre_row][cre_col] = true;
            let tile = self.at(checking_loc.location).unwrap();
            let reachable = if tile.is_part_of_loop {
                match (tile.tile, checking_loc.north, checking_loc.west) {
                    (Tile::Vertical, north, west) => vec![(!north, west)],
                    (Tile::Horizontal, north, west) => vec![(north, !west)],
                    (Tile::NorthAndEast, true, false) => vec![],
                    (Tile::NorthAndEast, true, true) => {
                        vec![(false, true), (false, false)]
                    }
                    (Tile::NorthAndEast, false, true) => {
                        vec![(true, true), (false, false)]
                    }
                    (Tile::NorthAndEast, false, false) => {
                        vec![(false, true), (true, true)]
                    }
                    (Tile::NorthAndWest, true, true) => vec![],
                    (Tile::NorthAndWest, true, false) => {
                        vec![(false, true), (false, false)]
                    }
                    (Tile::NorthAndWest, false, true) => {
                        vec![(true, false), (false, false)]
                    }
                    (Tile::NorthAndWest, false, false) => {
                        vec![(false, true), (true, false)]
                    }
                    (Tile::SouthAndWest, false, true) => vec![],
                    (Tile::SouthAndWest, true, false) => {
                        vec![(true, true), (false, false)]
                    }
                    (Tile::SouthAndWest, true, true) => {
                        vec![(true, false), (false, false)]
                    }
                    (Tile::SouthAndWest, false, false) => {
                        vec![(true, true), (true, false)]
                    }
                    (Tile::SouthAndEast, false, false) => vec![],
                    (Tile::SouthAndEast, true, false) => {
                        vec![(true, true), (false, true)]
                    }
                    (Tile::SouthAndEast, true, true) => {
                        vec![(true, false), (false, true)]
                    }
                    (Tile::SouthAndEast, false, true) => {
                        vec![(true, true), (true, false)]
                    }
                    (Tile::Start | Tile::NoPipe, _, _) => {
                        panic!("Start/NoPipe should not be part of loop")
                    }
                }
            } else {
                vec![
                    (!checking_loc.north, checking_loc.west),
                    (checking_loc.north, !checking_loc.west),
                    (!checking_loc.north, !checking_loc.west),
                ]
            };
            for (north, west) in reachable {
                to_check.push_back(NewLoc {
                    location: checking_loc.location,
                    north,
                    west,
                });
            }
            if checking_loc.west && checking_loc.location.col > 0 {
                to_check.push_back(NewLoc {
                    location: checking_loc.location.go(Direction::West),
                    north: checking_loc.north,
                    west: false,
                });
            }
            if !checking_loc.west
                && checking_loc.location.col < self.width as isize - 1
            {
                to_check.push_back(NewLoc {
                    location: checking_loc.location.go(Direction::East),
                    north: checking_loc.north,
                    west: true,
                });
            }
            if checking_loc.north && checking_loc.location.row > 0 {
                to_check.push_back(NewLoc {
                    location: checking_loc.location.go(Direction::North),
                    north: false,
                    west: checking_loc.west,
                });
            }
            if !checking_loc.north
                && checking_loc.location.row < self.height as isize - 1
            {
                to_check.push_back(NewLoc {
                    location: checking_loc.location.go(Direction::South),
                    north: true,
                    west: checking_loc.west,
                });
            }
        }

        let mut num_unreachable: usize = 0;
        for row in 0..self.height {
            for col in 0..self.width {
                let mut is_reachable = false;
                for (north, west) in vec![
                    (true, true),
                    (true, false),
                    (false, true),
                    (false, false),
                ] {
                    let newloc = NewLoc {
                        location: Location {
                            row: row as isize,
                            col: col as isize,
                        },
                        north,
                        west,
                    };
                    let (cre_row, cre_col) = newloc.cre_row_col();
                    if can_reach_edge[cre_row][cre_col] {
                        is_reachable = true;
                        break;
                    }
                }
                if !is_reachable {
                    num_unreachable += 1;
                }
            }
        }

        num_unreachable
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut grid = PipeGrid::from_reader(reader)?;
    if !grid.close_loop() {
        return Err(invalid_input("Uncloseable loop"));
    }

    let result = match part {
        Part::Part1 => grid.find_max_distance(),
        Part::Part2 => grid.num_can_reach_edge(),
    };
    println!("{result}");

    Ok(())
}
