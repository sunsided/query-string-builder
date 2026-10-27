//! The owning query string builder, backed by a single encoded buffer.

use crate::encode::write_encoded;
use alloc::string::String;
use core::fmt::{self, Display, Formatter, Write};

/// An owning query string builder for percent encoding key-value pairs.
///
/// This is the lifetime-free twin of [`QueryString`](crate::QueryString): it
/// can be stored in structs, returned from functions and passed around freely.
///
/// Keys and values are percent-encoded eagerly, straight into a single
/// internal [`String`] buffer — no per-pair strings are allocated. Use
/// [`with_capacity`](Self::with_capacity) to build without any reallocation,
/// and [`as_str`](Self::as_str) or [`into_string`](Self::into_string) to get
/// the rendered query string without copying it.
///
/// ## Example
///
/// ```
/// use query_string_builder::QueryStringOwned;
///
/// let qs = QueryStringOwned::new()
///     .with("q", "apple")
///     .with("tasty", true)
///     .with_opt("category", Some("fruits and vegetables"));
///
/// assert_eq!(
///     format!("https://example.com/{qs}"),
///     "https://example.com/?q=apple&tasty=true&category=fruits%20and%20vegetables"
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct QueryStringOwned {
    /// The rendered query string, including the leading `?` unless empty.
    buf: String,
    /// The number of key-value pairs encoded into `buf`.
    len: usize,
}

impl QueryStringOwned {
    /// Creates a new, empty query string builder.
    ///
    /// This does not allocate.
    pub const fn new() -> Self {
        Self {
            buf: String::new(),
            len: 0,
        }
    }

