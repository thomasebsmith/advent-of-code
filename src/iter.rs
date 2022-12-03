pub fn only_element<I: Iterator>(mut iter: I) -> Option<I::Item> {
    let Some(el) = iter.next() else {
        return None;
    };

    if iter.count() != 0 {
        return None;
    }

    Some(el)
}
