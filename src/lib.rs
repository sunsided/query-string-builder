//! # A query string builder for percent encoding key-value pairs
//!
//! This is a tiny helper crate for simplifying the construction of URL query strings.
//! The initial `?` question mark is automatically prepended.
//!
//! Two builders are provided:
//!
//! - [`QueryString`] borrows its keys and values and never allocates strings —
//!   neither while building nor while rendering. Use it when you assemble and
//!   format the query string in one place.
//! - [`QueryStringOwned`] owns its data and has no lifetime parameter. Use it
//!   when you want to store the builder in a struct or return it from a
//!   function. A borrowing builder can be converted via
//!   [`QueryString::into_owned`].
//!
//! ## Example
//!
//! ```
//! use query_string_builder::QueryString;
//!
//! let tasty = true;
//! let qs = QueryString::new()
//!     .with("q", "🍎 apple")
//!     .with("tasty", &tasty)
//!     .with_opt("color", None::<&str>)
//!     .with_opt("category", Some("fruits and vegetables?"));
//!
//! assert_eq!(
//!     format!("example.com/{qs}"),
//!     "example.com/?q=%F0%9F%8D%8E%20apple&tasty=true&category=fruits%20and%20vegetables?"
//! );
//! ```
//!
//! Since [`QueryString`] borrows its values, inline temporaries don't live long
//! enough — bind them to a variable first, or use [`QueryStringOwned`]:
//!
//! ```
//! use query_string_builder::QueryStringOwned;
//!
//! let qs = QueryStringOwned::new()
//!     .with("answer", 42)
//!     .with("works", true);
//!
//! assert_eq!(format!("example.com/{qs}"), "example.com/?answer=42&works=true");
//! ```

#![deny(unsafe_code)]

mod borrowed;
mod encode;
mod owned;

use percent_encoding::{AsciiSet, CONTROLS};

pub use borrowed::{IntoPart, Part, QueryString};
pub use owned::QueryStringOwned;

/// https://url.spec.whatwg.org/#query-percent-encode-set
pub(crate) const QUERY: &AsciiSet = &CONTROLS
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
