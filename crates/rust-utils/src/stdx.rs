#[cfg(feature = "alloc")]
cfg_group! {
    use alloc::boxed::Box;
    use core::mem::MaybeUninit;
}

#[cfg(all(feature = "alloc", feature = "utf8"))]
use alloc::string::String;

#[cfg_attr(docsrs, doc(cfg(feature = "ascii")))]
#[cfg(feature = "ascii")]
pub fn str_from_ascii(bytes: &[u8]) -> Option<&str> {
    // TODO(blocking): use `unicode_simd::from_ascii`
    bytes.is_ascii().then(|| unsafe { core::str::from_utf8_unchecked(bytes) })
}

#[cfg_attr(docsrs, doc(cfg(feature = "utf8")))]
#[cfg(feature = "utf8")]
pub fn str_from_utf8(bytes: &[u8]) -> Option<&str> {
    simdutf8::basic::from_utf8(bytes).ok()
}

#[cfg_attr(docsrs, doc(cfg(all(feature = "utf8", feature = "alloc"))))]
#[cfg(all(feature = "utf8", feature = "alloc"))]
pub fn string_from_utf8(bytes: Vec<u8>) -> Option<String> {
    let is_utf8 = simdutf8::basic::from_utf8(&bytes).is_ok();
    is_utf8.then(|| unsafe { String::from_utf8_unchecked(bytes) })
}

/// [`Box::new_uninit`](Box::new_uninit)
///
/// See <https://github.com/rust-lang/rust/issues/63291>
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg(feature = "alloc")]
pub fn box_new_uninit<T>() -> Box<MaybeUninit<T>> {
    use alloc::alloc::{alloc, handle_alloc_error};
    use core::alloc::Layout;

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

/// [`Box::new_zeroed`](Box::new_zeroed)
///
/// See <https://github.com/rust-lang/rust/issues/63291>
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg(feature = "alloc")]
pub fn box_new_zeroed<T>() -> Box<MaybeUninit<T>> {
    use alloc::alloc::{alloc_zeroed, handle_alloc_error};
    use core::alloc::Layout;

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

/// [`Box::assume_init`](Box::assume_init)
///
/// See <https://github.com/rust-lang/rust/issues/63291>
///
/// # Safety
/// See [`Box::assume_init`](Box::assume_init)
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg(feature = "alloc")]
pub unsafe fn box_assume_init<T>(b: Box<MaybeUninit<T>>) -> Box<T> {
    let ptr = Box::into_raw(b).cast::<T>();
    Box::from_raw(ptr)
}
