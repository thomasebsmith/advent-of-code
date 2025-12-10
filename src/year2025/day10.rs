use std::cmp::Reverse;
use std::io;
use std::io::BufRead;
// use std::ops::{AddAssign, Mul};

use crate::errors::invalid_input;
use crate::parse::parse_all;
use crate::part::Part;

type BitSet = Vec<bool>;

type Joltage = u16;

const MAX_NUM_JOLTAGES: usize = 10;

type JoltageArray = [Joltage; MAX_NUM_JOLTAGES];

fn xor(vec1: &mut BitSet, vec2: &BitSet) {
    assert_eq!(vec1.len(), vec2.len());
    for i in 0..vec1.len() {
        vec1[i] ^= vec2[i];
    }
}

/*
fn add<U: Copy, T: AddAssign<U>>(vec1: &mut Vec<T>, vec2: &Vec<U>) {
    assert_eq!(vec1.len(), vec2.len());
    for i in 0..vec1.len() {
        vec1[i] += vec2[i];
    }
}

fn any_greater<T: Ord>(vec1: &Vec<T>, vec2: &Vec<T>) -> bool {
    assert_eq!(vec1.len(), vec2.len());
    for i in 0..vec1.len() {
        if vec1[i] > vec2[i] {
            return true;
        }
    }
    false
}*/

fn add(arr1: &mut JoltageArray, arr2: &JoltageArray) {
    for i in 0..MAX_NUM_JOLTAGES {
        arr1[i] += arr2[i];
    }
}

fn add_mult(arr1: &mut JoltageArray, arr2: &JoltageArray, mult: Joltage) {
    for i in 0..MAX_NUM_JOLTAGES {
        arr1[i] += mult * arr2[i];
    }
}

