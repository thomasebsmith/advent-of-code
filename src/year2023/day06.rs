use std::io;

use crate::errors::invalid_input;
use crate::iter::join;
use crate::parse::{lines, parse_all};
use crate::part::Part;

// Rounds a float upward to the next highest integer that is not equal to it.
fn ceil_unequal(float: f64) -> i64 {
    (float + f64::EPSILON).ceil() as i64
}

// Rounds a float downward to the next lowest integer that is not equal to it.
fn floor_unequal(float: f64) -> i64 {
    (float - f64::EPSILON).floor() as i64
}

fn number_of_ways_to_win(time: i64, record_distance: i64) -> io::Result<i64> {
    // We want   c^2 - tc + r < 0
    //             where c = charge time, t = time, r = record_distance
    // By the quadratic formula, the zeroes are
    //   (t - sqrt(t^2 - 4r))/2 and
    //   (t + sqrt(t^2 - 4r))/2
    // We want to be between these (exclusive).

    let time_f = time as f64;
    let record_distance_f = record_distance as f64;

    let discriminant = (time_f * time_f - 4.0 * record_distance_f).sqrt();
    if discriminant.is_nan() {
        return Err(invalid_input("Unwinnable race"));
    }

    let lower_bound_f = (time_f - discriminant) / 2.0;
    let upper_bound_f = (time_f + discriminant) / 2.0;

    let lower_bound = ceil_unequal(lower_bound_f).clamp(0, time);
    let upper_bound = floor_unequal(upper_bound_f).clamp(0, time);

    // + 1 because lower_bound and upper_bound are inclusive
    let num_wins = upper_bound - lower_bound + 1;

    Ok(num_wins)
}

fn number_strings(line: &str) -> impl Iterator<Item = &str> {
    line.split_whitespace().skip(1)
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let [time_line, distance_line] = &lines(reader)?.collect::<Vec<_>>()[..]
    else {
        return Err(invalid_input("Expected 2 lines"));
    };

    let (times, distances) = match part {
        Part::Part1 => {
            let times = parse_all::<_, i64>(number_strings(time_line))?;
            let distances = parse_all::<_, i64>(number_strings(distance_line))?;
            (times, distances)
        }
        Part::Part2 => {
            let time = join(number_strings(time_line), "")
                .parse::<i64>()
                .map_err(invalid_input)?;
            let distance = join(number_strings(distance_line), "")
                .parse::<i64>()
                .map_err(invalid_input)?;
            (vec![time], vec![distance])
        }
    };

    if times.len() != distances.len() {
        return Err(invalid_input("Different numbers of times and distances"));
    }

    let mut result: i64 = 1;
    for (time, record_distance) in times.into_iter().zip(distances) {
        result *= number_of_ways_to_win(time, record_distance)?;
    }

    println!("{result}");

    Ok(())
}
