use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::part::Part;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Action {
    Rock,
    Paper,
    Scissors,
}

impl Action {
    fn score(self) -> u64 {
        match self {
            Self::Rock => 1,
            Self::Paper => 2,
            Self::Scissors => 3,
        }
    }

    fn dominator(self) -> Self {
        match self {
            Self::Rock => Self::Paper,
            Self::Paper => Self::Scissors,
            Self::Scissors => Self::Rock,
        }
    }

    fn loser(self) -> Self {
        match self {
            Self::Rock => Self::Scissors,
            Self::Paper => Self::Rock,
            Self::Scissors => Self::Paper,
        }
    }

    fn counter_to_get_outcome(self, outcome: Outcome) -> Self {
        match outcome {
            Outcome::Win => self.dominator(),
            Outcome::Lose => self.loser(),
            Outcome::Draw => self,
        }
    }

    fn from_opponent_str(string: &str) -> Option<Self> {
        match string {
            "A" => Some(Self::Rock),
            "B" => Some(Self::Paper),
            "C" => Some(Self::Scissors),
            _ => None,
        }
    }

    fn from_your_str(string: &str) -> Option<Self> {
        match string {
            "X" => Some(Self::Rock),
            "Y" => Some(Self::Paper),
            "Z" => Some(Self::Scissors),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
enum Outcome {
    Win,
    Lose,
    Draw,
}

impl Outcome {
    fn from_match(you: Action, opponent: Action) -> Self {
        if you == opponent {
            Self::Draw
        } else if you.dominator() == opponent {
            Self::Lose
        } else { // you.loser() == opponent
            Self::Win
        }
    }

    fn score(self) -> u64 {
        match self {
            Self::Win => 6,
            Self::Draw => 3,
            Self::Lose => 0,
        }
    }

    fn from_str(string: &str) -> Option<Self> {
        match string {
            "X" => Some(Self::Lose),
            "Y" => Some(Self::Draw),
            "Z" => Some(Self::Win),
            _ => None,
        }
    }
}

fn score(action: Action, outcome: Outcome) -> u64 {
    action.score() + outcome.score()
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut cur_score: u64 = 0;
    for line in reader.lines() {
        let line = line?;
        let words = line.split(' ').collect::<Vec<&str>>();
        if words.len() != 2 {
            Err(invalid_input("Invalid words (too short)"))?
        }

        let opponent = Action::from_opponent_str(words[0]).ok_or_else(|| {
            invalid_input("Invalid opponent action")
        })?;

        match part {
            Part::Part1 => {
                let you = Action::from_your_str(words[1]).ok_or_else(|| {
                    invalid_input("Invalid action")
                })?;
                cur_score += score(you, Outcome::from_match(you, opponent));
            },
            Part::Part2 => {
                let outcome = Outcome::from_str(words[1]).ok_or_else(|| {
                    invalid_input("Invalid outcome")
                })?;
                cur_score += score(
                    opponent.counter_to_get_outcome(outcome),
                    outcome,
                );
            },
        }
    }
    println!("{}", cur_score);

    Ok(())
}
