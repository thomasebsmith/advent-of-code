use std::collections::LinkedList;
use std::collections::linked_list;
use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::part::Part;

// TODO: Use a custom type instead of LinkedList so this isn't so slow

fn find<T, F>(
    list: &mut LinkedList<T>,
    predicate: F,
) -> linked_list::CursorMut<'_, T>
where
    F: Fn(&T) -> bool,
{
    let mut cursor = list.cursor_front_mut();
    while let Some(value) = cursor.current() {
        if predicate(value) {
            break;
        }
        cursor.move_next();
    }
    return cursor;
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut numbers = reader
        .lines()
        .enumerate()
        .map(|(i, line)| Ok((i, line?.parse::<i64>().map_err(invalid_input)?)))
        .collect::<io::Result<LinkedList<_>>>()?;

    if part == Part::Part2 {
        for (_, val) in numbers.iter_mut() {
            *val *= 811_589_153;
        }
    }

    let list_length: i64 = numbers.len().try_into().unwrap();

    let num_repeats = match part {
        Part::Part1 => 1,
        Part::Part2 => 10,
    };

    for _ in 0..num_repeats {
        for i in 0..numbers.len() {
            // TODO: This is slow...
            let mut cursor = find(&mut numbers, |(j, _)| i == *j);

            let Some(&mut (_, mut value)) = cursor.current() else {
                panic!("Invalid state: could not find element");
            };

            // Always positive
            value = ((value % (list_length - 1)) + (list_length - 1))
                % (list_length - 1);

            if value == 0 {
                continue;
            }

            let removed_node = cursor.remove_current_as_list().unwrap();
            if cursor.index().is_none() {
                cursor.move_next();
            }
            assert!(value > 0);
            for _ in 0..(value - 1) {
                cursor.move_next();

                // Skip the ghost element
                if cursor.index().is_none() {
                    cursor.move_next();
                }
            }
            cursor.splice_after(removed_node);

            for _ in 0..(value - 1) {
                cursor.move_prev();

                if cursor.index().is_none() {
                    cursor.move_prev();
                }
            }
        }
    }

    // This could also be faster
    let mut cursor = find(&mut numbers, |&(_, value)| value == 0);

    if cursor.index().is_none() {
        Err(invalid_input("Could not find 0 in list"))?
    }

    for _ in 0..1000 {
        cursor.move_next();
        if cursor.index().is_none() {
            cursor.move_next();
        }
    }
    let el1000 = cursor.current().unwrap().1;

    for _ in 0..1000 {
        cursor.move_next();
        if cursor.index().is_none() {
            cursor.move_next();
        }
    }
    let el2000 = cursor.current().unwrap().1;

    for _ in 0..1000 {
        cursor.move_next();
        if cursor.index().is_none() {
            cursor.move_next();
        }
    }
    let el3000 = cursor.current().unwrap().1;

    println!("{}", el1000 + el2000 + el3000);

    Ok(())
}
