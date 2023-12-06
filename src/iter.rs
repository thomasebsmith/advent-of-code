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

pub struct SplitBy<I: Iterator, P> {
    iter: I,
    split_predicate: P,
}

impl<I: Iterator, P: FnMut(&I::Item) -> bool> Iterator for SplitBy<I, P> {
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut result = Self::Item::new();
        let mut items_exist = false;
        while let Some(item) = self.iter.next() {
            items_exist = true;
            if (self.split_predicate)(&item) {
                break;
            }
            result.push(item);
        }

        if items_exist {
            Some(result)
        } else {
            None
        }
    }
}

pub fn split_by<I, P>(iter: I, split_predicate: P) -> SplitBy<I, P>
where
    I: Iterator,
    P: FnMut(&I::Item) -> bool,
{
    SplitBy {
        iter,
        split_predicate,
    }
}

pub fn join<I>(iterator: I, joiner: &str) -> String
where
    I: Iterator,
    I::Item: AsRef<str>,
{
    let mut result = String::new();
    let mut is_first = true;
    for item in iterator {
        if !is_first {
            result += joiner;
        }
        is_first = false;
        result += item.as_ref();
    }

    result
}
