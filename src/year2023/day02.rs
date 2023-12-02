use std::collections::HashMap;
use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::part::Part;

struct CubeSet {
    cubes: HashMap<String, u64>,
}

impl CubeSet {
    fn contains(&self, other: &Self) -> bool {
        for (cube, count) in &other.cubes {
            if self.cubes.get(cube) < Some(count) {
                return false;
            }
        }
        true
    }

    fn union(&self, other: &Self) -> Self {
        let mut new_cubes = self.cubes.clone();
        for (cube, count) in &other.cubes {
            let existing_count = new_cubes.get(cube);
            if existing_count < Some(count) {
                new_cubes.insert(cube.to_owned(), *count);
            }
        }
        Self { cubes: new_cubes }
    }

    fn power(&self) -> u64 {
        self.cubes.get("red").unwrap_or(&0)
            * self.cubes.get("green").unwrap_or(&0)
            * self.cubes.get("blue").unwrap_or(&0)
    }
}

struct Game {
    id: u64,
    draws: Vec<CubeSet>,
}

impl Game {
    fn from_line(line: &str) -> io::Result<Self> {
        let [game_id_str, draws_str] =
            &line.split(": ").collect::<Vec<_>>()[..]
        else {
            return Err(invalid_input("No : "));
        };

        let id = game_id_str
            .strip_prefix("Game ")
            .ok_or_else(|| invalid_input("Expected Game "))?
            .parse::<u64>()
            .map_err(invalid_input)?;

        let draws = draws_str
            .split("; ")
            .map(|draw| {
                Ok(CubeSet {
                    cubes: draw
                        .split(", ")
                        .map(|cube_desc| {
                            let [count, name] =
                                cube_desc.split(" ").collect::<Vec<&str>>()[..]
                            else {
                                return Err(invalid_input("No words"));
                            };
                            let count =
                                count.parse::<u64>().map_err(invalid_input)?;
                            Ok((name.to_owned(), count))
                        })
                        .collect::<io::Result<HashMap<String, u64>>>()?,
                })
            })
            .collect::<io::Result<Vec<_>>>()?;

        Ok(Self { id, draws })
    }

    fn possible_from_bag(&self, bag: &CubeSet) -> bool {
        self.draws.iter().all(|draw| bag.contains(draw))
    }

    fn min_cubeset(&self) -> CubeSet {
        self.draws.iter().fold(
            CubeSet {
                cubes: HashMap::new(),
            },
            |acc, x| CubeSet::union(&acc, x),
        )
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut sum: u64 = 0;

    let to_check = CubeSet {
        cubes: HashMap::from([
            ("red".to_owned(), 12),
            ("green".to_owned(), 13),
            ("blue".to_owned(), 14),
        ]),
    };

    for line in reader.lines() {
        let line = line?;
        if line == "" {
            continue;
        }

        let game = Game::from_line(&line)?;

        sum += match part {
            Part::Part1 => {
                if game.possible_from_bag(&to_check) {
                    game.id
                } else {
                    0
                }
            }
            Part::Part2 => game.min_cubeset().power(),
        };
    }

    println!("{sum}");

    Ok(())
}
