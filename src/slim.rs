use std::fmt;
use std::fmt::{Debug, Display, Formatter, Write};

use crate::{QueryString, QUERY};
use percent_encoding::utf8_percent_encode;

/// A type alias for the [`WrappedQueryString`] root.
pub type QueryStringSimple = WrappedQueryString<RootMarker, EmptyValue>;

/// A query string builder for percent encoding key-value pairs.
/// This variant reduces string allocations as much as possible, defers them to the
/// time of actual rendering, and is capable of storing references.
///
/// ## Example
///
/// ```
/// use query_string_builder::QueryString;
///
/// let weight: &f32 = &99.9;
///
/// let qs = QueryString::simple()
///             .with_value("q", "apple")
///             .with_value("category", "fruits and vegetables")
///             .with_opt_value("weight", Some(weight));
///
/// assert_eq!(
///     format!("https://example.com/{qs}"),
///     "https://example.com/?q=apple&category=fruits%20and%20vegetables&weight=99.9"
/// );
/// ```
pub struct WrappedQueryString<B, T>
where
    B: ConditionalDisplay + Identifyable,
    T: Display,
{
    base: BaseOption<B>,
    value: KvpOption<T>,
}

impl Default for QueryStringSimple {
    fn default() -> Self {
        QueryString::simple()
    }
}

/// A helper type to track the values of [`WrappedQueryString`].
pub struct Kvp<K, V>
where
    K: Display,
    V: Display,
{
    key: K,
    value: V,
}

enum BaseOption<B> {
    Some(B),
    None,
}

enum KvpOption<T> {
    Some(T),
    None,
}

/// This type serves as a root marker for the builder. It has no public constructor,
/// thus can only be created within this crate.
pub struct RootMarker(());

/// This type serves as an empty value marker for the builder. It has no public constructor,
/// thus can only be created within this crate.
pub struct EmptyValue(());

impl<B, T> WrappedQueryString<B, T>
where
    B: ConditionalDisplay + Identifyable,
    T: Display,
{
    /// Creates a new, empty query string builder.
    pub(crate) fn new() -> WrappedQueryString<RootMarker, EmptyValue> {
        WrappedQueryString {
            base: BaseOption::None,
            value: KvpOption::None,
        }
    }

    /// Appends a key-value pair to the query string.
    ///
    /// ## Example
    ///
    /// ```
    /// use query_string_builder::QueryString;
    ///
    /// let qs = QueryString::dynamic()
    ///             .with_value("q", "🍎 apple")
    ///             .with_value("category", "fruits and vegetables")
    ///             .with_value("answer", 42);
    ///
    /// assert_eq!(
    ///     format!("https://example.com/{qs}"),
    ///     "https://example.com/?q=%F0%9F%8D%8E%20apple&category=fruits%20and%20vegetables&answer=42"
    /// );
    /// ```
    pub fn with_value<K: Display, V: Display>(
        self,
        key: K,
        value: V,
    ) -> WrappedQueryString<Self, Kvp<K, V>> {
        WrappedQueryString {
            base: BaseOption::Some(self),
            value: KvpOption::Some(Kvp { key, value }),
        }
    }

    /// Appends a key-value pair to the query string if the value exists.
    ///
    /// ## Example
    ///
    /// ```
    /// use query_string_builder::QueryString;
    ///
    /// let qs = QueryString::dynamic()
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
    pub fn with_opt_value<K: Display, V: Display>(
        self,
        key: K,
        value: Option<V>,
    ) -> WrappedQueryString<Self, Kvp<K, V>> {
        if let Some(value) = value {
            WrappedQueryString {
                base: BaseOption::Some(self),
                value: KvpOption::Some(Kvp { key, value }),
            }
        } else {
            WrappedQueryString {
                base: BaseOption::Some(self),
                value: KvpOption::None,
            }
        }
    }

    /// Determines the number of key-value pairs currently in the builder.
    pub fn len(&self) -> usize {
        if self.is_empty() {
            return 0;
        }

        1 + self.base.len()
    }

    /// Determines if the builder is currently empty.
    pub fn is_empty(&self) -> bool {
        // If this is the root node, and we don't have a value, we're empty.
        if self.is_root() && self.value.is_empty() {
            return true;
        }

        // If we're not the root node we need to check if all values are empty.
        if !self.value.is_empty() {
            return false;
        }

        self.base.is_empty()
    }
}

pub trait Identifyable {
    fn is_root(&self) -> bool;
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
}

impl Identifyable for RootMarker {
    fn is_root(&self) -> bool {
        true
    }

    fn is_empty(&self) -> bool {
        true
    }

    fn len(&self) -> usize {
        0
    }
}

