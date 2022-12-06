use std::collections::{HashSet, VecDeque};
use std::hash::Hash;

pub fn n_elements<I: Iterator>(n: usize, iter: I) -> Option<Vec<I::Item>> {
    let result: Vec<_> = iter.collect();
    if result.len() != n {
        None
    } else {
        Some(result)
    }
}

pub fn only_element<I: Iterator>(mut iter: I) -> Option<I::Item> {
    let result = iter.next()?;
    match iter.next() {
        Some(_) => None,
        None => Some(result),
    }
}

pub struct ConsecutiveSequences<I: Iterator> {
    iter: I,
    sequence_size: usize,
    recent_elements: VecDeque<I::Item>,
}

impl<I: Iterator> Iterator for ConsecutiveSequences<I>
where
    I::Item: Copy,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.recent_elements.len() < self.sequence_size {
            match self.iter.next() {
                Some(element) => self.recent_elements.push_back(element),
                None => {
                    self.recent_elements.clear();
                    return None;
                }
            }
        }

        let mut result = Vec::<I::Item>::with_capacity(self.sequence_size);
        for element in self.recent_elements.iter() {
            result.push(*element);
        }
        self.recent_elements.pop_front();
        Some(result)
    }
}

pub fn consecutive_sequences<I: Iterator>(
    n: usize,
    iter: I,
) -> ConsecutiveSequences<I> {
    ConsecutiveSequences {
        iter,
        sequence_size: n,
        recent_elements: VecDeque::with_capacity(n),
    }
}

pub fn all_unique<I: Iterator>(mut iter: I) -> bool
where
    I::Item: Eq,
    I::Item: Hash,
{
    let mut items = HashSet::<I::Item>::new();
    loop {
        match iter.next() {
            Some(item) => {
                // insert returns false if the item was already present (not
                // unique)
                if !items.insert(item) {
                    return false;
                }
            }
            None => {
                return true;
            }
        }
    }
}
