//! The borrowing, zero-allocation query string builder.

use crate::encode::write_encoded;
use crate::{QueryStringOwned, QUERY};
use percent_encoding::utf8_percent_encode;
use std::fmt::{self, Debug, Display, Formatter, Write};

/// A borrowed key or value of a query string pair.
///
/// You usually don't interact with this type directly; it is produced by the
/// [`IntoPart`] conversions accepted by [`QueryString`]'s methods.
#[derive(Clone, Copy)]
pub enum Part<'a> {
    /// A plain string slice.
    Str(&'a str),
    /// Any other borrowed [`Display`] value, rendered on demand.
    Display(&'a dyn Display),
}

impl Display for Part<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Part::Str(s) => Display::fmt(s, f),
            Part::Display(d) => Display::fmt(d, f),
        }
    }
}

impl Part<'_> {
    /// Writes this part into the formatter, percent-encoded, without
    /// allocating intermediate strings.
    fn write_encoded_to(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Part::Str(s) => {
                for piece in utf8_percent_encode(s, QUERY) {
                    f.write_str(piece)?;
                }
                Ok(())
            }
            Part::Display(d) => write_encoded(f, *d),
        }
    }
}

/// Conversion into a borrowed query string [`Part`].
///
/// Implemented for `&str` and for `&T` of any [`Display`] type, so string
/// literals, `&String`, `&i32`, `&bool` etc. can all be passed to
/// [`QueryString`]'s methods directly.
pub trait IntoPart<'a> {
    /// Performs the conversion.
    fn into_part(self) -> Part<'a>;
}

impl<'a> IntoPart<'a> for &'a str {
    fn into_part(self) -> Part<'a> {
        Part::Str(self)
    }
}

impl<'a, T: Display> IntoPart<'a> for &'a T {
    fn into_part(self) -> Part<'a> {
        Part::Display(self)
    }
}

/// A zero-allocation query string builder for percent encoding key-value pairs.
///
/// This builder borrows all keys and values. Building performs a single [`Vec`]
/// allocation for the pair list; rendering percent-encodes each value on the
/// fly without allocating intermediate strings.
///
/// If you need a builder without a lifetime parameter — e.g. to store it in a
/// struct or return it from a function that owns the values — use
/// [`QueryStringOwned`] or convert via [`into_owned`](Self::into_owned).
///
/// ## Example
///
/// ```
/// use query_string_builder::QueryString;
///
/// let tasty = true;
/// let qs = QueryString::new()
///     .with("q", "apple")
///     .with("tasty", &tasty)
///     .with_opt("category", Some("fruits and vegetables"));
///
/// assert_eq!(
///     format!("https://example.com/{qs}"),
///     "https://example.com/?q=apple&tasty=true&category=fruits%20and%20vegetables"
/// );
/// ```
///
/// ## Borrowing footgun
///
/// Because the builder borrows its values, temporaries created inline do not
/// live long enough — bind them to a variable first:
///
/// ```compile_fail
/// use query_string_builder::QueryString;
///
/// let qs = QueryString::new().with("answer", &42.to_string()); // temporary dropped here
/// println!("{qs}");
/// ```
///
/// ```
/// use query_string_builder::QueryString;
///
/// let answer = 42.to_string();
/// let qs = QueryString::new().with("answer", &answer);
/// assert_eq!(qs.to_string(), "?answer=42");
/// ```
#[derive(Clone, Default)]
pub struct QueryString<'a> {
    pairs: Vec<(Part<'a>, Part<'a>)>,
}

impl<'a> QueryString<'a> {
    /// Creates a new, empty query string builder.
    pub fn new() -> Self {
        Self { pairs: Vec::new() }
    }

    /// Appends a key-value pair to the query string.
    ///
    /// ## Example
    ///
    /// ```
    /// use query_string_builder::QueryString;
    ///
    /// let answer = 42;
    /// let qs = QueryString::new()
    ///     .with("q", "🍎 apple")
    ///     .with("category", "fruits and vegetables")
    ///     .with("answer", &answer);
    ///
    /// assert_eq!(
    ///     format!("https://example.com/{qs}"),
    ///     "https://example.com/?q=%F0%9F%8D%8E%20apple&category=fruits%20and%20vegetables&answer=42"
    /// );
    /// ```
    #[doc(alias = "with_value")]
    pub fn with<K, V>(mut self, key: K, value: V) -> Self
    where
        K: IntoPart<'a>,
        V: IntoPart<'a>,
    {
        self.pairs.push((key.into_part(), value.into_part()));
        self
    }

