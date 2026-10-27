//! Zero-allocation percent encoding for [`Display`] values.

use crate::QUERY;
use percent_encoding::utf8_percent_encode;
use std::fmt::{self, Display, Write};

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Formatter;

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
    fn test_encode_into_string() {
        let mut out = String::from("?k=");
        write_encoded(&mut out, &Chunked).unwrap();
        assert_eq!(out, "?k=Gr%C3%BCnkohl%20%26%20more");
    }
}
