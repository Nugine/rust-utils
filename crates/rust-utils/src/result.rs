use crate::sealed::Sealed;

pub trait ResultExt<T, E>: Sealed {
    fn inspect_err_(self, f: impl FnOnce(&E)) -> Self;
}

impl<T, E> Sealed for Result<T, E> {}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn inspect_err_(self, f: impl FnOnce(&E)) -> Self {
        self.map_err(|e| {
            f(&e);
            e
        })
    }
}
