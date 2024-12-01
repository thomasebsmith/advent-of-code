use std::io;
//use std::ops::{Add, RangeInclusive};

//use bigdecimal::{Signed, Zero};

//use crate::errors::invalid_input;
//use crate::parse::{lines, parse_all};
use crate::part::Part;

/*
//type VecN = f64;
//const ZERO: VecN = 0.0;
//type VecN = isize;
//const ZERO: VecN = 0;
type VecN = bigdecimal::BigDecimal;
const ZERO: VecN = VecN::zero();

#[derive(Clone, Debug)]
struct Vector {
    x: VecN,
    y: VecN,
    z: VecN,
}

impl Vector {
    fn from_string(string: &str) -> io::Result<Self> {
        let [x, y, z] = parse_all::<_, VecN>(string.split(",").map(str::trim))?[..] else {
            return Err(invalid_input("Expected <x>, <y>, <z>"));
        };
        Ok(Self { x, y, z })
    }
}

impl Add for Vector {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

struct TwoDBox {
    x: RangeInclusive<VecN>,
    y: RangeInclusive<VecN>,
}

impl TwoDBox {
    fn contains(&self, vector: Vector) -> bool {
        self.x.contains(&vector.x) && self.y.contains(&vector.y)
    }

    fn contains_intersection(&self, intersection: Intersection) -> bool {
        match intersection {
            Intersection::Point(vector) => self.contains(vector),
            Intersection::Line(pos, vel) => {
                panic!("Same line!");
            }
        }
    }
}

#[derive(Debug)]
struct Hailstone {
    initial_position: Vector,
    velocity: Vector,
}

enum Intersection {
    Point(Vector),
    Line(Vector, Vector),
}

impl Hailstone {
    fn from_line(line: impl AsRef<str>) -> io::Result<Self> {
        let [position_str, velocity_str] = &line.as_ref().split(" @ ").collect::<Vec<_>>()[..] else {
            return Err(invalid_input("Expected <position> @ <velocity>"));
        };
        Ok(Self {
            initial_position: Vector::from_string(position_str)?,
            velocity: Vector::from_string(velocity_str)?,
        })
    }

    //fn simple_velocity(&self) -> Vector {
        // TODO rename
        //let the_gcd = gcd(self.x, other.x
        //self.velocity 
    //}

    /*fn two_d_path_in_box(&self, the_box: &TwoDBox) -> Path {
        let start = self.initial_position.flatten_two_d();
    }*/
    fn two_d_intersection(&self, other: &Hailstone) -> Vec<Intersection> {
        // p + xv = p' + x'v'
        // y = yi + yv * (x - xi) / xv
        // y * xv = yi * xv + yv * x - yv * xi
        // (xv) * y + (-yv) * x + (yv * xi - yi * xv) = 0
        //if denominator == 0 {
            //None
        //} else {
            //Some(Vector {
                //x: 
        //}
        let p_this = self.initial_position + self.velocity;
        let p_other = other.initial_position + other.velocity;

        let denominator = (self.initial_position.x - p_this.x) * (other.initial_position.y - p_other.y) - (self.initial_position.y - p_this.y) * (other.initial_position.x - p_other.x);
        if denominator == ZERO {
            // Lines have the same slope
            // The question is, are they the same line?
            let x_diff = self.initial_position.x - other.initial_position.x;
            let y_diff = self.initial_position.y - other.initial_position.y;
            if x_diff * self.velocity.y == y_diff * self.velocity.x {
                // They are the same line. But are they going in opposite directions?
                let this_headed_toward_other = /* x_diff.signum() == 0 || */ self.velocity.x.signum() == -x_diff.signum();
                let other_headed_toward_this = /* x_diff.signum() == 0 || */ other.velocity.x.signum() == x_diff.signum();
                let mut intersections = Vec::<Intersection>::new();
                if this_headed_toward_other {
                    intersections.push(Intersection::Line(other.initial_position, other.velocity));
                }
                if other_headed_toward_this {
                    intersections.push(Intersection::Line(self.initial_position, self.velocity));
                }
                intersections
            } else {
                vec![]
            }
        } else {
            // TODO: integer or float math is questionable here
            let m1 = self.initial_position.x * p_this.y - self.initial_position.y * p_this.x;
            let m2 = other.initial_position.x * p_other.y - other.initial_position.y * p_other.x;

            let x = (m1 * (other.initial_position.x - p_other.x) - m2 * (self.initial_position.x - p_this.x)) / denominator;
            let y = (m1 * (other.initial_position.y - p_other.y) - m2 * (self.initial_position.y - p_this.y)) / denominator;

            let this_x_signum = (x - self.initial_position.x).signum();
            let this_y_signum = (y - self.initial_position.y).signum();
            let other_x_signum = (x - other.initial_position.x).signum();
            let other_y_signum = (y - other.initial_position.y).signum();

            let forward_this_x = /* this_x_signum == 0 || */ this_x_signum == self.velocity.x.signum();
            let forward_this_y = /* this_y_signum == 0 || */ this_y_signum == self.velocity.y.signum();
            if forward_this_x != forward_this_y {
                println!("this={:?}, other={:?}", &self, &other);
                println!("(x, y) = ({x}, {y})");
                println!("{} vs {}", (x - self.initial_position.x), self.velocity.x);
                println!("{} vs {}", (y - self.initial_position.y), self.velocity.y);
                panic!("fail");
            }
            let forward_other_x = /* other_x_signum == 0 || */ other_x_signum == other.velocity.x.signum();
            let forward_other_y = /* other_y_signum == 0 ||*/ other_y_signum == other.velocity.y.signum();
            if forward_other_x != forward_other_y {
                println!("this={:?}, other={:?}", &self, &other);
                println!("(x, y) = ({x}, {y})");
                println!("{} vs {}", (x - other.initial_position.x), other.velocity.x);
                println!("{} vs {}", (y - other.initial_position.y), other.velocity.y);
                panic!("fail");
            }
            assert!(forward_other_x == forward_other_y);

            if forward_this_x && forward_this_y && forward_other_x && forward_other_y {
                //println!("Intersection at ({x}, {y})");
                vec![Intersection::Point(Vector {
                    x,
                    y,
                    z: ZERO // TODO
                })]
            } else {
                //println!("not all forward @ ({x}, {y}): {forward_this_x} {forward_this_y} {forward_other_x} {forward_other_y}", );
                vec![]
            }
        }
    }
}*/

pub fn run<R: io::Read>(
    _part: Part,
    _reader: io::BufReader<R>,
) -> io::Result<()> {
/*
    //let test_area = TwoDBox { x: 200000000000000.0..=400000000000000.0, y: 200000000000000.0..=400000000000000.0 };
    let test_area = TwoDBox { x: 200000000000000.into()..=400000000000000.into(), y: 200000000000000.into()..=400000000000000.into() };
    //let test_area = TwoDBox { x: 7.0..=27.0, y: 7.0..=27.0 };
    let hailstones = lines(reader)?.map(Hailstone::from_line).collect::<io::Result<Vec<_>>>()?;

    let mut result = 0usize;

    for i in 0..hailstones.len() {
        for j in 0..i {
            let h1 = &hailstones[i];
            let h2 = &hailstones[j];
            for intersection in h1.two_d_intersection(h2) {
                if test_area.contains_intersection(intersection) {
                    result += 1;
                    break;
                } else{
                    //println!("not in test area: {:?}", intersection);
                }
            }
        }
    }

    println!("{result}");*/

    Ok(())
}