    /// Appends a key-value pair to the query string if the value exists.
    ///
    /// ## Example
    ///
    /// ```
    /// use query_string_builder::QueryString;
    ///
    /// let works = true;
    /// let qs = QueryString::new()
    ///     .with_opt("q", Some("🍎 apple"))
    ///     .with_opt("f", None::<&str>)
    ///     .with_opt("category", Some("fruits and vegetables"))
    ///     .with_opt("works", Some(&works));
    ///
    /// assert_eq!(
    ///     format!("https://example.com/{qs}"),
    ///     "https://example.com/?q=%F0%9F%8D%8E%20apple&category=fruits%20and%20vegetables&works=true"
    /// );
    /// ```
    #[doc(alias = "with_opt_value")]
    pub fn with_opt<K, V>(self, key: K, value: Option<V>) -> Self
    where
        K: IntoPart<'a>,
        V: IntoPart<'a>,
    {
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
    pub fn push<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: IntoPart<'a>,
        V: IntoPart<'a>,
    {
        self.pairs.push((key.into_part(), value.into_part()));
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
    /// qs.push_opt("q", None::<&str>);
    /// qs.push_opt("q", Some("🍎 apple"));
    ///
    /// assert_eq!(
    ///     format!("https://example.com/{qs}"),
    ///     "https://example.com/?q=%F0%9F%8D%8E%20apple"
    /// );
    /// ```
    pub fn push_opt<K, V>(&mut self, key: K, value: Option<V>) -> &mut Self
    where
        K: IntoPart<'a>,
        V: IntoPart<'a>,
    {
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
    /// let mut qs = QueryString::new().with("q", "apple");
    /// let more = QueryString::new().with("q", "pear");
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
    /// let qs = QueryString::new().with("q", "apple");
    /// let more = QueryString::new().with("q", "pear");
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

    /// Converts this borrowing builder into a [`QueryStringOwned`] by rendering
    /// each key and value to an owned [`String`].
    ///
    /// Useful for building cheaply with borrows and then storing or returning
    /// the result past the borrows' lifetimes.
    ///
    /// ## Example
    ///
    /// ```
    /// use query_string_builder::{QueryString, QueryStringOwned};
    ///
    /// let qs: QueryStringOwned = {
    ///     let q = String::from("apple");
    ///     QueryString::new().with("q", &q).into_owned()
    /// };
    ///
    /// assert_eq!(qs.to_string(), "?q=apple");
    /// ```
    pub fn into_owned(self) -> QueryStringOwned {
        QueryStringOwned::from_pairs(
            self.pairs
                .into_iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
        )
    }
}

impl Display for QueryString<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.pairs.is_empty() {
            return Ok(());
        }
        f.write_char('?')?;
        for (i, (key, value)) in self.pairs.iter().enumerate() {
            if i > 0 {
                f.write_char('&')?;
            }
            key.write_encoded_to(f)?;
            f.write_char('=')?;
            value.write_encoded_to(f)?;
        }
        Ok(())
    }
}

impl Debug for QueryString<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(
                self.pairs
                    .iter()
                    .map(|(key, value)| (key.to_string(), value.to_string())),
            )
            .finish()
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
        let tasty = true;
        let weight = 99.9;
        let qs = QueryString::new()
            .with("q", "apple???")
            .with("category", "fruits and vegetables")
            .with("tasty", &tasty)
            .with("weight", &weight);
        assert_eq!(
            qs.to_string(),
            "?q=apple???&category=fruits%20and%20vegetables&tasty=true&weight=99.9"
        );
        assert_eq!(qs.len(), 4);
        assert!(!qs.is_empty());
    }

    #[test]
    fn test_encoding() {
        let qs = QueryString::new()
            .with("q", "Grünkohl")
            .with("category", "Gemüse");
        assert_eq!(qs.to_string(), "?q=Gr%C3%BCnkohl&category=Gem%C3%BCse");
    }

    #[test]
    fn test_emoji() {
        let qs = QueryString::new().with("q", "🥦").with("🍽️", "🍔🍕");
        assert_eq!(
            qs.to_string(),
            "?q=%F0%9F%A5%A6&%F0%9F%8D%BD%EF%B8%8F=%F0%9F%8D%94%F0%9F%8D%95"
        );
    }

    #[test]
    fn test_optional() {
        let tasty = true;
        let weight = 99.9;
        let qs = QueryString::new()
            .with("q", "celery")
            .with_opt("taste", None::<&str>)
            .with_opt("category", Some("fruits and vegetables"))
            .with_opt("tasty", Some(&tasty))
            .with_opt("weight", Some(&weight));
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
        qs.push_opt("b", None::<&str>);
        qs.push_opt("c", Some("🍎 apple"));

        assert_eq!(
            format!("https://example.com/{qs}"),
            "https://example.com/?a=apple&c=%F0%9F%8D%8E%20apple"
        );
    }

    #[test]
    fn test_append() {
        let qs = QueryString::new().with("q", "apple");
        let more = QueryString::new().with("q", "pear");

        let mut qs = qs.append_into(more);
        qs.append(QueryString::new().with("answer", "42"));

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
            qs.push(*key, *value);
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

    #[test]
    fn test_non_string_refs() {
        let count = 12i32;
        let tasty = true;
        let weight = 99.9f64;
        let owned_string = String::from("kale");
        let qs = QueryString::new()
            .with("count", &count)
            .with("tasty", &tasty)
            .with("weight", &weight)
            .with("q", &owned_string);
        assert_eq!(qs.to_string(), "?count=12&tasty=true&weight=99.9&q=kale");
    }

    #[test]
    fn test_into_owned() {
        let owned = {
            let q = String::from("Grünkohl");
            QueryString::new().with("q", &q).into_owned()
        };
        assert_eq!(owned.to_string(), "?q=Gr%C3%BCnkohl");
        assert_eq!(owned.len(), 1);
    }

    #[test]
    fn test_debug() {
        let qs = QueryString::new().with("q", "apple");
        assert_eq!(format!("{qs:?}"), r#"{"q": "apple"}"#);
    }

    #[test]
    fn test_clone_default() {
        let qs = QueryString::default().with("q", "apple");
        let clone = qs.clone();
        assert_eq!(clone.to_string(), qs.to_string());
    }
}
