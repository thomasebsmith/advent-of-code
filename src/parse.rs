use std::error::Error;
use std::io::{self, BufRead};
use std::str::FromStr;

use crate::errors::invalid_input;
use crate::iter::split_by;

pub fn lines_vec<R: io::Read>(
    reader: io::BufReader<R>,
) -> io::Result<Vec<String>> {
    reader.lines().collect::<io::Result<Vec<_>>>()
}

pub fn lines<R: io::Read>(
    reader: io::BufReader<R>,
) -> io::Result<impl Iterator<Item = String>> {
    lines_vec(reader).map(Vec::into_iter)
}

pub fn paragraphs<I>(iterator: I) -> impl Iterator<Item = Vec<I::Item>>
where
    I: Iterator,
    I::Item: AsRef<str>,
{
    split_by(iterator, |line| line.as_ref().is_empty())
}

pub fn parse_all<I, T>(iterator: I) -> io::Result<Vec<T>>
where
    I: Iterator,
    I::Item: AsRef<str>,
    T: FromStr,
    T::Err: Into<Box<dyn Error + Send + Sync>>,
{
    iterator
        .map(|string| string.as_ref().parse().map_err(invalid_input))
        .collect::<io::Result<Vec<_>>>()
}

pub fn parse_words<T>(line: &str) -> io::Result<Vec<T>>
where
    T: FromStr,
    T::Err: Into<Box<dyn Error + Send + Sync>>,
{
    parse_all(line.split_whitespace())
}