pub trait ConditionalDisplay {
    fn cond_fmt(&self, should_display: bool, f: &mut Formatter<'_>) -> Result<usize, fmt::Error>;
}

impl ConditionalDisplay for RootMarker {
    fn cond_fmt(&self, _should_display: bool, _f: &mut Formatter<'_>) -> Result<usize, fmt::Error> {
        unreachable!()
    }
}

impl<B> ConditionalDisplay for BaseOption<B>
where
    B: ConditionalDisplay,
{
    fn cond_fmt(&self, should_display: bool, f: &mut Formatter<'_>) -> Result<usize, fmt::Error> {
        match self {
            BaseOption::Some(base) => Ok(base.cond_fmt(should_display, f)?),
            BaseOption::None => {
                // Reached the root marker.
                if should_display {
                    f.write_char('?')?;
                }
                Ok(0)
            }
        }
    }
}

impl<B, T> ConditionalDisplay for WrappedQueryString<B, T>
where
    B: ConditionalDisplay + Identifyable,
    T: Display,
{
    fn cond_fmt(&self, should_display: bool, f: &mut Formatter<'_>) -> Result<usize, fmt::Error> {
        let depth = if !should_display {
            // Our caller had nothing to display. If we have nothing to display either,
            // we move on to our parent.
            if self.value.is_empty() {
                return self.base.cond_fmt(false, f);
            }

            // We do have things to display - render the parent!
            self.base.cond_fmt(true, f)?
        } else {
            // The caller has things to display - go ahead regardless.
            self.base.cond_fmt(true, f)?
        };

        // If we have nothing to render, return the known depth.
        if self.value.is_empty() {
            return Ok(depth);
        }

        // Display and increase the depth.
        self.value.fmt(f)?;

        // If our parent indicated content was displayable, add the combinator.
        if should_display {
            f.write_char('&')?;
        }

        Ok(depth + 1)
    }
}

impl<B> BaseOption<B>
where
    B: Identifyable + ConditionalDisplay,
{
    fn is_empty(&self) -> bool {
        match self {
            BaseOption::Some(value) => value.is_empty(),
            BaseOption::None => true,
        }
    }

    fn len(&self) -> usize {
        match self {
            BaseOption::Some(value) => value.len(),
            BaseOption::None => 0,
        }
    }
}

impl<B, T> Identifyable for WrappedQueryString<B, T>
where
    B: ConditionalDisplay + Identifyable,
    T: Display,
{
    fn is_root(&self) -> bool {
        match self.base {
            BaseOption::Some(_) => false,
            BaseOption::None => true,
        }
    }

    fn is_empty(&self) -> bool {
        match self.value {
            KvpOption::Some(_) => false,
            KvpOption::None => self.base.is_empty(),
        }
    }

    fn len(&self) -> usize {
        match self.value {
            KvpOption::Some(_) => 1 + self.base.len(),
            KvpOption::None => self.base.len(),
        }
    }
}

impl<T> KvpOption<T> {
    fn is_empty(&self) -> bool {
        match self {
            KvpOption::Some(_) => false,
            KvpOption::None => true,
        }
    }
}

impl Display for RootMarker {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_char('?')
    }
}

impl Display for EmptyValue {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl<T> Display for BaseOption<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BaseOption::Some(d) => Display::fmt(d, f),
            BaseOption::None => Ok(()),
        }
    }
}

impl<T> Display for KvpOption<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            KvpOption::Some(d) => Display::fmt(d, f),
            KvpOption::None => Ok(()),
        }
    }
}

impl<K, V> Display for Kvp<K, V>
where
    K: Display,
    V: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&utf8_percent_encode(&self.key.to_string(), QUERY), f)?;
        f.write_char('=')?;
        Display::fmt(&utf8_percent_encode(&self.value.to_string(), QUERY), f)
    }
}

impl<B, T> Display for WrappedQueryString<B, T>
where
    B: ConditionalDisplay + Identifyable,
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let should_display = !self.value.is_empty();

        self.base.cond_fmt(should_display, f)?;
        if should_display {
            Display::fmt(&self.value, f)?;
        }

        Ok(())
    }
}

impl<B, T> Debug for WrappedQueryString<B, T>
where
    B: ConditionalDisplay + Identifyable,
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use crate::QueryString;

    #[test]
    fn test_empty() {
        let qs = QueryString::simple();

        assert!(qs.is_empty());
        assert_eq!(qs.len(), 0);

        assert_eq!(qs.to_string(), "");
    }

    #[test]
    fn test_empty_complex() {
        let qs = QueryString::simple().with_opt_value("key", None::<&str>);

        assert!(qs.is_empty());
        assert_eq!(qs.len(), 0);

        assert_eq!(qs.to_string(), "");
    }

    #[test]
    fn test_simple() {
        let apple = "apple???";

        let qs = QueryString::simple()
            .with_value("q", &apple)
            .with_value("category", "fruits and vegetables")
            .with_value("tasty", true)
            .with_value("weight", 99.9);

        assert!(!qs.is_empty());
        assert_eq!(qs.len(), 4);

        assert_eq!(
            qs.to_string(),
            "?q=apple???&category=fruits%20and%20vegetables&tasty=true&weight=99.9"
        );
    }

    #[test]
    fn test_encoding() {
        let qs = QueryString::simple()
            .with_value("q", "Grünkohl")
            .with_value("category", "Gemüse");

        assert!(!qs.is_empty());
        assert_eq!(qs.len(), 2);

        assert_eq!(qs.to_string(), "?q=Gr%C3%BCnkohl&category=Gem%C3%BCse");
    }

    #[test]
    fn test_emoji() {
        let qs = QueryString::simple()
            .with_value("q", "🥦")
            .with_value("🍽️", "🍔🍕");

        assert!(!qs.is_empty());
        assert_eq!(qs.len(), 2);

        assert_eq!(
            qs.to_string(),
            "?q=%F0%9F%A5%A6&%F0%9F%8D%BD%EF%B8%8F=%F0%9F%8D%94%F0%9F%8D%95"
        );
    }

    #[test]
    fn test_optional() {
        let qs = QueryString::simple()
            .with_value("q", "celery")
            .with_opt_value("taste", None::<String>)
            .with_opt_value("category", Some("fruits and vegetables"))
            .with_opt_value("tasty", Some(true))
            .with_opt_value("weight", Some(99.9));

        assert!(!qs.is_empty());
        assert_eq!(qs.len(), 4);

        assert_eq!(
            qs.to_string(),
            "?q=celery&category=fruits%20and%20vegetables&tasty=true&weight=99.9"
        );
        assert_eq!(qs.len(), 4); // not five!
    }
}
