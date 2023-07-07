//! # A query string builder for percent encoding key-value pairs
//!
//! This is a tiny helper crate for simplifying the construction of URL query strings.
//! The initial `?` question mark is automatically prepended.
//!
//! ## Example
//!
//! ```
//! use query_string_builder::QueryString;
//!
//! let qs = QueryString::new()
//!             .with_value("q", "apple")
//!             .with_value("category", "fruits and vegetables");
//!
//! assert_eq!(
//!     format!("https://example.com/{qs}"),
//!     "https://example.com/?q=apple&category=fruits%20and%20vegetables"
//! );
//! ```

#![deny(unsafe_code)]

use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use std::fmt::{Debug, Display, Formatter};

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

/// A query string builder for percent encoding key-value pairs.
///
/// ## Example
///
/// ```
/// use query_string_builder::QueryString;
///
/// let qs = QueryString::new()
///             .with_value("q", "apple")
///             .with_value("category", "fruits and vegetables");
///
/// assert_eq!(
///     format!("https://example.com/{qs}"),
///     "https://example.com/?q=apple&category=fruits%20and%20vegetables"
/// );
/// ```
#[derive(Debug, Default, Clone)]
pub struct QueryString<'a> {
    pairs: Vec<Kvp<'a>>,
}

impl<'a> QueryString<'a> {
    /// Creates a new, empty query string builder.
    pub fn new() -> Self {
        Self {
            pairs: Vec::default(),
        }
    }

    /// Appends a key-value pair to the query string.
    pub fn with_value(mut self, key: &'a str, value: &'a str) -> Self {
        self.pairs.push(Kvp { key, value });
        self
    }

    /// Appends a key-value pair to the query string.
    pub fn push(&mut self, key: &'a str, value: &'a str) -> &Self {
        self.pairs.push(Kvp { key, value });
        self
    }

    /// Determines the number of key-value pairs currently in the builder.
    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    /// Determines if the builder is currently empty.
    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }
}

impl<'a> Display for QueryString<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.pairs.is_empty() {
            return Ok(());
        } else {
            write!(f, "?")?;
            for (i, pair) in self.pairs.iter().enumerate() {
                if i > 0 {
                    write!(f, "&")?;
                }
                write!(
                    f,
                    "{key}={value}",
                    key = utf8_percent_encode(pair.key, FRAGMENT),
                    value = utf8_percent_encode(pair.value, FRAGMENT)
                )?;
            }
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
struct Kvp<'a> {
    key: &'a str,
    value: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let qs = QueryString::new()
            .with_value("q", "apple")
            .with_value("category", "fruits and vegetables");
        assert_eq!(
            qs.to_string(),
            "?q=apple&category=fruits%20and%20vegetables"
        );
    }

    #[test]
    fn test_encoding() {
        let qs = QueryString::new()
            .with_value("q", "Grünkohl")
            .with_value("category", "Gemüse");
        assert_eq!(qs.to_string(), "?q=Gr%C3%BCnkohl&category=Gem%C3%BCse");
    }

    #[test]
    fn test_emoji() {
        let qs = QueryString::new()
            .with_value("q", "🥦")
            .with_value("🍽️", "🍔🍕");
        assert_eq!(
            qs.to_string(),
            "?q=%F0%9F%A5%A6&%F0%9F%8D%BD%EF%B8%8F=%F0%9F%8D%94%F0%9F%8D%95"
        );
    }
}
