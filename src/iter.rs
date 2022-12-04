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
