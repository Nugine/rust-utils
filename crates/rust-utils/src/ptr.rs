pub fn cast_ptr<T: ?Sized, U>(p: &T) -> *const U {
    <*const T>::cast(p)
}

pub fn cast_ptr_mut<T: ?Sized, U>(p: &mut T) -> *mut U {
    <*mut T>::cast(p)
}