    /// Creates a new, empty query string builder whose buffer can hold at
    /// least `capacity` bytes of rendered output without reallocating.
    ///
    /// The capacity refers to the *encoded* output, including the leading `?`
    /// and the `=` and `&` separators.
    ///
    /// ## Example
    ///
    /// ```
    /// use query_string_builder::QueryStringOwned;
    ///
    /// let qs = QueryStringOwned::with_capacity(32).with("q", "apple");
    /// assert!(qs.capacity() >= 32);
    /// assert_eq!(qs.as_str(), "?q=apple");
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: String::with_capacity(capacity),
            len: 0,
        }
    }

    /// Wraps an already rendered query string and its pair count.
    pub(crate) fn from_rendered(buf: String, len: usize) -> Self {
        Self { buf, len }
    }

    /// Appends a key-value pair to the query string.
    ///
    /// ## Example
    ///
    /// ```
    /// use query_string_builder::QueryStringOwned;
    ///
    /// let qs = QueryStringOwned::new()
    ///     .with("q", "🍎 apple")
    ///     .with("category", "fruits and vegetables")
    ///     .with("answer", 42);
    ///
    /// assert_eq!(
    ///     format!("https://example.com/{qs}"),
    ///     "https://example.com/?q=%F0%9F%8D%8E%20apple&category=fruits%20and%20vegetables&answer=42"
    /// );
    /// ```
    #[doc(alias = "with_value")]
    #[must_use]
    pub fn with<K: Display, V: Display>(mut self, key: K, value: V) -> Self {
        self.push(key, value);
        self
    }

    /// Appends a key-value pair to the query string if the value exists.
    ///
    /// ## Example
    ///
    /// ```
    /// use query_string_builder::QueryStringOwned;
    ///
    /// let qs = QueryStringOwned::new()
    ///     .with_opt("q", Some("🍎 apple"))
    ///     .with_opt("f", None::<String>)
    ///     .with_opt("category", Some("fruits and vegetables"))
    ///     .with_opt("works", Some(true));
    ///
    /// assert_eq!(
    ///     format!("https://example.com/{qs}"),
    ///     "https://example.com/?q=%F0%9F%8D%8E%20apple&category=fruits%20and%20vegetables&works=true"
    /// );
    /// ```
    #[doc(alias = "with_opt_value")]
    #[must_use]
    pub fn with_opt<K: Display, V: Display>(self, key: K, value: Option<V>) -> Self {
        if let Some(value) = value {
            self.with(key, value)
        } else {
            self
        }
    }

    /// Appends a key-value pair to the query string.
    ///
    /// ## Panics
    ///
    /// Panics if a [`Display`] implementation of `key` or `value` returns an
    /// error, just like [`ToString::to_string`](alloc::string::ToString::to_string) does.
    ///
    /// ## Example
    ///
    /// ```
    /// use query_string_builder::QueryStringOwned;
    ///
    /// let mut qs = QueryStringOwned::new();
    /// qs.push("q", "apple");
    /// qs.push("category", "fruits and vegetables");
    ///
    /// assert_eq!(
    ///     format!("https://example.com/{qs}"),
    ///     "https://example.com/?q=apple&category=fruits%20and%20vegetables"
    /// );
    /// ```
    pub fn push<K: Display, V: Display>(&mut self, key: K, value: V) -> &mut Self {
        self.buf.push(if self.len == 0 { '?' } else { '&' });
        write_encoded(&mut self.buf, &key)
            .and_then(|()| self.buf.write_char('='))
            .and_then(|()| write_encoded(&mut self.buf, &value))
            .expect("a Display implementation returned an error unexpectedly");
        self.len += 1;
        self
    }

    /// Appends a key-value pair to the query string if the value exists.
    ///
    /// ## Example
    ///
    /// ```
    /// use query_string_builder::QueryStringOwned;
    ///
    /// let mut qs = QueryStringOwned::new();
    /// qs.push_opt("q", None::<String>);
    /// qs.push_opt("q", Some("🍎 apple"));
    ///
    /// assert_eq!(
    ///     format!("https://example.com/{qs}"),
    ///     "https://example.com/?q=%F0%9F%8D%8E%20apple"
    /// );
    /// ```
    pub fn push_opt<K: Display, V: Display>(&mut self, key: K, value: Option<V>) -> &mut Self {
        if let Some(value) = value {
            self.push(key, value)
        } else {
            self
        }
    }

    /// Determines the number of key-value pairs currently in the builder.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Determines if the builder is currently empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the number of bytes the internal buffer can hold without
    /// reallocating.
    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    /// Returns the rendered query string, including the leading `?`.
    ///
    /// An empty builder renders as the empty string. This is identical to
    /// the [`Display`] output but borrows the internal buffer instead of
    /// formatting.
    ///
    /// ## Example
    ///
    /// ```
    /// use query_string_builder::QueryStringOwned;
    ///
    /// let qs = QueryStringOwned::new().with("q", "🍎");
    /// assert_eq!(qs.as_str(), "?q=%F0%9F%8D%8E");
    /// assert_eq!(QueryStringOwned::new().as_str(), "");
    /// ```
    pub fn as_str(&self) -> &str {
        &self.buf
    }

    /// Consumes the builder and returns the rendered query string, including
    /// the leading `?`, without copying it.
    ///
    /// ## Example
    ///
    /// ```
    /// use query_string_builder::QueryStringOwned;
    ///
    /// let url = QueryStringOwned::new().with("q", "apple").into_string();
    /// assert_eq!(url, "?q=apple");
    /// ```
    pub fn into_string(self) -> String {
        self.buf
    }

    /// Appends another query string builder's values.
    ///
    /// ## Example
    ///
    /// ```
    /// use query_string_builder::QueryStringOwned;
    ///
    /// let mut qs = QueryStringOwned::new().with("q", "apple");
    /// let more = QueryStringOwned::new().with("q", "pear");
    ///
    /// qs.append(more);
    ///
    /// assert_eq!(
    ///     format!("https://example.com/{qs}"),
    ///     "https://example.com/?q=apple&q=pear"
    /// );
    /// ```
    pub fn append(&mut self, other: QueryStringOwned) {
        if other.is_empty() {
            return;
        }
        if self.is_empty() {
            // Reuse the other buffer instead of copying into an empty one,
            // unless ours was pre-sized to hold the result.
            if self.buf.capacity() < other.buf.len() {
                *self = other;
                return;
            }
            self.buf.push_str(&other.buf);
        } else {
            self.buf.push('&');
            self.buf.push_str(&other.buf[1..]);
        }
        self.len += other.len;
    }

    /// Appends another query string builder's values, consuming both types.
    ///
    /// ## Example
    ///
    /// ```
    /// use query_string_builder::QueryStringOwned;
    ///
    /// let qs = QueryStringOwned::new().with("q", "apple");
    /// let more = QueryStringOwned::new().with("q", "pear");
    ///
    /// let qs = qs.append_into(more);
    ///
    /// assert_eq!(
    ///     format!("https://example.com/{qs}"),
    ///     "https://example.com/?q=apple&q=pear"
    /// );
    /// ```
    #[must_use]
    pub fn append_into(mut self, other: QueryStringOwned) -> Self {
        self.append(other);
        self
    }
}

impl Display for QueryStringOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.buf)
    }
}

impl AsRef<str> for QueryStringOwned {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<QueryStringOwned> for String {
    fn from(qs: QueryStringOwned) -> Self {
        qs.into_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let qs = QueryStringOwned::new();
        assert_eq!(qs.to_string(), "");
        assert_eq!(qs.len(), 0);
        assert!(qs.is_empty());
    }

    #[test]
    fn test_simple() {
        let qs = QueryStringOwned::new()
            .with("q", "apple???")
            .with("category", "fruits and vegetables")
            .with("tasty", true)
            .with("weight", 99.9);
        assert_eq!(
            qs.to_string(),
            "?q=apple???&category=fruits%20and%20vegetables&tasty=true&weight=99.9"
        );
        assert_eq!(qs.len(), 4);
        assert!(!qs.is_empty());
    }

