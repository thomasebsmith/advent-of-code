use std::io::{self, BufRead};

use crate::iter::split_by;

pub fn lines<R: io::Read>(
    reader: io::BufReader<R>,
) -> io::Result<impl Iterator<Item = String>> {
    reader
        .lines()
        .collect::<io::Result<Vec<_>>>()
        .map(Vec::into_iter)
}

pub fn paragraphs<I: Iterator>(
    iterator: I,
) -> impl Iterator<Item = Vec<I::Item>>
where
    I::Item: AsRef<str>,
{
    split_by(iterator, |line| line.as_ref().is_empty())
}
