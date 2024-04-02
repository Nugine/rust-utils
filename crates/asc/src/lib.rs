#![deny(
    clippy::all,
    clippy::as_conversions,
    clippy::float_arithmetic,
    clippy::arithmetic_side_effects,
    clippy::must_use_candidate,
    clippy::missing_inline_in_public_items,
    clippy::missing_const_for_fn
)]
#![allow(
    clippy::missing_safety_doc, // TODO
)]
// ---
#![no_std]

extern crate alloc;

use core::fmt;
use core::marker::PhantomData;
use core::mem;
use core::mem::ManuallyDrop;
use core::ops::Deref;
use core::ptr;
use core::ptr::NonNull;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::*;

use alloc::boxed::Box;

pub struct Asc<T: ?Sized> {
    inner: NonNull<Inner<T>>,
    _marker: PhantomData<T>,
}

unsafe impl<T: Send + Sync> Send for Asc<T> {}
unsafe impl<T: Send + Sync> Sync for Asc<T> {}

#[repr(C)]
struct Inner<T: ?Sized> {
    strong: AtomicUsize,
    data: T,
}

unsafe fn box_from_nonnull<T: ?Sized>(p: NonNull<T>) -> Box<T> {
    Box::from_raw(p.as_ptr())
}

fn box_into_nonnull<T>(b: Box<T>) -> NonNull<T> {
    unsafe { NonNull::new_unchecked(Box::into_raw(b)) }
}

#[cfg(not(target_pointer_width = "64"))]
#[cold]
fn critical() -> ! {
    struct Bomb {}

    impl Drop for Bomb {
        fn drop(&mut self) {
            panic!("bomb")
        }
    }

    let _bomb = Bomb {};
    panic!("critical failure")
}

impl<T> Asc<T> {
    #[inline]
    #[must_use]
    pub fn new(data: T) -> Self {
        let inner = Box::new(Inner {
            strong: AtomicUsize::new(1),
            data,
        });
        Self {
            inner: box_into_nonnull(inner),
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn try_unwrap(this: Self) -> Result<T, Self> {
        let s = this.strong();
        if s.compare_exchange(1, 0, Relaxed, Relaxed).is_err() {
            return Err(this);
        }
        let _ = s.load(Acquire);
        unsafe {
            let this = ManuallyDrop::new(this);
            let data = ptr::read(&this.inner.as_ref().data);
            Ok(data)
        }
    }

    #[inline]
    #[must_use]
    #[allow(clippy::missing_const_for_fn, clippy::as_conversions)]
    pub unsafe fn from_raw(ptr: *const T) -> Self {
        let offset = mem::size_of::<AtomicUsize>();
        let inner = ptr.cast::<u8>().sub(offset) as *mut Inner<T>;
        Self {
            inner: NonNull::new_unchecked(inner),
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized> Asc<T> {
    #[allow(clippy::missing_const_for_fn)] // nightly
    fn strong(&self) -> &AtomicUsize {
        unsafe { &self.inner.as_ref().strong }
    }

    #[inline]
    #[must_use]
    pub fn shallow_clone(&self) -> Self {
        let s = self.strong();
        let old = s.fetch_add(1, Relaxed);

        #[cfg(not(target_pointer_width = "64"))]
        if old >= isize::MAX as usize {
            critical()
        }

        #[cfg(target_pointer_width = "64")]
        let _ = old;

        Self {
            inner: self.inner,
            _marker: PhantomData,
        }
    }

    #[inline(never)]
    unsafe fn destroy(&mut self) {
        drop(box_from_nonnull(self.inner))
    }

    #[inline]
    #[must_use]
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        ptr::eq(this.inner.as_ptr(), other.inner.as_ptr())
    }

    #[inline]
    #[must_use]
    pub unsafe fn get_mut_unchecked(this: &mut Self) -> &mut T {
        &mut this.inner.as_mut().data
    }

    #[allow(clippy::missing_const_for_fn)] // nightly
    #[inline]
    #[must_use]
    pub fn as_ptr(this: &Self) -> *const T {
        unsafe { ptr::addr_of!(this.inner.as_ref().data) }
    }

    #[inline]
    #[must_use]
    pub fn into_raw(this: Self) -> *const T {
        let this = ManuallyDrop::new(this);
        Self::as_ptr(&*this)
    }
}

impl<T: ?Sized> Deref for Asc<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &self.inner.as_ref().data }
    }
}

impl<T: ?Sized> Clone for Asc<T> {
    #[inline]
    fn clone(&self) -> Self {
        self.shallow_clone()
    }
}

impl<T: ?Sized> Drop for Asc<T> {
    #[inline]
    fn drop(&mut self) {
        let s = self.strong();
        if s.fetch_sub(1, Release) != 1 {
            return;
        }

        let _ = s.load(Acquire);
        unsafe { self.destroy() };
    }
}

impl<T: Clone> Asc<T> {
    #[inline]
    #[must_use]
    pub fn make_mut(this: &mut Self) -> &mut T {
        let s = this.strong();
        let count = s.load(Acquire);
        if count > 1 {
            *this = Asc::new(T::clone(&**this));
        }
        unsafe { &mut this.inner.as_mut().data }
    }
}

impl<T: fmt::Debug + ?Sized> fmt::Debug for Asc<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <T as fmt::Debug>::fmt(&**self, f)
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::*;

    use serde::{Deserialize, Serialize};

    impl<'de, T: Deserialize<'de>> Deserialize<'de> for Asc<T> {
        #[inline]
        fn deserialize<D>(deserializer: D) -> Result<Asc<T>, D::Error>
        where
            D: ::serde::de::Deserializer<'de>,
        {
            T::deserialize(deserializer).map(Asc::new)
        }
    }

    impl<T: Serialize> Serialize for Asc<T> {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ::serde::ser::Serializer,
        {
            T::serialize(&**self, serializer)
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    use alloc::string::String;

    #[test]
    fn simple() {
        let a = Asc::new(String::from("hello"));
        let a1 = a.shallow_clone();
        let a2 = Asc::clone(&a);

        assert_eq!(&*a, "hello");

        drop(a1);
        drop(a);
        drop(a2);
    }
}
