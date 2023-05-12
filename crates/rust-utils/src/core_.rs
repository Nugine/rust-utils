pub fn default<T: Default>() -> T {
    T::default()
}

pub fn default_with<T: Default>(f: impl FnOnce(&mut T)) -> T {
    let mut t = T::default();
    f(&mut t);
    t
}

pub fn map_collect<C, T, I, F>(iterable: I, f: F) -> C
where
    I: IntoIterator,
    F: FnMut(I::Item) -> T,
    C: FromIterator<T>,
{
    iterable.into_iter().map(f).collect()
}
