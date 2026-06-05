//! Zero-allocation percent encoding for [`Display`] values.

use crate::QUERY;
use percent_encoding::utf8_percent_encode;
use std::fmt::{self, Display, Formatter, Write};

/// A [`fmt::Write`] adapter that percent-encodes every chunk written through it.
///
/// Wrapping a [`Formatter`] in this type allows any [`Display`] value to be
/// rendered percent-encoded without allocating an intermediate [`String`]:
/// the standard formatting machinery only ever hands complete `&str` slices
/// to [`write_str`](Write::write_str), and percent encoding operates on
/// individual bytes, so encoding chunk by chunk is equivalent to encoding
/// the whole rendered string at once.
pub(crate) struct EncodeWriter<'f, 'a> {
    inner: &'f mut Formatter<'a>,
}

impl Write for EncodeWriter<'_, '_> {
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

/// Renders `value` into the formatter, percent-encoded, with zero
/// intermediate string allocation.
pub(crate) fn write_encoded(f: &mut Formatter<'_>, value: &dyn Display) -> fmt::Result {
    let mut writer = EncodeWriter { inner: f };
    write!(writer, "{value}")
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
