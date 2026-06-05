//! The owning, allocation-friendly query string builder.

use crate::QUERY;
use percent_encoding::utf8_percent_encode;
use std::fmt::{self, Display, Formatter, Write};

/// An owning query string builder for percent encoding key-value pairs.
///
/// This is the lifetime-free twin of [`QueryString`](crate::QueryString): it
/// eagerly converts keys and values to owned [`String`]s, so it can be stored
/// in structs, returned from functions and passed around freely.
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
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct QueryStringOwned {
    pairs: Vec<(String, String)>,
}

impl QueryStringOwned {
    /// Creates a new, empty query string builder.
    pub fn new() -> Self {
        Self { pairs: Vec::new() }
    }

    pub(crate) fn from_pairs(pairs: Vec<(String, String)>) -> Self {
        Self { pairs }
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
    pub fn with<K: ToString, V: ToString>(mut self, key: K, value: V) -> Self {
        self.pairs.push((key.to_string(), value.to_string()));
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
    pub fn with_opt<K: ToString, V: ToString>(self, key: K, value: Option<V>) -> Self {
        if let Some(value) = value {
            self.with(key, value)
        } else {
            self
        }
    }

    /// Appends a key-value pair to the query string.
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
    pub fn push<K: ToString, V: ToString>(&mut self, key: K, value: V) -> &mut Self {
        self.pairs.push((key.to_string(), value.to_string()));
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
    pub fn push_opt<K: ToString, V: ToString>(&mut self, key: K, value: Option<V>) -> &mut Self {
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
    pub fn append(&mut self, mut other: QueryStringOwned) {
        self.pairs.append(&mut other.pairs)
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
    pub fn append_into(mut self, mut other: QueryStringOwned) -> Self {
        self.pairs.append(&mut other.pairs);
        self
    }
}

impl Display for QueryStringOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.pairs.is_empty() {
            return Ok(());
        }
        f.write_char('?')?;
        for (i, (key, value)) in self.pairs.iter().enumerate() {
            if i > 0 {
                f.write_char('&')?;
            }
            Display::fmt(&utf8_percent_encode(key, QUERY), f)?;
            f.write_char('=')?;
            Display::fmt(&utf8_percent_encode(value, QUERY), f)?;
        }
        Ok(())
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