    #[test]
    fn test_encoding() {
        let qs = QueryStringOwned::new()
            .with("q", "Grünkohl")
            .with("category", "Gemüse");
        assert_eq!(qs.to_string(), "?q=Gr%C3%BCnkohl&category=Gem%C3%BCse");
    }

    #[test]
    fn test_emoji() {
        let qs = QueryStringOwned::new().with("q", "🥦").with("🍽️", "🍔🍕");
        assert_eq!(
            qs.to_string(),
            "?q=%F0%9F%A5%A6&%F0%9F%8D%BD%EF%B8%8F=%F0%9F%8D%94%F0%9F%8D%95"
        );
    }

    #[test]
    fn test_optional() {
        let qs = QueryStringOwned::new()
            .with("q", "celery")
            .with_opt("taste", None::<String>)
            .with_opt("category", Some("fruits and vegetables"))
            .with_opt("tasty", Some(true))
            .with_opt("weight", Some(99.9));
        assert_eq!(
            qs.to_string(),
            "?q=celery&category=fruits%20and%20vegetables&tasty=true&weight=99.9"
        );
        assert_eq!(qs.len(), 4); // not five!
    }

    #[test]
    fn test_push_optional() {
        let mut qs = QueryStringOwned::new();
        qs.push("a", "apple");
        qs.push_opt("b", None::<String>);
        qs.push_opt("c", Some("🍎 apple"));

        assert_eq!(
            format!("https://example.com/{qs}"),
            "https://example.com/?a=apple&c=%F0%9F%8D%8E%20apple"
        );
    }

    #[test]
    fn test_append() {
        let qs = QueryStringOwned::new().with("q", "apple");
        let more = QueryStringOwned::new().with("q", "pear");

        let mut qs = qs.append_into(more);
        qs.append(QueryStringOwned::new().with("answer", "42"));

        assert_eq!(
            format!("https://example.com/{qs}"),
            "https://example.com/?q=apple&q=pear&answer=42"
        );
    }

    #[test]
    fn test_append_empty_sides() {
        let mut qs = QueryStringOwned::new();
        qs.append(QueryStringOwned::new());
        assert_eq!(qs.as_str(), "");
        assert!(qs.is_empty());

        qs.append(QueryStringOwned::new().with("a", 1));
        qs.append(QueryStringOwned::new());
        assert_eq!(qs.as_str(), "?a=1");
        assert_eq!(qs.len(), 1);

        let mut sized = QueryStringOwned::with_capacity(64);
        sized.append(QueryStringOwned::new().with("b", 2).with("c", 3));
        assert_eq!(sized.as_str(), "?b=2&c=3");
        assert_eq!(sized.len(), 2);
        assert!(sized.capacity() >= 64);
    }

    #[test]
    fn test_as_str_matches_display() {
        let qs = QueryStringOwned::new().with("q", "a b").with("n", 1);
        assert_eq!(qs.as_str(), qs.to_string());
        assert_eq!(AsRef::<str>::as_ref(&qs), qs.as_str());
        assert_eq!(String::from(qs), "?q=a%20b&n=1");
    }

    #[test]
    fn test_with_capacity_does_not_reallocate() {
        let qs = QueryStringOwned::with_capacity(64);
        let ptr = qs.as_str().as_ptr();
        let qs = qs.with("q", "apple").with("n", 42);
        assert_eq!(qs.as_str().as_ptr(), ptr);
    }

    #[test]
    fn test_characters() {
        let tests = vec![
            ("space", " ", "%20"),
            ("double_quote", "\"", "%22"),
            ("hash", "#", "%23"),
            ("less_than", "<", "%3C"),
            ("equals", "=", "%3D"),
            ("greater_than", ">", "%3E"),
            ("percent", "%", "%25"),
            ("ampersand", "&", "%26"),
            ("plus", "+", "%2B"),
            //
            ("dollar", "$", "$"),
            ("single_quote", "'", "'"),
            ("comma", ",", ","),
            ("forward_slash", "/", "/"),
            ("colon", ":", ":"),
            ("semicolon", ";", ";"),
            ("question_mark", "?", "?"),
            ("at", "@", "@"),
            ("left_bracket", "[", "["),
            ("backslash", "\\", "\\"),
            ("right_bracket", "]", "]"),
            ("caret", "^", "^"),
            ("underscore", "_", "_"),
            ("grave", "^", "^"),
            ("left_curly", "{", "{"),
            ("pipe", "|", "|"),
            ("right_curly", "}", "}"),
        ];

        let mut qs = QueryStringOwned::new();
        for (key, value, _) in &tests {
            qs.push(key, value);
        }

        let mut expected = String::new();
        for (i, (key, _, value)) in tests.iter().enumerate() {
            if i > 0 {
                expected.push('&');
            }
            expected.push_str(&format!("{key}={value}"));
        }

        assert_eq!(
            format!("https://example.com/{qs}"),
            format!("https://example.com/?{expected}")
        );
    }
}
