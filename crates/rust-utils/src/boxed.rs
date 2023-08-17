use crate::sealed::Sealed;

use core::alloc::Layout;
use core::mem::MaybeUninit;

use alloc::alloc::{alloc, alloc_zeroed, handle_alloc_error};
use alloc::boxed::Box;

pub trait BoxExt<T: ?Sized>: Sealed {
    /// [`Box::new_uninit`](Box::new_uninit)
    ///
    /// See <https://github.com/rust-lang/rust/issues/63291>
    fn new_uninit_() -> Box<MaybeUninit<T>>
    where
        T: Sized;

    /// [`Box::new_zeroed`](Box::new_zeroed)
    ///
    /// See <https://github.com/rust-lang/rust/issues/63291>
    fn new_zeroed_() -> Box<MaybeUninit<T>>
    where
        T: Sized;

    /// [`Box::assume_init`](Box::assume_init)
    ///
    /// See <https://github.com/rust-lang/rust/issues/63291>
    ///
    /// # Safety
    /// See [`Box::assume_init`](Box::assume_init)
    unsafe fn assume_init_(this: Box<MaybeUninit<T>>) -> Box<T>
    where
        T: Sized;
}

impl<T: ?Sized> Sealed for Box<T> {}

impl<T: ?Sized> BoxExt<T> for Box<T> {
    fn new_uninit_() -> Box<MaybeUninit<T>>
    where
        T: Sized,
    {
        // TODO: inline_const
        assert!(core::mem::size_of::<T>() != 0);

        let layout = Layout::new::<T>();
        unsafe {
            let ptr = alloc(layout);
            if ptr.is_null() {
                handle_alloc_error(layout)
            }
            Box::from_raw(ptr.cast())
        }
    }

    fn new_zeroed_() -> Box<MaybeUninit<T>>
    where
        T: Sized,
    {
        // TODO: inline_const
        assert!(core::mem::size_of::<T>() != 0);

        let layout = Layout::new::<T>();
        unsafe {
            let ptr = alloc_zeroed(layout);
            if ptr.is_null() {
                handle_alloc_error(layout)
            }
            Box::from_raw(ptr.cast())
        }
    }

    unsafe fn assume_init_(this: Box<MaybeUninit<T>>) -> Box<T>
    where
        T: Sized,
    {
        let ptr = Box::into_raw(this).cast::<T>();
        Box::from_raw(ptr)
    }
}
