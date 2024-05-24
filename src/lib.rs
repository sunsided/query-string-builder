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
//!             .with_value("tasty", true)
//!             .with_opt_value("color", None::<String>)
//!             .with_opt_value("category", Some("fruits and vegetables?"));
//!
//! assert_eq!(
//!     format!("example.com/{qs}"),
//!     "example.com/?q=%F0%9F%8D%8E%20apple&tasty=true&category=fruits%20and%20vegetables?"
//! );
//! ```

#![deny(unsafe_code)]

use std::fmt::{Display, Formatter, Write};

use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
const QUERY: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

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
#[derive(Default)]
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
    ///
    /// ## Example
    ///
    /// ```
    /// use query_string_builder::QueryString;
    ///
    /// let qs = QueryString::new()
    ///             .with_value("q", "🍎 apple")
    ///             .with_value("category", "fruits and vegetables")
    ///             .with_value("answer", 42);
    ///
    /// assert_eq!(
    ///     format!("https://example.com/{qs}"),
    ///     "https://example.com/?q=%F0%9F%8D%8E%20apple&category=fruits%20and%20vegetables&answer=42"
    /// );
    /// ```
    pub fn with_value<K: ToString + 'static, V: ToString + 'static>(
        mut self,
        key: K,
        value: V,
    ) -> Self {
        self.pairs
            .push(Kvp::new(Key::from(key), Value::from(value)));
        self
    }

    /// TODO: Provide documentation
    pub fn with<K: Into<Key<'a>>, V: Into<Value<'a>>>(mut self, key: K, value: V) -> Self {
        self.pairs.push(Kvp::new(key.into(), value.into()));
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
    ///             .with_opt_value("category", Some("fruits and vegetables"))
    ///             .with_opt_value("works", Some(true));
    ///
    /// assert_eq!(
    ///     format!("https://example.com/{qs}"),
    ///     "https://example.com/?q=%F0%9F%8D%8E%20apple&category=fruits%20and%20vegetables&works=true"
    /// );
    /// ```
    pub fn with_opt_value<K: ToString + 'static, V: ToString + 'static>(
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
    pub fn push<K: ToString + 'static, V: ToString + 'static>(
        &mut self,
        key: K,
        value: V,
    ) -> &Self {
        self.pairs
            .push(Kvp::new(Key::from(key), Value::from(value)));
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
    pub fn push_opt<K: ToString + 'static, V: ToString + 'static>(
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
    pub fn append(&mut self, mut other: QueryString<'a>) {
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
    pub fn append_into(mut self, mut other: QueryString<'a>) -> Self {
        self.pairs.append(&mut other.pairs);
        self
    }
}

impl<'a> Display for QueryString<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.pairs.is_empty() {
            Ok(())
        } else {
            f.write_char('?')?;
            for (i, pair) in self.pairs.iter().enumerate() {
                if i > 0 {
                    f.write_char('&')?;
                }

                Display::fmt(&pair.key, f)?;
                f.write_char('=')?;
                Display::fmt(&pair.value, f)?;
            }
            Ok(())
        }
    }
}

pub struct Key<'a>(QueryPart<'a>);

pub struct Value<'a>(QueryPart<'a>);

impl<'a> Key<'a> {
    pub fn from<T: ToString + 'static>(value: T) -> Self {
        Self(QueryPart::Owned(Box::new(value)))
    }

    pub fn from_ref<T: ToString>(value: &'a T) -> Self {
        Self(QueryPart::Reference(value))
    }

    pub fn from_str(key: &'a str) -> Key<'a> {
        Self(QueryPart::RefStr(key))
    }
}

impl<'a> Value<'a> {
    pub fn from<T: ToString + 'static>(value: T) -> Self {
        Self(QueryPart::Owned(Box::new(value)))
    }

    pub fn from_ref<T: ToString>(value: &'a T) -> Self {
        Self(QueryPart::Reference(value))
    }

    pub fn from_str(value: &'a str) -> Self {
        Self(QueryPart::RefStr(&value))
    }
}

impl<'a> From<&'a str> for Key<'a> {
    fn from(value: &'a str) -> Self {
        Self::from_str(value)
    }
}

impl<'a> From<&'a str> for Value<'a> {
    fn from(value: &'a str) -> Self {
        Self::from_str(value)
    }
}

struct Kvp<'a> {
    key: QueryPart<'a>,
    value: QueryPart<'a>,
}

impl<'a> Kvp<'a> {
    pub fn new<K: Into<QueryPart<'a>>, V: Into<QueryPart<'a>>>(key: K, value: V) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

enum QueryPart<'a> {
    /// Captures a string reference.
    RefStr(&'a str),
    Owned(Box<dyn ToString>),
    Reference(&'a dyn ToString),
}

impl<'a> From<Key<'a>> for QueryPart<'a> {
    fn from(value: Key<'a>) -> Self {
        value.0
    }
}

impl<'a> From<Value<'a>> for QueryPart<'a> {
    fn from(value: Value<'a>) -> Self {
        value.0
    }
}

impl<'a> Display for QueryPart<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let encode = |x| utf8_percent_encode(x, QUERY);
        let mut write = |x| Display::fmt(&encode(x), f);
        match self {
            QueryPart::Owned(b) => write(&b.to_string()),
            QueryPart::Reference(b) => write(&b.to_string()),
            QueryPart::RefStr(s) => write(s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let qs = QueryString::new();
        assert_eq!(qs.to_string(), "");
        assert_eq!(qs.len(), 0);
        assert!(qs.is_empty());
    }

    #[test]
    fn test_simple() {
        let qs = QueryString::new()
            .with_value("q", "apple???")
            .with_value("category", "fruits and vegetables")
            .with_value("tasty", true)
            .with_value("weight", 99.9);
        assert_eq!(
            qs.to_string(),
            "?q=apple???&category=fruits%20and%20vegetables&tasty=true&weight=99.9"
        );
        assert_eq!(qs.len(), 4);
        assert!(!qs.is_empty());
    }

    #[test]
    fn test_deferred() {
        let query = String::from("apple???");

        struct Complex;
        impl Display for Complex {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "complex")
            }
        }

        let complex = Complex;
        let complex_ref = Complex;

        let qs = QueryString::new()
            .with("q", Value::from_str(&query))
            .with("owned", Value::from(complex))
            .with("borrowed", Value::from_ref(&complex_ref));
        assert_eq!(qs.to_string(), "?q=apple???&owned=complex&borrowed=complex");
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
            .with_opt_value("category", Some("fruits and vegetables"))
            .with_opt_value("tasty", Some(true))
            .with_opt_value("weight", Some(99.9));
        assert_eq!(
            qs.to_string(),
            "?q=celery&category=fruits%20and%20vegetables&tasty=true&weight=99.9"
        );
        assert_eq!(qs.len(), 4); // not five!
    }

    #[test]
    fn test_push_optional() {
        let mut qs = QueryString::new();
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
        let qs = QueryString::new().with_value("q", "apple");
        let more = QueryString::new().with_value("q", "pear");

        let mut qs = qs.append_into(more);
        qs.append(QueryString::new().with_value("answer", "42"));

        assert_eq!(
            format!("https://example.com/{qs}"),
            "https://example.com/?q=apple&q=pear&answer=42"
        );
    }
}
