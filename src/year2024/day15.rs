use std::collections::VecDeque;
use std::io;

use crate::errors::invalid_input;
use crate::parse::{lines, paragraphs};
use crate::part::Part;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Position {
    row: isize,
    col: isize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Direction {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

impl Direction {
    #[allow(dead_code)]
    const ALL: [Self; 4] = [Self::Up, Self::Down, Self::Left, Self::Right];
}

impl Position {
    fn move_one(self, direction: Direction) -> Self {
        let row = self.row;
        let col = self.col;
        match direction {
            Direction::Up => Self { row: row - 1, col },
            Direction::Down => Self { row: row + 1, col },
            Direction::Left => Self { row, col: col - 1 },
            Direction::Right => Self { row, col: col + 1 },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Cell {
    Empty,
    Wall,
    Box,
    LeftBox,
    RightBox,
}

struct Warehouse {
    layout: Vec<Vec<Cell>>,
    width: isize,
    height: isize,
    robot_position: Position,
    moves: VecDeque<Direction>,
}

impl Warehouse {
    fn new<R: io::Read>(
        reader: io::BufReader<R>,
        part: Part,
    ) -> io::Result<Self> {
        let the_paragraphs = paragraphs(lines(reader)?).collect::<Vec<_>>();
        let [map_lines, move_lines] = &the_paragraphs[..] else {
            return Err(invalid_input("Expected map then moves"));
        };
        let mut width: Option<isize> = None;
        let mut layout: Vec<Vec<Cell>> = Vec::new();
        let mut robot_position: Option<Position> = None;
        for (row, line) in map_lines.into_iter().enumerate() {
            let mut line_layout: Vec<Cell> = Vec::new();
            for (col, ch) in line.chars().enumerate() {
                match part {
                    Part::Part1 => {
                        let position = Position {
                            row: row as isize,
                            col: col as isize,
                        };
                        line_layout.push(match ch {
                            '.' => Cell::Empty,
                            '#' => Cell::Wall,
                            'O' => Cell::Box,
                            '@' => {
                                if robot_position.is_some() {
                                    return Err(invalid_input(
                                        "Multiple robots",
                                    ));
                                }
                                robot_position = Some(position);
                                Cell::Empty
                            }
                            _ => return Err(invalid_input("Unknown cell")),
                        });
                    }
                    Part::Part2 => {
                        let position = Position {
                            row: row as isize,
                            col: col as isize * 2,
                        };
                        line_layout.extend(match ch {
                            '.' => [Cell::Empty, Cell::Empty],
                            '#' => [Cell::Wall, Cell::Wall],
                            'O' => [Cell::LeftBox, Cell::RightBox],
                            '@' => {
                                if robot_position.is_some() {
                                    return Err(invalid_input(
                                        "Multiple robots",
                                    ));
                                }
                                robot_position = Some(position);
                                [Cell::Empty, Cell::Empty]
                            }
                            _ => return Err(invalid_input("Unknown cell")),
                        });
                    }
                }
            }
            if let Some(current_width) = width {
                if current_width != line_layout.len() as isize {
                    return Err(invalid_input("Mismatched widths"));
                }
            } else {
                width = Some(line_layout.len() as isize);
            }

            layout.push(line_layout);
        }

        let Some(width) = width else {
            return Err(invalid_input("No lines"));
        };
        let height = layout.len() as isize;

        let Some(robot_position) = robot_position else {
            return Err(invalid_input("No robot"));
        };

        let mut moves: VecDeque<Direction> = VecDeque::new();
        for line in move_lines {
            for move_char in line.chars() {
                moves.push_back(match move_char {
                    '^' => Direction::Up,
                    'v' => Direction::Down,
                    '<' => Direction::Left,
                    '>' => Direction::Right,
                    _ => return Err(invalid_input("Unknown move")),
                });
            }
        }

        Ok(Self {
            layout,
            width,
            height,
            robot_position,
            moves,
        })
    }

    fn in_bounds(&self, position: Position) -> bool {
        position.row >= 0
            && position.row < self.height
            && position.col >= 0
            && position.col < self.width
    }

    fn at(&self, position: Position) -> Option<Cell> {
        if !self.in_bounds(position) {
            None
        } else {
            Some(self.layout[position.row as usize][position.col as usize])
        }
    }

    fn at_mut(&mut self, position: Position) -> Option<&mut Cell> {
        if !self.in_bounds(position) {
            None
        } else {
            Some(&mut self.layout[position.row as usize][position.col as usize])
        }
    }

    fn can_clear(&self, position: Position, direction: Direction) -> bool {
        let Some(cell) = self.at(position) else {
            return false;
        };
        match cell {
            Cell::Wall => false,
            Cell::Empty => true,
            Cell::Box => {
                self.can_clear(position.move_one(direction), direction)
            }
            Cell::LeftBox | Cell::RightBox => {
                let left_position = if cell == Cell::LeftBox {
                    position
                } else {
                    position.move_one(Direction::Left)
                };
                let right_position = left_position.move_one(Direction::Right);

                match direction {
                    Direction::Left => self.can_clear(
                        left_position.move_one(direction),
                        direction,
                    ),
                    Direction::Right => self.can_clear(
                        right_position.move_one(direction),
                        direction,
                    ),
                    Direction::Up | Direction::Down => {
                        self.can_clear(
                            left_position.move_one(direction),
                            direction,
                        ) && self.can_clear(
                            right_position.move_one(direction),
                            direction,
                        )
                    }
                }
            }
        }
    }

    fn clear(&mut self, position: Position, direction: Direction) -> bool {
        let Some(cell) = self.at(position) else {
            return false;
        };
        match cell {
            Cell::Wall => false,
            Cell::Empty => true,
            Cell::Box => {
                if !self.clear(position.move_one(direction), direction) {
                    false
                } else {
                    *self.at_mut(position.move_one(direction)).unwrap() =
                        Cell::Box;
                    *self.at_mut(position).unwrap() = Cell::Empty;
                    true
                }
            }
            Cell::LeftBox | Cell::RightBox => {
                let left_position = if cell == Cell::LeftBox {
                    position
                } else {
                    position.move_one(Direction::Left)
                };
                let right_position = left_position.move_one(Direction::Right);

                let new_left_position = left_position.move_one(direction);
                let new_right_position = right_position.move_one(direction);

                match direction {
                    Direction::Left => {
                        if !self.clear(new_left_position, direction) {
                            false
                        } else {
                            *self.at_mut(new_left_position).unwrap() =
                                Cell::LeftBox;
                            *self.at_mut(new_right_position).unwrap() =
                                Cell::RightBox;
                            *self.at_mut(right_position).unwrap() = Cell::Empty;
                            true
                        }
                    }
                    Direction::Right => {
                        if !self.clear(new_right_position, direction) {
                            false
                        } else {
                            *self.at_mut(new_right_position).unwrap() =
                                Cell::RightBox;
                            *self.at_mut(new_left_position).unwrap() =
                                Cell::LeftBox;
                            *self.at_mut(left_position).unwrap() = Cell::Empty;
                            true
                        }
                    }
                    Direction::Up | Direction::Down => {
                        if !self.can_clear(new_left_position, direction)
                            || !self.clear(new_right_position, direction)
                        {
                            false
                        } else {
                            self.clear(new_left_position, direction);
                            *self.at_mut(new_left_position).unwrap() =
                                Cell::LeftBox;
                            *self.at_mut(new_right_position).unwrap() =
                                Cell::RightBox;
                            *self.at_mut(left_position).unwrap() = Cell::Empty;
                            *self.at_mut(right_position).unwrap() = Cell::Empty;
                            true
                        }
                    }
                }
            }
        }
    }

    fn simulate(&mut self) {
        while let Some(direction) = self.moves.pop_front() {
            let possible_new_position = self.robot_position.move_one(direction);
            if self.clear(possible_new_position, direction) {
                self.robot_position = possible_new_position;
            }
        }
    }

    fn gps_coordinate_sum(&self) -> isize {
        let mut sum = 0isize;
        for row in 0..self.height {
            for col in 0..self.width {
                let position = Position { row, col };
                if matches!(self.at(position), Some(Cell::Box | Cell::LeftBox))
                {
                    sum += position.row * 100 + position.col;
                }
            }
        }
        sum
    }

    #[allow(dead_code)]
    fn print(&self) {
        for row in 0..self.height {
            for col in 0..self.width {
                let position = Position { row, col };
                if position == self.robot_position {
                    print!("@");
                } else {
                    print!(
                        "{}",
                        match self.at(position).unwrap() {
                            Cell::Empty => '.',
                            Cell::Wall => '#',
                            Cell::Box => 'O',
                            Cell::LeftBox => '[',
                            Cell::RightBox => ']',
                        }
                    );
                }
            }
            println!();
        }
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut warehouse = Warehouse::new(reader, part)?;

    warehouse.simulate();

    let result = warehouse.gps_coordinate_sum();

    println!("{result}");

    Ok(())
}
