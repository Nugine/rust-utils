use crate::sealed::Sealed;

impl<T> Sealed for [T] {}

pub trait SliceExt<T>: Sealed {
    fn get2_mut(&mut self, i: usize, j: usize) -> Option<(&mut T, &mut T)>;
}

impl<T> SliceExt<T> for [T] {
    fn get2_mut(&mut self, i: usize, j: usize) -> Option<(&mut T, &mut T)> {
        if i == j || i >= self.len() || j >= self.len() {
            return None;
        }
        // SAFETY: `i` and `j` are in bounds and not equal
        unsafe {
            let ptr = self.as_mut_ptr();
            Some((&mut *ptr.add(i), &mut *ptr.add(j)))
        }
    }
}
