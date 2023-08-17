use crate::sealed::Sealed;

pub trait StrExt {
    fn from_ascii_simd(bytes: &[u8]) -> Option<&str>;

    fn from_utf8_simd(bytes: &[u8]) -> Option<&str>;
}

impl Sealed for str {}

impl StrExt for str {
    fn from_ascii_simd(bytes: &[u8]) -> Option<&str> {
        // TODO(blocking): use `unicode_simd::from_ascii`
        bytes.is_ascii().then(|| unsafe { core::str::from_utf8_unchecked(bytes) })
    }

    fn from_utf8_simd(bytes: &[u8]) -> Option<&str> {
        simdutf8::basic::from_utf8(bytes).ok()
    }
}
