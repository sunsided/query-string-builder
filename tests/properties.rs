//! Property tests for the query string builders.

use percent_encoding::percent_decode_str;
use proptest::prelude::*;
use query_string_builder::{QueryString, QueryStringOwned};
use std::fmt::{self, Display, Formatter};

/// A `Display` implementation that writes `s` in chunks split at the given
/// char-boundary offsets, simulating arbitrary `Display` implementations.
struct ChunkedStr<'a> {
    s: &'a str,
    splits: Vec<usize>,
}

impl Display for ChunkedStr<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut start = 0;
        for &end in &self.splits {
            f.write_str(&self.s[start..end])?;
            start = end;
        }
        f.write_str(&self.s[start..])
    }
}

/// Generates a string together with a sorted list of char-boundary split points.
fn string_with_splits() -> impl Strategy<Value = (String, Vec<usize>)> {
    any::<String>().prop_flat_map(|s| {
        let boundaries: Vec<usize> = s.char_indices().map(|(i, _)| i).skip(1).collect();
        let len = boundaries.len();
        (
            Just(s),
            proptest::collection::vec(0..=len.max(1) - 1, 0..=len.min(8)).prop_map(move |idx| {
                let mut splits: Vec<usize> = idx
                    .into_iter()
                    .filter(|&i| i < len)
                    .map(|i| boundaries[i])
                    .collect();
                splits.sort_unstable();
                splits.dedup();
                splits
            }),
        )
    })
}

fn pairs() -> impl Strategy<Value = Vec<(String, String)>> {
    proptest::collection::vec((any::<String>(), any::<String>()), 1..8)
}

proptest! {
    /// The EncodeWriter invariant: a Display value writing in arbitrary chunks
    /// renders identically to the same string passed as a plain `&str`.
    #[test]
    fn chunk_split_equivalence((s, splits) in string_with_splits()) {
        let chunked = ChunkedStr { s: &s, splits };
        let via_chunks = QueryString::new().with("k", &chunked).to_string();
        let via_str = QueryString::new().with("k", s.as_str()).to_string();
        prop_assert_eq!(via_chunks, via_str);
    }

    /// Percent-decoding the rendered output recovers the original pairs.
    #[test]
    fn roundtrip_decode(pairs in pairs()) {
        let mut qs = QueryString::new();
        for (key, value) in &pairs {
            qs.push(key.as_str(), value.as_str());
        }
        let rendered = qs.to_string();

        let body = rendered.strip_prefix('?').expect("non-empty builder must render with leading '?'");
        let decoded: Vec<(String, String)> = body
            .split('&')
            .map(|pair| {
                let (key, value) = pair.split_once('=').expect("each pair must contain '='");
                (
                    percent_decode_str(key).decode_utf8().expect("valid UTF-8").into_owned(),
                    percent_decode_str(value).decode_utf8().expect("valid UTF-8").into_owned(),
                )
            })
            .collect();
        prop_assert_eq!(decoded, pairs);
    }

    /// Borrowed and owned builders render identically; `into_owned` preserves output.
    #[test]
    fn owned_borrowed_equivalence(pairs in pairs()) {
        let mut borrowed = QueryString::new();
        let mut owned = QueryStringOwned::new();
        for (key, value) in &pairs {
            borrowed.push(key.as_str(), value.as_str());
            owned.push(key, value);
        }
        let rendered = borrowed.to_string();
        prop_assert_eq!(&rendered, &owned.to_string());
        prop_assert_eq!(&rendered, &borrowed.into_owned().to_string());
    }

    /// The rendered output never contains raw characters from the encode set,
    /// and separators structure the output into exactly `len()` pairs.
    #[test]
    fn well_formedness(pairs in pairs()) {
        let mut qs = QueryString::new();
        for (key, value) in &pairs {
            qs.push(key.as_str(), value.as_str());
        }
        let expected_len = qs.len();
        let rendered = qs.to_string();
        let body = rendered.strip_prefix('?').expect("leading '?'");

        for forbidden in [' ', '"', '#', '<', '>', '+'] {
            prop_assert!(!body.contains(forbidden), "raw {forbidden:?} in {body:?}");
        }
        prop_assert!(body.chars().all(|c| !c.is_control()), "raw control char in {body:?}");

        let parts: Vec<&str> = body.split('&').collect();
        prop_assert_eq!(parts.len(), expected_len);
        for part in parts {
            prop_assert_eq!(part.matches('=').count(), 1, "exactly one '=' in {}", part);
        }
    }
}
