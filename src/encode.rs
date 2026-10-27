//! Zero-allocation percent encoding for [`Display`] values.

use crate::QUERY;
use core::fmt::{self, Display, Write};
use percent_encoding::utf8_percent_encode;

/// A [`fmt::Write`] adapter that percent-encodes every chunk written through it.
///
/// Wrapping a [`Formatter`] or [`String`] in this type allows any [`Display`]
/// value to be rendered percent-encoded without allocating an intermediate
/// [`String`]:
/// the standard formatting machinery only ever hands complete `&str` slices
/// to [`write_str`](Write::write_str), and percent encoding operates on
/// individual bytes, so encoding chunk by chunk is equivalent to encoding
/// the whole rendered string at once.
pub(crate) struct EncodeWriter<'w, W: Write + ?Sized> {
    inner: &'w mut W,
}

impl<W: Write + ?Sized> Write for EncodeWriter<'_, W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // Iterate the encoded pieces directly instead of nesting `Display::fmt`
        // so that width/padding flags of the outer formatter cannot leak into
        // the individual chunks.
        for piece in utf8_percent_encode(s, QUERY) {
            self.inner.write_str(piece)?;
        }
        Ok(())
    }
}

/// Renders `value` into `out`, percent-encoded, with zero intermediate
/// string allocation.
pub(crate) fn write_encoded<W: Write + ?Sized>(out: &mut W, value: &dyn Display) -> fmt::Result {
    let mut writer = EncodeWriter { inner: out };
    write!(writer, "{value}")
}

/// A [`fmt::Write`] sink that only counts the bytes written to it.
struct ByteCounter(usize);

impl Write for ByteCounter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0 += s.len();
        Ok(())
    }
}

/// Returns the length of `s` once percent-encoded.
pub(crate) fn encoded_str_len(s: &str) -> usize {
    utf8_percent_encode(s, QUERY).map(str::len).sum()
}

/// Returns the length of `value` once rendered and percent-encoded, by
/// formatting it into a counter instead of a buffer.
///
/// Returns zero if the [`Display`] implementation fails.
pub(crate) fn encoded_len(value: &dyn Display) -> usize {
    let mut counter = ByteCounter(0);
    match write_encoded(&mut counter, value) {
        Ok(()) => counter.0,
        Err(fmt::Error) => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::fmt::Formatter;

    /// A `Display` implementation that writes in multiple small chunks,
    /// exercising the chunk-wise encoding path.
    struct Chunked;

    impl Display for Chunked {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            f.write_str("Grün")?;
            f.write_char('k')?;
            f.write_str("ohl & more")
        }
    }

    struct Wrapper(Chunked);

    impl Display for Wrapper {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write_encoded(f, &self.0)
        }
    }

    #[test]
    fn test_chunked_encoding() {
        assert_eq!(Wrapper(Chunked).to_string(), "Gr%C3%BCnkohl%20%26%20more");
    }

    #[test]
    fn test_encoded_len() {
        assert_eq!(encoded_len(&Chunked), "Gr%C3%BCnkohl%20%26%20more".len());
        assert_eq!(encoded_str_len("a b"), "a%20b".len());
        assert_eq!(encoded_len(&42), 2);
    }

    #[test]
    fn test_encode_into_string() {
        let mut out = String::from("?k=");
        write_encoded(&mut out, &Chunked).unwrap();
        assert_eq!(out, "?k=Gr%C3%BCnkohl%20%26%20more");
    }
}
