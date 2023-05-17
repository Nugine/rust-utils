#[cfg(feature = "alloc")]
use alloc::vec::Vec;

pub fn map_collect<C, T, I, F>(iterable: I, f: F) -> C
where
    I: IntoIterator,
    F: FnMut(I::Item) -> T,
    C: FromIterator<T>,
{
    iterable.into_iter().map(f).collect()
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg(feature = "alloc")]
pub fn map_collect_vec<T, I, F>(iterable: I, f: F) -> Vec<T>
where
    I: IntoIterator,
    F: FnMut(I::Item) -> T,
{
    map_collect(iterable, f)
}

pub fn filter_map_collect<C, T, I, F>(iterable: I, f: F) -> C
where
    I: IntoIterator,
    F: FnMut(I::Item) -> Option<T>,
    C: FromIterator<T>,
{
    iterable.into_iter().filter_map(f).collect()
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg(feature = "alloc")]
pub fn filter_map_collect_vec<T, I, F>(iterable: I, f: F) -> Vec<T>
where
    I: IntoIterator,
    F: FnMut(I::Item) -> Option<T>,
{
    filter_map_collect(iterable, f)
}
