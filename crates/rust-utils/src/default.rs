pub fn default<T: Default>() -> T {
    T::default()
}

pub fn default_with<T: Default>(f: impl FnOnce(&mut T)) -> T {
    let mut t = T::default();
    f(&mut t);
    t
}
