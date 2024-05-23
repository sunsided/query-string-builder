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

use std::fmt::{Display, Formatter};
use std::rc::Rc;
use std::sync::Arc;

use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};

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
#[derive(Default)]
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
        self.pairs.push(Kvp {
            key: key.to_string(),
            value: Value::eager(value),
        });
        self
    }

    /// Appends a key-value pair to the query string. Supports lazy evaluation.
    ///
    /// ## Example
    ///
    /// ```
    /// use std::sync::Arc;
    /// use query_string_builder::{QueryString, Value};
    ///
    /// let qs = QueryString::new()
    ///             .with("q", Value::eager("🍎 apple"))
    ///             .with("category", || "fruits and vegetables")
    ///             .with("answer", Arc::new(42));
    ///
    /// assert_eq!(
    ///     format!("https://example.com/{qs}"),
    ///     "https://example.com/?q=%F0%9F%8D%8E%20apple&category=fruits%20and%20vegetables&answer=42"
    /// );
    /// ```
    pub fn with<K: ToString, V: Into<Value>>(mut self, key: K, value: V) -> Self {
        self.pairs.push(Kvp {
            key: key.to_string(),
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
        self.pairs.push(Kvp {
            key: key.to_string(),
            value: Value::eager(value),
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

                // We convert the value to a String here. Ideally we'd print it directly (skipping
                // the intermediate string generation) but due to the required percent encoding
                // this isn't immediately possible.
                let value = pair.value.render();

                write!(
                    f,
                    "{key}={value}",
                    key = utf8_percent_encode(&pair.key, FRAGMENT),
                    value = utf8_percent_encode(&value, FRAGMENT)
                )?;
            }
            Ok(())
        }
    }
}

struct Kvp {
    key: String,
    value: Value,
}

pub struct Value {
    value: Box<dyn Fn() -> String>,
}

impl Value {
    pub fn eager<T: ToString>(value: T) -> Self {
        let value = value.to_string();
        Value {
            value: Box::new(move || value.to_string()),
        }
    }

    pub fn lazy<T: ToString + 'static>(value: T) -> Self {
        Value {
            value: Box::new(move || value.to_string()),
        }
    }

    pub fn lazy_box<T: ToString + 'static>(value: Box<T>) -> Self {
        Value {
            value: Box::new(move || value.to_string()),
        }
    }

    pub fn lazy_rc<T: ToString + 'static>(value: Rc<T>) -> Self {
        Value {
            value: Box::new(move || value.to_string()),
        }
    }

    pub fn lazy_arc<T: ToString + 'static>(value: Arc<T>) -> Self {
        Value {
            value: Box::new(move || value.to_string()),
        }
    }

    pub fn lazy_fn<F, T>(func: F) -> Self
    where
        F: Fn() -> T + 'static,
        T: ToString + 'static,
    {
        Value {
            value: Box::new(move || func().to_string()),
        }
    }

    fn render(&self) -> String {
        (self.value)()
    }
}

impl<T> From<Rc<T>> for Value
where
    T: ToString + 'static,
{
    fn from(value: Rc<T>) -> Self {
        Value::lazy_rc(value)
    }
}

impl<T> From<Arc<T>> for Value
where
    T: ToString + 'static,
{
    fn from(value: Arc<T>) -> Self {
        Value::lazy_arc(value)
    }
}

impl<F, T> From<F> for Value
where
    F: Fn() -> T + 'static,
    T: ToString + 'static,
{
    fn from(value: F) -> Self {
        Value::lazy_fn(value)
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
    fn test_lazy() {
        let qs = QueryString::new()
            .with("x", Value::eager("y"))
            .with("q", Value::lazy("apple???"))
            .with("category", Value::lazy_fn(|| "fruits and vegetables"))
            .with("tasty", Value::lazy_box(Box::new(true)))
            .with("weight", Value::lazy_arc(Arc::new(99.9)));
        assert_eq!(
            qs.to_string(),
            "?x=y&q=apple???&category=fruits%20and%20vegetables&tasty=true&weight=99.9"
        );
    }

    #[test]
    fn test_lazy_implicit() {
        let qs = QueryString::new()
            .with("q", Value::lazy("apple???"))
            .with("category", || "fruits and vegetables")
            .with("tasty", Rc::new(true))
            .with("weight", Arc::new(99.9));
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
}
