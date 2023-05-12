use alloc::vec::Vec;

pub fn map_collect_vec<T, I, F>(iterable: I, f: F) -> Vec<T>
where
    I: IntoIterator,
    F: FnMut(I::Item) -> T,
{
    iterable.into_iter().map(f).collect()
}
