use alloc::vec::Vec;

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub fn map_collect_vec<T, I, F>(iterable: I, f: F) -> Vec<T>
where
    I: IntoIterator,
    F: FnMut(I::Item) -> T,
{
    iterable.into_iter().map(f).collect()
}
