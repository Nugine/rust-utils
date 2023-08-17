/// TODO: move to rust_utils
pub fn drain_filter<'a, T, F>(v: &'a mut Vec<T>, mut f: F) -> impl Iterator<Item = T> + 'a
where
    F: FnMut(&mut T) -> bool + 'a,
{
    let mut i = 0;
    std::iter::from_fn(move || {
        while i < v.len() {
            if f(&mut v[i]) {
                return Some(v.remove(i));
            } else {
                i += 1;
            }
        }
        None
    })
}
