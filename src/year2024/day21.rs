use std::collections::{HashMap, HashSet, VecDeque};
use std::io;
use std::io::BufRead;
use std::iter::once;

use crate::errors::invalid_input;
use crate::part::Part;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Position {
    row: isize,
    col: isize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Direction {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

impl Direction {
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

type Key = char;

struct Keypad {
    key_by_position: HashMap<Position, Key>,
    position_by_key: HashMap<Key, Position>,
}

impl Keypad {
    fn new(layout: Vec<Vec<Option<Key>>>) -> Self {
        let mut key_by_position = HashMap::<Position, Key>::new();
        let mut position_by_key = HashMap::<Key, Position>::new();
        for (row, layout_row) in layout.into_iter().enumerate() {
            for (col, maybe_key) in layout_row.into_iter().enumerate() {
                if let Some(key) = maybe_key {
                    let position = Position {
                        row: row as isize,
                        col: col as isize,
                    };
                    key_by_position.insert(position, key);
                    if position_by_key.insert(key, position).is_some() {
                        panic!("Duplicate key");
                    }
                }
            }
        }

        Keypad {
            key_by_position,
            position_by_key,
        }
    }

    fn at(&self, position: Position) -> Option<Key> {
        self.key_by_position.get(&position).copied()
    }

    fn get_shortest_paths(
        &self,
        from: Position,
        to: Position,
    ) -> Vec<Vec<Direction>> {
        let mut min_steps = usize::MAX;
        let mut possible_paths = Vec::<Vec<Direction>>::new();
        let mut paths = VecDeque::<(Position, Vec<Direction>)>::new();
        paths.push_back((from, Vec::new()));
        let mut visited = HashSet::<Position>::new();
        while let Some((pos, path)) = paths.pop_front() {
            visited.insert(pos);

            if path.len() > min_steps {
                break;
            }
            if pos == to {
                if path.len() < min_steps {
                    min_steps = path.len();
                }
                possible_paths.push(path);
                continue;
            }
            let neighbors =
                Direction::ALL.into_iter().filter_map(|direction| {
                    let new_pos = pos.move_one(direction);
                    if self.at(new_pos).is_some() && !visited.contains(&new_pos)
                    {
                        let mut new_path = path.clone();
                        new_path.push(direction);
                        Some((new_pos, new_path))
                    } else {
                        None
                    }
                });
            paths.extend(neighbors);
        }

        possible_paths
    }
}

struct Memoizer {
    numeric_keypad: Keypad,
    directional_keypad: Keypad,
    num_directional_keypads: isize,
    cache: HashMap<(Position, Position, isize), isize>,
}

impl Memoizer {
    fn new(
        numeric_keypad: Keypad,
        directional_keypad: Keypad,
        num_directional_keypads: isize,
    ) -> Self {
        Self {
            numeric_keypad,
            directional_keypad,
            num_directional_keypads,
            cache: HashMap::new(),
        }
    }

    fn get_keypad(&self, layer: isize) -> &Keypad {
        if layer == 0 {
            &self.numeric_keypad
        } else {
            &self.directional_keypad
        }
    }

    fn compute(&mut self, key: (Position, Position, isize)) -> isize {
        let (start, end, layer) = key;

        if layer == self.num_directional_keypads + 1 {
            return 1;
        }

        let controlled = self.get_keypad(layer - 1);
        let controlling = self.get_keypad(layer);

        let left_pos = *controlling.position_by_key.get(&'<').unwrap();
        let right_pos = *controlling.position_by_key.get(&'>').unwrap();
        let up_pos = *controlling.position_by_key.get(&'^').unwrap();
        let down_pos = *controlling.position_by_key.get(&'v').unwrap();
        let activate_pos = *controlling.position_by_key.get(&'A').unwrap();

        let mut min_result = isize::MAX;
        for spp in controlled.get_shortest_paths(start, end) {
            let path = once(activate_pos)
                .chain(spp.into_iter().map(|direction| match direction {
                    Direction::Up => up_pos,
                    Direction::Down => down_pos,
                    Direction::Left => left_pos,
                    Direction::Right => right_pos,
                }))
                .chain(once(activate_pos))
                .collect::<Vec<_>>();
            let mut result = 0isize;
            for window in path.windows(2) {
                let begin = window[0];
                let finish = window[1];
                result += self.get((begin, finish, layer + 1));
            }
            if result <= 0 {
                panic!("result is {result} (path len={})", path.len());
            }
            if result < min_result {
                min_result = result;
            }
        }
        min_result
    }

    fn get(&mut self, key: (Position, Position, isize)) -> isize {
        if let Some(&result) = self.cache.get(&key) {
            result
        } else {
            let computed = self.compute(key);
            self.cache.insert(key, computed);
            computed
        }
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let numeric_keypad = Keypad::new(vec![
        vec![Some('7'), Some('8'), Some('9')],
        vec![Some('4'), Some('5'), Some('6')],
        vec![Some('1'), Some('2'), Some('3')],
        vec![None, Some('0'), Some('A')],
    ]);
    let directional_keypad = Keypad::new(vec![
        vec![None, Some('^'), Some('A')],
        vec![Some('<'), Some('v'), Some('>')],
    ]);

    let num_directional_keypads = match part {
        Part::Part1 => 3,
        Part::Part2 => 26,
    };
    let mut memoizer = Memoizer::new(
        numeric_keypad,
        directional_keypad,
        num_directional_keypads,
    );

    let mut result = 0isize;

    for code in reader.lines() {
        let code = code?;
        if code.len() != 4 || code.chars().last().unwrap() != 'A' {
            panic!("Expected ...A");
        }
        let num = (&code[0..3]).parse::<isize>().map_err(invalid_input)?;

        let mut positions = code
            .chars()
            .map(|ch| {
                *memoizer.numeric_keypad.position_by_key.get(&ch).unwrap()
            })
            .collect::<Vec<_>>();
        positions.insert(
            0,
            *memoizer.numeric_keypad.position_by_key.get(&'A').unwrap(),
        );
        let mut num_positions = 0isize;
        for window in positions.windows(2) {
            let begin = window[0];
            let finish = window[1];
            num_positions += memoizer.get((begin, finish, 1));
        }
        result += num * num_positions;
    }

    println!("{result}");

    Ok(())
}
