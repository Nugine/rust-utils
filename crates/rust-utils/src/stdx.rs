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
