use crate::sealed::Sealed;

use alloc::vec::Vec;
use core::ptr;

pub trait VecExt<T>: Sealed {
    fn insert_vec(&mut self, index: usize, other: Vec<T>);
}

impl<T> Sealed for Vec<T> {}

impl<T> VecExt<T> for Vec<T> {
    fn insert_vec(&mut self, index: usize, mut v: Vec<T>) {
        self.reserve_exact(v.len());

        unsafe {
            let len = self.len();
            let base = self.as_mut_ptr();
            let additional = v.len();

            // move existing elements
            ptr::copy(base.add(index), base.add(index + additional), len - index);

            // copy from `v`
            ptr::copy_nonoverlapping(v.as_ptr(), base.add(index), additional);

            // set length
            v.set_len(0);
            self.set_len(len + additional);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn o<T: ToOwned + ?Sized>(x: &T) -> T::Owned {
        x.to_owned()
    }

    #[test]
    fn test_insert_vec() {
        {
            let mut v = vec![1, 2, 3, 4, 5];
            v.insert_vec(2, vec![6, 7, 8]);
            assert_eq!(v, vec![1, 2, 6, 7, 8, 3, 4, 5]);
        }

        // owned strings
        {
            let mut v = vec![o("a"), o("b"), o("c"), o("d"), o("e")];
            v.insert_vec(2, vec![o("f"), o("g"), o("h")]);
            assert_eq!(v, vec![o("a"), o("b"), o("f"), o("g"), o("h"), o("c"), o("d"), o("e")]);
        }
    }
}
