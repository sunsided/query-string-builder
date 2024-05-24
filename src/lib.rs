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

use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use percent_encoding::{percent_encode, utf8_percent_encode, AsciiSet, CONTROLS};

/// https://url.spec.whatwg.org/#query-percent-encode-set
const QUERY: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    // The following values are not strictly required by RFC 3986 but could help resolving recursion
    // where a URL is passed as a value. In these cases, occurrences of equal signs and ampersands
    // could break parsing.
    // By a similar logic, encoding the percent sign helps to resolve ambiguity.
    // The plus sign is also added to the set as to not confuse it with a space.
    .add(b'%')
    .add(b'&')
    .add(b'=')
    .add(b'+');

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
#[derive(Default, Clone)]
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
    ///             .with_value("category", "fruits and vegetables")
    ///             .with_value("answer", 42);
    ///
    /// assert_eq!(
    ///     format!("https://example.com/{qs}"),
    ///     "https://example.com/?q=%F0%9F%8D%8E%20apple&category=fruits%20and%20vegetables&answer=42"
    /// );
    /// ```
    pub fn with_value<K: ToString, V: ToString>(mut self, key: K, value: V) -> Self {
        self.pairs
            .push(Kvp::new(Key::from(key), Value::from(value)));
        self
    }

    pub fn with<K: Into<Key>, V: Into<Value>>(mut self, key: K, value: V) -> Self {
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
    pub fn with_opt_value<K: ToString, V: ToString>(self, key: K, value: Option<V>) -> Self {
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
    pub fn push<K: ToString, V: ToString>(&mut self, key: K, value: V) -> &Self {
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
    pub fn push_opt<K: ToString, V: ToString>(&mut self, key: K, value: Option<V>) -> &Self {
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
            Ok(())
        } else {
            write!(f, "?")?;
            for (i, pair) in self.pairs.iter().enumerate() {
                if i > 0 {
                    write!(f, "&")?;
                }

                write!(f, "{key}={value}", key = pair.key.0, value = pair.value.0)?;
            }
            Ok(())
        }
    }
}

#[derive(Clone)]
struct Kvp {
    key: Key,
    value: Value,
}

impl Kvp {
    pub fn new(key: Key, value: Value) -> Self {
        Self { key, value }
    }
}

#[derive(Clone)]
pub struct Key(Text);

#[derive(Clone)]
pub struct Value(Text);

impl Key {
    pub fn from<K: ToString>(key: K) -> Self {
        Self(Text::Unformatted(key.to_string()))
    }

    pub fn from_eager<K: ToString>(key: K) -> Self {
        Self(Text::Formatted(
            utf8_percent_encode(&key.to_string(), QUERY).to_string(),
        ))
    }

    pub fn from_str<K: Borrow<str>>(key: K) -> Self {
        Self(Text::Formatted(
            utf8_percent_encode(key.borrow(), QUERY).to_string(),
        ))
    }

    pub fn from_raw_bytes<K: AsRef<[u8]>>(key: K) -> Self {
        let key = key.as_ref();
        Self(Text::Formatted(percent_encode(key, QUERY).to_string()))
    }
}

impl Value {
    pub fn from<V: ToString>(value: V) -> Self {
        Self(Text::Unformatted(value.to_string()))
    }

    pub fn from_eager<V: ToString>(value: V) -> Self {
        Self(Text::Formatted(
            utf8_percent_encode(&value.to_string(), QUERY).to_string(),
        ))
    }

    pub fn from_str<V: Borrow<str>>(value: V) -> Self {
        Self(Text::Formatted(
            utf8_percent_encode(value.borrow(), QUERY).to_string(),
        ))
    }

    pub fn from_raw_bytes<V: AsRef<[u8]>>(value: V) -> Self {
        let key = value.as_ref();
        Self(Text::Formatted(percent_encode(key, QUERY).to_string()))
    }
}

impl FromStr for Key {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Key::from_str(s))
    }
}

impl FromStr for Value {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Value::from_str(s))
    }
}

impl<T> From<T> for Key
where
    T: ToString,
{
    fn from(value: T) -> Self {
        // TODO: If possible, specialize for &'static str
        Key::from(value)
    }
}

impl<T> From<T> for Value
where
    T: ToString,
{
    fn from(value: T) -> Self {
        // TODO: If possible, specialize for &'static str
        Value::from(value)
    }
}

#[derive(Clone)]
enum Text {
    /// The text is kept as it was provided originally. Percent encoding is done when rendering.
    Unformatted(String),
    /// The text is already percent encoded and can be used as-is when rendering.
    Formatted(String),
}

impl Display for Text {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Text::Unformatted(text) => write!(f, "{}", utf8_percent_encode(&text, QUERY)),
            Text::Formatted(text) => write!(f, "{}", text),
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
    fn test_with_simple() {
        let qs = QueryString::new()
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
    fn test_with_explicit() {
        let qs = QueryString::new()
            .with(Key::from_str("q"), Value::from_str("apple???"))
            .with(
                Key::from_eager("category"),
                Value::from_str("fruits and vegetables"),
            )
            .with(Key::from_raw_bytes("tasty".as_bytes()), Value::from(true))
            .with(Key::from("weight"), Value::from_eager(99.9));
        assert_eq!(
            qs.to_string(),
            "?q=apple???&category=fruits%20and%20vegetables&tasty=true&weight=99.9"
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

        let mut qs = QueryString::new();
        for (key, value, _) in &tests {
            qs.push(key.to_string(), value.to_string());
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
