#![feature(map_first_last)]

mod part;
mod year2022;

use std::env;
use std::error::Error;
use std::io;
use std::process::ExitCode;

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
        Err(io::Error::new(io::ErrorKind::InvalidInput, message))?
    }

    let year: u64 = args[1].parse()?;
    let day: u8 = args[2].parse()?;
    let part = match args[3].parse::<u8>()? {
        1 => part::Part::Part1,
        2 => part::Part::Part2,
        _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid part"))?,
    };

    let run_function = match year {
        2022 => year2022::run,
        _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid year"))?,
    };

    Ok(run_function(day, part, &args[4])?)
}

fn main() -> ExitCode {
    match parse_args_and_run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{}", error);
            ExitCode::FAILURE
        },
    }
}
