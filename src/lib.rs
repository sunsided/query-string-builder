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
//!             .with_value("q", "🍎 apple")
//!             .with_opt_value("color", None::<String>)
//!             .with_opt_value("category", Some("fruits and vegetables?"));
//!
//! assert_eq!(
//!     format!("example.com/{qs}"),
//!     "example.com/?q=%F0%9F%8D%8E%20apple&category=fruits%20and%20vegetables?"
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
pub struct QueryString {
    pairs: Vec<Kvp>,
}

impl QueryString {
    /// Creates a new, empty query string builder.
    pub fn new() -> Self {
        Self {
            pairs: Vec::default(),
        }
    }

    /// Appends a key-value pair to the query string.
    ///
    /// ## Example
    ///
    /// ```
    /// use query_string_builder::QueryString;
    ///
    /// let qs = QueryString::new()
    ///             .with_value("q", "🍎 apple")
    ///             .with_value("category", "fruits and vegetables");
    ///
    /// assert_eq!(
    ///     format!("https://example.com/{qs}"),
    ///     "https://example.com/?q=%F0%9F%8D%8E%20apple&category=fruits%20and%20vegetables"
    /// );
    /// ```
    pub fn with_value<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.pairs.push(Kvp {
            key: key.into(),
            value: value.into(),
        });
        self
    }

    /// Appends a key-value pair to the query string if the value exists.
    ///
    /// ## Example
    ///
    /// ```
    /// use query_string_builder::QueryString;
    ///
    /// let qs = QueryString::new()
    ///             .with_opt_value("q", Some("🍎 apple"))
    ///             .with_opt_value("f", None::<String>)
    ///             .with_opt_value("category", Some("fruits and vegetables"));
    ///
    /// assert_eq!(
    ///     format!("https://example.com/{qs}"),
    ///     "https://example.com/?q=%F0%9F%8D%8E%20apple&category=fruits%20and%20vegetables"
    /// );
    /// ```
    pub fn with_opt_value<K: Into<String>, V: Into<String>>(
        self,
        key: K,
        value: Option<V>,
    ) -> Self {
        if let Some(value) = value {
            self.with_value(key, value)
        } else {
            self
        }
    }

    /// Appends a key-value pair to the query string.
    ///
    /// ## Example
    ///
    /// ```
    /// use query_string_builder::QueryString;
    ///
    /// let mut qs = QueryString::new();
    /// qs.push("q", "apple");
    /// qs.push("category", "fruits and vegetables");
    ///
    /// assert_eq!(
    ///     format!("https://example.com/{qs}"),
    ///     "https://example.com/?q=apple&category=fruits%20and%20vegetables"
    /// );
    /// ```
    pub fn push<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) -> &Self {
        self.pairs.push(Kvp {
            key: key.into(),
            value: value.into(),
        });
        self
    }

    /// Appends a key-value pair to the query string if the value exists.
    ///
    /// ## Example
    ///
    /// ```
    /// use query_string_builder::QueryString;
    ///
    /// let mut qs = QueryString::new();
    /// qs.push_opt("q", None::<String>);
    /// qs.push_opt("q", Some("🍎 apple"));
    ///
    /// assert_eq!(
    ///     format!("https://example.com/{qs}"),
    ///     "https://example.com/?q=%F0%9F%8D%8E%20apple"
    /// );
    /// ```
    pub fn push_opt<K: Into<String>, V: Into<String>>(
        &mut self,
        key: K,
        value: Option<V>,
    ) -> &Self {
        if let Some(value) = value {
            self.push(key, value)
        } else {
            self
        }
    }

    /// Determines the number of key-value pairs currently in the builder.
    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    /// Determines if the builder is currently empty.
    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    /// Appends another query string builder's values.
    ///
    /// ## Example
    ///
    /// ```
    /// use query_string_builder::QueryString;
    ///
    /// let mut qs = QueryString::new().with_value("q", "apple");
    /// let more = QueryString::new().with_value("q", "pear");
    ///
    /// qs.append(more);
    ///
    /// assert_eq!(
    ///     format!("https://example.com/{qs}"),
    ///     "https://example.com/?q=apple&q=pear"
    /// );
    /// ```
    pub fn append(&mut self, mut other: QueryString) {
        self.pairs.append(&mut other.pairs)
    }

    /// Appends another query string builder's values, consuming both types.
    ///
    /// ## Example
    ///
    /// ```
    /// use query_string_builder::QueryString;
    ///
    /// let qs = QueryString::new().with_value("q", "apple");
    /// let more = QueryString::new().with_value("q", "pear");
    ///
    /// let qs = qs.append_into(more);
    ///
    /// assert_eq!(
    ///     format!("https://example.com/{qs}"),
    ///     "https://example.com/?q=apple&q=pear"
    /// );
    /// ```
    pub fn append_into(mut self, mut other: QueryString) -> Self {
        self.pairs.append(&mut other.pairs);
        self
    }
}

impl Display for QueryString {
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
                    key = utf8_percent_encode(&pair.key, FRAGMENT),
                    value = utf8_percent_encode(&pair.value, FRAGMENT)
                )?;
            }
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
struct Kvp {
    key: String,
    value: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let qs = QueryString::new()
            .with_value("q", "apple???")
            .with_value("category", "fruits and vegetables");
        assert_eq!(
            qs.to_string(),
            "?q=apple???&category=fruits%20and%20vegetables"
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

    #[test]
    fn test_optional() {
        let qs = QueryString::new()
            .with_value("q", "celery")
            .with_opt_value("taste", None::<String>)
            .with_opt_value("category", Some("fruits and vegetables"));
        assert_eq!(
            qs.to_string(),
            "?q=celery&category=fruits%20and%20vegetables"
        );
    }
}
