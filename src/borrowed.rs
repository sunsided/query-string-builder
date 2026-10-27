//! The borrowing, zero-allocation query string builder.

use crate::encode::write_encoded;
use crate::storage::{CapacityError, InlinePairs, Pair, PairStorage};
use crate::{QUERY, QueryStringOwned};
use percent_encoding::utf8_percent_encode;
use std::fmt::{self, Debug, Display, Formatter, Write};
use std::marker::PhantomData;

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

    /// Returns the exact encoded length for string slices, and zero for
    /// [`Display`] values whose length is unknown until rendered.
    fn encoded_len_hint(&self) -> usize {
        match self {
            Part::Str(s) => utf8_percent_encode(s, QUERY).map(str::len).sum(),
            Part::Display(_) => 0,
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
/// This builder borrows all keys and values. Rendering percent-encodes each
/// value on the fly without allocating intermediate strings.
///
/// The pairs are kept in a [`PairStorage`], by default a [`Vec`] — the only
/// allocation the builder makes. Pre-size it with
/// [`with_capacity`](Self::with_capacity), or avoid the heap entirely with
/// fixed-capacity [`inline`](Self::inline) storage or any custom storage via
/// [`with_storage`](Self::with_storage).
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
#[derive(Clone)]
pub struct QueryString<'a, S = Vec<Pair<'a>>> {
    pairs: S,
    _pairs: PhantomData<Pair<'a>>,
}

// Only the `Vec`-backed builder is `Default`, so `QueryString::default()`
// keeps resolving without type annotations.
impl Default for QueryString<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// The panic message of the infallible methods when storage is full.
const STORAGE_FULL: &str = "query string pair storage is full";

impl<'a> QueryString<'a> {
    /// Creates a new, empty query string builder backed by a [`Vec`].
    ///
    /// This does not allocate until the first pair is added.
    pub const fn new() -> Self {
        Self::with_storage(Vec::new())
    }

    /// Creates a new, empty query string builder with room for at least
    /// `capacity` pairs before reallocating.
    ///
    /// ## Example
    ///
    /// ```
    /// use query_string_builder::QueryString;
    ///
    /// let qs = QueryString::with_capacity(2)
    ///     .with("q", "apple")
    ///     .with("category", "fruits");
    /// assert_eq!(qs.to_string(), "?q=apple&category=fruits");
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_storage(Vec::with_capacity(capacity))
    }

    /// Creates a new, empty query string builder holding up to `N` pairs
    /// inline, without any heap allocation.
    ///
    /// Adding more than `N` pairs through [`with`](QueryString::with) or
    /// [`push`](QueryString::push) panics; use
    /// [`try_with`](QueryString::try_with) or
    /// [`try_push`](QueryString::try_push) to handle a full builder.
    ///
    /// ## Example
    ///
    /// ```
    /// use query_string_builder::QueryString;
    ///
    /// let answer = 42;
    /// let qs = QueryString::inline::<2>()
    ///     .with("q", "🍎 apple")
    ///     .with("answer", &answer);
    ///
    /// assert_eq!(qs.to_string(), "?q=%F0%9F%8D%8E%20apple&answer=42");
    /// ```
    pub const fn inline<const N: usize>() -> QueryString<'a, InlinePairs<'a, N>> {
        QueryString::with_storage(InlinePairs::new())
    }
}

impl<'a, S: PairStorage<'a>> QueryString<'a, S> {
    /// Creates a query string builder on top of the given pair storage.
    ///
    /// Pairs already present in `storage` are kept.
    ///
    /// ## Example
    ///
    /// ```
    /// use query_string_builder::{InlinePairs, QueryString};
    ///
    /// let qs = QueryString::with_storage(InlinePairs::<4>::new()).with("q", "apple");
    /// assert_eq!(qs.to_string(), "?q=apple");
    /// ```
    pub const fn with_storage(storage: S) -> Self {
        Self {
            pairs: storage,
            _pairs: PhantomData,
        }
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
    ///
    /// ## Panics
    ///
    /// Panics if fixed-capacity storage is full.
    #[doc(alias = "with_value")]
    #[must_use]
    #[track_caller]
    pub fn with<K, V>(mut self, key: K, value: V) -> Self
    where
        K: IntoPart<'a>,
        V: IntoPart<'a>,
    {
        self.push(key, value);
        self
    }

    /// Appends a key-value pair to the query string, or fails if
    /// fixed-capacity storage is full.
    ///
    /// ## Example
    ///
    /// ```
    /// use query_string_builder::{CapacityError, QueryString};
    ///
    /// # fn main() -> Result<(), CapacityError> {
    /// let qs = QueryString::inline::<1>().try_with("q", "apple")?;
    /// assert_eq!(qs.to_string(), "?q=apple");
    /// assert!(qs.try_with("full", "yes").is_err());
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_with<K, V>(mut self, key: K, value: V) -> Result<Self, CapacityError>
    where
        K: IntoPart<'a>,
        V: IntoPart<'a>,
    {
        self.try_push(key, value)?;
        Ok(self)
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
    ///
    /// ## Panics
    ///
    /// Panics if fixed-capacity storage is full and the value exists.
    #[doc(alias = "with_opt_value")]
    #[must_use]
    #[track_caller]
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
    ///
    /// ## Panics
    ///
    /// Panics if fixed-capacity storage is full.
    #[track_caller]
    pub fn push<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: IntoPart<'a>,
        V: IntoPart<'a>,
    {
        match self.try_push(key, value) {
            Ok(this) => this,
            Err(CapacityError) => panic!("{STORAGE_FULL}"),
        }
    }

    /// Appends a key-value pair to the query string, or fails if
    /// fixed-capacity storage is full.
    ///
    /// ## Example
    ///
    /// ```
    /// use query_string_builder::{CapacityError, QueryString};
    ///
    /// let mut qs = QueryString::inline::<1>();
    /// assert!(qs.try_push("q", "apple").is_ok());
    /// assert_eq!(qs.try_push("full", "yes").err(), Some(CapacityError));
    /// assert_eq!(qs.to_string(), "?q=apple");
    /// ```
    pub fn try_push<K, V>(&mut self, key: K, value: V) -> Result<&mut Self, CapacityError>
    where
        K: IntoPart<'a>,
        V: IntoPart<'a>,
    {
        self.pairs
            .try_push((key.into_part(), value.into_part()))
            .map_err(|_| CapacityError)?;
        Ok(self)
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
    ///
    /// ## Panics
    ///
    /// Panics if fixed-capacity storage is full and the value exists.
    #[track_caller]
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
        self.pairs.as_slice().len()
    }

    /// Determines if the builder is currently empty.
    pub fn is_empty(&self) -> bool {
        self.pairs.as_slice().is_empty()
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
    ///
    /// The other builder may use a different storage type.
    ///
    /// ## Panics
    ///
    /// Panics if fixed-capacity storage runs full.
    #[track_caller]
    pub fn append<T: PairStorage<'a>>(&mut self, other: QueryString<'a, T>) {
        for &pair in other.pairs.as_slice() {
            if self.pairs.try_push(pair).is_err() {
                panic!("{STORAGE_FULL}");
            }
        }
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
    ///
    /// ## Panics
    ///
    /// Panics if fixed-capacity storage runs full.
    #[must_use]
    #[track_caller]
    pub fn append_into<T: PairStorage<'a>>(mut self, other: QueryString<'a, T>) -> Self {
        self.append(other);
        self
    }

    /// Converts this borrowing builder into a [`QueryStringOwned`] by rendering
    /// it once into the owned builder's buffer.
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
        let mut buf = String::with_capacity(self.encoded_len_hint());
        write!(buf, "{self}").expect("a Display implementation returned an error unexpectedly");
        QueryStringOwned::from_rendered(buf, self.len())
    }

    /// Returns a lower bound of the rendered length in bytes, exact when all
    /// keys and values are plain string slices.
    fn encoded_len_hint(&self) -> usize {
        let pairs = self.pairs.as_slice();
        if pairs.is_empty() {
            return 0;
        }
        // The leading '?', one '=' per pair and one '&' between pairs.
        let separators = 2 * pairs.len();
        pairs
            .iter()
            .map(|(key, value)| key.encoded_len_hint() + value.encoded_len_hint())
            .sum::<usize>()
            + separators
    }
}

impl<'a, S: PairStorage<'a>> Display for QueryString<'a, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let pairs = self.pairs.as_slice();
        if pairs.is_empty() {
            return Ok(());
        }
        f.write_char('?')?;
        for (i, (key, value)) in pairs.iter().enumerate() {
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

impl<'a, S: PairStorage<'a>> Debug for QueryString<'a, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(
                self.pairs
                    .as_slice()
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
    fn test_into_owned_exact_capacity() {
        let qs = QueryString::new()
            .with("q", "apple???")
            .with("category", "fruits and vegetables");
        let rendered = qs.to_string();
        let owned = qs.into_owned();
        assert_eq!(owned.as_str(), rendered);
        assert_eq!(owned.capacity(), rendered.len());
    }

    #[test]
    fn test_inline() {
        let tasty = true;
        let qs = QueryString::inline::<3>()
            .with("q", "Grünkohl")
            .with_opt("color", None::<&str>)
            .with("tasty", &tasty);
        assert_eq!(qs.to_string(), "?q=Gr%C3%BCnkohl&tasty=true");
        assert_eq!(qs.len(), 2);
        assert_eq!(format!("{qs:?}"), r#"{"q": "Grünkohl", "tasty": "true"}"#);
    }

    #[test]
    fn test_inline_empty() {
        let qs = QueryString::inline::<0>();
        assert_eq!(qs.to_string(), "");
        assert!(qs.is_empty());
    }

    #[test]
    #[should_panic(expected = "query string pair storage is full")]
    fn test_inline_overflow_panics() {
        let _ = QueryString::inline::<1>().with("a", "1").with("b", "2");
    }

    #[test]
    fn test_inline_try_push_full() {
        let mut qs = QueryString::inline::<1>();
        qs.try_push("a", "1").unwrap();
        assert_eq!(qs.try_push("b", "2").err(), Some(CapacityError));
        assert_eq!(qs.to_string(), "?a=1");
    }

    #[test]
    fn test_append_across_storage() {
        let mut qs = QueryString::inline::<3>().with("a", "1");
        qs.append(QueryString::new().with("b", "2").with("c", "3"));
        assert_eq!(qs.to_string(), "?a=1&b=2&c=3");

        let qs = QueryString::new().append_into(qs);
        assert_eq!(qs.to_string(), "?a=1&b=2&c=3");
    }

    #[test]
    #[should_panic(expected = "query string pair storage is full")]
    fn test_append_overflow_panics() {
        let mut qs = QueryString::inline::<1>().with("a", "1");
        qs.append(QueryString::new().with("b", "2"));
    }

    #[test]
    fn test_inline_into_owned() {
        let qs = QueryString::inline::<1>().with("q", "a b");
        assert_eq!(qs.into_owned().as_str(), "?q=a%20b");
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
