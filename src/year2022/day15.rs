use std::collections::LinkedList;
use std::io;
use std::io::BufRead;
use std::ops::Sub;
use std::str::FromStr;

use crate::errors::invalid_input;
use crate::part::Part;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Vector2D {
    pub x: i64,
    pub y: i64,
}

impl Vector2D {
    fn manhattan_len(self) -> i64 {
        self.x.abs() + self.y.abs()
    }

    fn manhattan_distance(self, other: Self) -> i64 {
        (self - other).manhattan_len()
    }
}

impl Sub for Vector2D {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl FromStr for Vector2D {
    type Err = io::Error;

    fn from_str(string: &str) -> io::Result<Self> {
        let [x_text, y_text] = &string.split(", ").collect::<Vec<_>>()[..] else {
            Err(invalid_input("Could not split by \", \" into 2 components"))?
        };

        let x = x_text
            .strip_prefix("x=")
            .ok_or_else(|| invalid_input("Expected \"x=\""))?
            .parse::<i64>()
            .map_err(invalid_input)?;

        let y = y_text
            .strip_prefix("y=")
            .ok_or_else(|| invalid_input("Expected \"y=\""))?
            .parse::<i64>()
            .map_err(invalid_input)?;

        Ok(Self { x, y })
    }
}

struct Sensor {
    pub location: Vector2D,
    pub closest_beacon_location: Vector2D,
}

impl Sensor {
    pub fn no_beacon_segment(&self, row_y: i64) -> Option<HorizontalSegment> {
        self.no_distress_beacon_segment(row_y).map(|segment| {
            let mut segment = segment;
            if self.closest_beacon_location.y == row_y {
                segment.len -= 1;
                if self.closest_beacon_location.x < self.location.x {
                    segment.x += 1;
                }
            }
            segment
        })
    }

    pub fn no_distress_beacon_segment(&self, row_y: i64) -> Option<HorizontalSegment> {
        let max_distance = self.location.manhattan_distance(
            self.closest_beacon_location,
        );

        let distance_to_row = (self.location.y - row_y).abs();

        let no_beacon_radius_in_row = max_distance - distance_to_row;

        if no_beacon_radius_in_row < 0 {
            return None;
        }

        let x = self.location.x - no_beacon_radius_in_row;
        let len = no_beacon_radius_in_row * 2 + 1;

        Some(HorizontalSegment { x, len })
    }
}

impl FromStr for Sensor {
    type Err = io::Error;

    fn from_str(string: &str) -> io::Result<Self> {
        let [
            sensor_location_text,
            beacon_location_text,
        ] = &string.split(": ").collect::<Vec<_>>()[..] else {
            Err(invalid_input("Could not split by \": \" into 2 components"))?
        };

        let location = sensor_location_text
            .strip_prefix("Sensor at ")
            .ok_or_else(|| invalid_input("Expected \"Sensor at \""))?
            .parse::<Vector2D>()?;

        let closest_beacon_location = beacon_location_text
            .strip_prefix("closest beacon is at ")
            .ok_or_else(|| invalid_input("Expected \"closest beacon is at \""))?
            .parse::<Vector2D>()?;

        Ok(Self { location, closest_beacon_location })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HorizontalSegment {
    pub x: i64,
    pub len: i64,
}

struct LineSegments {
    segments: LinkedList<HorizontalSegment>,
}

impl LineSegments {
    pub fn new(segment: HorizontalSegment) -> Self {
        Self {
            segments: LinkedList::from([segment]),
        }
    }

    pub fn summed_len(&self) -> i64 {
        self.segments.iter().map(|segment| segment.len).sum::<i64>()
    }

    pub fn into_segments(self) -> LinkedList<HorizontalSegment> {
        self.segments
    }

    pub fn remove(&mut self, to_remove: HorizontalSegment) {
        let mut cursor = self.segments.cursor_front_mut();
        while !cursor.index().is_none() {
            let segment = *cursor.current().unwrap();

            if segment.x + segment.len <= to_remove.x {
                cursor.move_next();
                continue;
            }

            if to_remove.x + to_remove.len <= segment.x {
                break;
            }

            // We have stuff to remove
            let first_segment = if segment.x < to_remove.x {
                let leftover_beginning_len = to_remove.x - segment.x;
                Some(HorizontalSegment { x: segment.x, len: leftover_beginning_len })
            } else {
                None
            };

            let second_segment = if to_remove.x + to_remove.len < segment.x + segment.len {
                let leftover_end_len = segment.x + segment.len - (to_remove.x + to_remove.len);
                Some(HorizontalSegment { x: to_remove.x + to_remove.len, len: leftover_end_len })
            } else {
                None
            };

            cursor.remove_current(); // Also moves cursor to next node

            for segment in [first_segment, second_segment] {
                if let Some(segment) = segment {
                    cursor.insert_before(segment);
                }
            }
        }
    }
}

fn part_1(sensors: Vec<Sensor>) -> io::Result<()> {
    const ROW_Y: i64 = 2_000_000;

    const ROW_X: i64 = -5_000_000;
    const ROW_LEN: i64 = 10_000_000;

    let mut possible_beacons_in_row = LineSegments::new(HorizontalSegment { x: ROW_X, len: ROW_LEN });

    for sensor in sensors {
        if let Some(segment) = sensor.no_beacon_segment(ROW_Y) {
            possible_beacons_in_row.remove(segment);
        }
    }

    println!("{}", ROW_LEN - possible_beacons_in_row.summed_len());

    Ok(())
}

fn part_2(sensors: Vec<Sensor>) -> io::Result<()> {
    const MIN_X: i64 = 0;
    const MAX_X: i64 = 4_000_001;

    const MIN_Y: i64 = 0;
    const MAX_Y: i64 = 4_000_001;

    for row_y in MIN_Y..=MAX_Y {
        let mut possible_beacons_in_row = LineSegments::new(HorizontalSegment { x: MIN_X, len: MAX_X - MIN_X });

        for sensor in &sensors {
            if let Some(segment) = sensor.no_distress_beacon_segment(row_y) {
                possible_beacons_in_row.remove(segment);
            }
        }

        if possible_beacons_in_row.summed_len() != 0 {
            let segments = possible_beacons_in_row.into_segments();
            let mut possible_scores = Vec::<i64>::new();

            for segment in segments {
                for x in segment.x..(segment.x + segment.len) {
                    possible_scores.push(x * 4_000_000 + row_y);
                }
            }
            println!("y = {}: possible scores: {:?}", row_y, possible_scores);
        }
    }

    Ok(())
}



pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let sensors = reader
        .lines()
        .map(|line| line?.parse())
        .collect::<io::Result<Vec<Sensor>>>()?;

    let func = match part {
        Part::Part1 => part_1,
        Part::Part2 => part_2,
    };

    func(sensors)
}
