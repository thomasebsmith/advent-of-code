#![feature(array_try_from_fn)]
#![feature(iter_array_chunks)]
#![feature(iterator_try_collect)]
#![feature(linked_list_cursors)]

mod errors;
mod iter;
mod part;
mod year2022;
mod year2023;

use std::env;
use std::error::Error;
use std::fs::File;
use std::io;
use std::process::ExitCode;

use crate::errors::invalid_input;

fn parse_args_and_run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 5 {
        let executable_name = if args.len() >= 1 {
            &args[0]
        } else {
            "cargo run --"
        };
        let message = format!(
            "Usage: {} <year> <day> <part> <input file>",
            executable_name,
        );
        Err(invalid_input(message))?
    }

    let year: u64 = args[1].parse()?;
    let day: u8 = args[2].parse()?;
    let part = match args[3].parse::<u8>()? {
        1 => part::Part::Part1,
        2 => part::Part::Part2,
        _ => Err(invalid_input("Invalid part"))?,
    };

    let run_function = match year {
        2022 => year2022::run,
        2023 => year2023::run,
        _ => Err(invalid_input("Invalid year"))?,
    };

    let file = File::open(&args[4])?;
    let reader = io::BufReader::new(file);

    Ok(run_function(day, part, reader)?)
}

fn main() -> ExitCode {
    match parse_args_and_run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{}", error);
            ExitCode::FAILURE
        }
    }
}