fn any_greater(arr1: &JoltageArray, arr2: &JoltageArray) -> bool {
    for i in 0..MAX_NUM_JOLTAGES {
        if arr1[i] > arr2[i] {
            return true;
        }
    }
    false
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct LightingSearchState {
    buttons_considered: usize,
    button_presses: usize,
    lights: BitSet,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct JoltageSearchState {
    button_presses: usize,
    next_button: usize,
    joltages: JoltageArray,
}

struct Machine {
    lighting_diagram: BitSet,
    lighting_buttons: Vec<BitSet>,
    joltage_buttons: Vec<JoltageArray>,
    joltages: JoltageArray,
}

impl Machine {
    fn from_line(line: &str) -> io::Result<Self> {
        let components = line.split_whitespace().collect::<Vec<_>>();
        if components.len() < 3 {
            return Err(invalid_input(
                "Expected a lighting diagram, at least one button, and joltages",
            ));
        }

        let diagram_str = components[0];
        let button_strs = &components[1..components.len() - 1];
        let joltages_str = components[components.len() - 1];

        if !diagram_str.starts_with('[') || !diagram_str.ends_with(']') {
            return Err(invalid_input(
                "Expected lighting diagram to be wrapped in square brackets",
            ));
        }

        let lighting_diagram = (&diagram_str[1..diagram_str.len() - 1])
            .chars()
            .map(|ch| match ch {
                '.' => Ok(false),
                '#' => Ok(true),
                _ => Err(invalid_input(
                    "Expected lighting diagram to have . or #",
                )),
            })
            .collect::<io::Result<BitSet>>()?;

        if !joltages_str.starts_with('{') || !joltages_str.ends_with('}') {
            return Err(invalid_input(
                "Expected joltages to be wrapped in curly braces",
            ));
        }

        let joltages_vec = parse_all::<_, Joltage>(
            (&joltages_str[1..joltages_str.len() - 1]).split(','),
        )?;
        if joltages_vec.len() > MAX_NUM_JOLTAGES {
            return Err(invalid_input(format!(
                "Expected at most {MAX_NUM_JOLTAGES} voltages"
            )));
        }

        let mut joltages = [0; MAX_NUM_JOLTAGES];
        for (i, joltage) in joltages_vec.into_iter().enumerate() {
            joltages[i] = joltage;
        }

        let (lighting_buttons, joltage_buttons) = button_strs
            .into_iter()
            .map(|button_str| {
                if !button_str.starts_with('(') || !button_str.ends_with(')') {
                    return Err(invalid_input(
                        "Expected button string to be wrapped in parentheses",
                    ));
                }

                let mut lighting_button = vec![false; lighting_diagram.len()];
                let mut joltage_button = [0; MAX_NUM_JOLTAGES];
                let ids = parse_all::<_, usize>(
                    (&button_str[1..button_str.len() - 1]).split(','),
                )?;
                if ids.is_empty() {
                    return Err(invalid_input("Expected a non-empty button"));
                }
                for id in ids {
                    if id >= lighting_button.len() || id >= MAX_NUM_JOLTAGES {
                        return Err(invalid_input("Out-of-range ID"));
                    }
                    lighting_button[id] = !lighting_button[id];
                    joltage_button[id] += 1;
                }

                Ok((lighting_button, joltage_button))
            })
            .collect::<io::Result<(Vec<BitSet>, Vec<JoltageArray>)>>()?;

        Ok(Self {
            lighting_diagram,
            lighting_buttons,
            joltage_buttons,
            joltages,
        })
    }

    fn fewest_button_presses_diagram(&self) -> Option<usize> {
        let mut best_result = usize::MAX;
        let mut states = Vec::<LightingSearchState>::new();
        states.push(LightingSearchState {
            buttons_considered: 0,
            button_presses: 0,
            lights: vec![false; self.lighting_diagram.len()],
        });

        while let Some(mut state) = states.pop() {
            if state.lights == self.lighting_diagram {
                if state.button_presses < best_result {
                    best_result = state.button_presses;
                }
                continue;
            }
            if state.button_presses >= best_result {
                continue;
            }
            if state.buttons_considered == self.lighting_buttons.len() {
                continue;
            }

            state.buttons_considered += 1;

            let mut state_press = state.clone();
            state_press.button_presses += 1;
            xor(
                &mut state_press.lights,
                &self.lighting_buttons[state_press.buttons_considered - 1],
            );
            states.push(state_press);

            let state_no_press = state;
            states.push(state_no_press);
        }

        if best_result == usize::MAX {
            None
        } else {
            Some(best_result)
        }
    }

    fn fewest_button_presses_joltage(&self) -> Option<usize> {
        let mut buttons = self.joltage_buttons.clone();
        buttons.sort_by_key(|button| Reverse(button.iter().sum::<Joltage>()));

        let mut button_id_to_last_joltage_ids =
            vec![vec![0usize; 0]; buttons.len()];
        {
            let mut taken = [false; MAX_NUM_JOLTAGES];
            for (button_id, button) in buttons.iter().enumerate().rev() {
                for joltage_id in 0..MAX_NUM_JOLTAGES {
                    if button[joltage_id] == 1 && !taken[joltage_id] {
                        button_id_to_last_joltage_ids[button_id]
                            .push(joltage_id);
                        taken[joltage_id] = true;
                    }
                }
            }
        }

        let mut best_result = usize::MAX;
        let mut states = Vec::<JoltageSearchState>::new();
        states.push(JoltageSearchState {
            button_presses: 0,
            next_button: 0,
            joltages: [0; MAX_NUM_JOLTAGES],
        });

        while let Some(mut state) = states.pop() {
            if state.button_presses >= best_result {
                continue;
            }
            if any_greater(&state.joltages, &self.joltages) {
                continue;
            }
            if state.next_button >= buttons.len() {
                continue;
            }
            if state.joltages == self.joltages {
                if state.button_presses < best_result {
                    best_result = state.button_presses;
                }
                continue;
            }

            let last_joltage_ids =
                &button_id_to_last_joltage_ids[state.next_button];
            if !last_joltage_ids.is_empty() {
                for &joltage_id in last_joltage_ids {
                    let diff =
                        self.joltages[joltage_id] - state.joltages[joltage_id];
                    state.button_presses += diff as usize;
                    add_mult(
                        &mut state.joltages,
                        &buttons[state.next_button],
                        diff,
                    );
                }
                state.next_button += 1;
                if state.next_button == buttons.len() {
                    if state.joltages == self.joltages {
                        if state.button_presses < best_result {
                            best_result = state.button_presses;
                        }
                    }
                } else {
                    states.push(state);
                }
                continue;
            }

            let mut state_move = state.clone();
            state_move.next_button += 1;
            states.push(state_move);

            state.button_presses += 1;
            add(&mut state.joltages, &buttons[state.next_button]);
            states.push(state);
        }

        // Uncomment for intermediate results:
        // println!("Final result: {best_result}");
        if best_result == usize::MAX {
            None
        } else {
            Some(best_result)
        }
    }
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let machines = reader
        .lines()
        .map(|line| Machine::from_line(&line?))
        .collect::<io::Result<Vec<_>>>()?;

    let solution_func = match part {
        Part::Part1 => Machine::fewest_button_presses_diagram,
        Part::Part2 => Machine::fewest_button_presses_joltage,
    };

    let result = machines
        .iter()
        .map(solution_func)
        .sum::<Option<usize>>()
        .ok_or_else(|| invalid_input("Unable to find solution for machine"))?;
    println!("{result}");

    Ok(())
}
