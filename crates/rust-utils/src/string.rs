use crate::sealed::Sealed;

use alloc::string::String;
use alloc::vec::Vec;

pub trait StringExt {
    fn from_utf8_simd(bytes: Vec<u8>) -> Option<String>;
}

impl Sealed for String {}

impl StringExt for String {
    fn from_utf8_simd(bytes: Vec<u8>) -> Option<String> {
        let is_utf8 = simdutf8::basic::from_utf8(&bytes).is_ok();
        is_utf8.then(|| unsafe { String::from_utf8_unchecked(bytes) })
    }
}
