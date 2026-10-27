//! Allocation-count tests backing the crate's zero-allocation claims.
//!
//! A counting global allocator tracks allocations per thread, so tests
//! running in parallel do not interfere with each other.

use query_string_builder::{QueryString, QueryStringOwned};
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::fmt::Write;

struct CountingAllocator;

thread_local! {
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.with(|n| n.set(n.get() + 1));
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCATIONS.with(|n| n.set(n.get() + 1));
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

/// Runs `f` and returns its result along with the number of allocations
/// (including reallocations) it performed on this thread.
fn count<T>(f: impl FnOnce() -> T) -> (T, usize) {
    let before = ALLOCATIONS.with(Cell::get);
    let result = f();
    let after = ALLOCATIONS.with(Cell::get);
    (result, after - before)
}

#[test]
fn owned_new_does_not_allocate() {
    let (qs, allocations) = count(QueryStringOwned::new);
    assert_eq!(allocations, 0);
    assert!(qs.is_empty());
}

#[test]
fn owned_with_capacity_builds_with_one_allocation() {
    let tasty = true;
    let (qs, allocations) = count(|| {
        QueryStringOwned::with_capacity(128)
            .with("q", "🍎 apple")
            .with("answer", 42)
            .with("tasty", tasty)
            .with("weight", 99.9)
            .with_opt("color", None::<&str>)
            .with_opt("category", Some("fruits and vegetables"))
    });
    assert_eq!(allocations, 1, "only the up-front buffer allocation");
    assert_eq!(qs.len(), 5);
}

#[test]
fn owned_rendering_does_not_allocate() {
    let qs = QueryStringOwned::new()
        .with("q", "🍎 apple")
        .with("answer", 42);
    let mut out = String::with_capacity(64);

    let ((), allocations) = count(|| {
        write!(out, "{qs}").unwrap();
        let _ = qs.as_str();
    });
    assert_eq!(allocations, 0);

    let (rendered, allocations) = count(|| qs.into_string());
    assert_eq!(allocations, 0);
    assert_eq!(rendered, out);
}

#[test]
fn owned_append_into_presized_does_not_allocate() {
    let mut qs = QueryStringOwned::with_capacity(64).with("a", 1);
    let more = QueryStringOwned::new().with("b", 2).with("c", 3);

    let ((), allocations) = count(|| qs.append(more));
    assert_eq!(allocations, 0);
    assert_eq!(qs.as_str(), "?a=1&b=2&c=3");
}

#[test]
fn borrowed_rendering_does_not_allocate() {
    let answer = 42;
    let qs = QueryString::new()
        .with("q", "🍎 apple")
        .with("answer", &answer);
    let mut out = String::with_capacity(64);

    let ((), allocations) = count(|| write!(out, "{qs}").unwrap());
    assert_eq!(allocations, 0);
    assert_eq!(out, "?q=%F0%9F%8D%8E%20apple&answer=42");
}

#[test]
fn borrowed_into_owned_renders_once() {
    let qs = QueryString::new()
        .with("q", "apple???")
        .with("category", "fruits and vegetables");

    let (owned, allocations) = count(|| qs.into_owned());
    assert_eq!(allocations, 1, "string slices are sized exactly up front");
    assert_eq!(owned.len(), 2);
}

#[test]
fn borrowed_inline_never_allocates() {
    let answer = 42;
    let mut out = String::with_capacity(64);

    let (qs, allocations) = count(|| {
        QueryString::inline::<3>()
            .with("q", "🍎 apple")
            .with("answer", &answer)
            .with_opt("color", None::<&str>)
    });
    assert_eq!(allocations, 0, "building with inline storage");

    let ((), allocations) = count(|| write!(out, "{qs}").unwrap());
    assert_eq!(allocations, 0, "rendering with inline storage");
    assert_eq!(out, "?q=%F0%9F%8D%8E%20apple&answer=42");
}

#[test]
fn borrowed_with_capacity_builds_with_one_allocation() {
    let answer = 42;
    let (qs, allocations) = count(|| {
        QueryString::with_capacity(4)
            .with("q", "🍎 apple")
            .with("answer", &answer)
            .with("a", "1")
            .with("b", "2")
    });
    assert_eq!(allocations, 1, "only the up-front pair allocation");
    assert_eq!(qs.len(), 4);
}

#[test]
fn borrowed_encoded_len_does_not_allocate() {
    let answer = 42;
    let weight = 99.9;
    let qs = QueryString::inline::<3>()
        .with("q", "🍎 apple")
        .with("answer", &answer)
        .with("weight", &weight);

    let (len, allocations) = count(|| qs.encoded_len());
    assert_eq!(allocations, 0);
    assert_eq!(len, qs.to_string().len());
}

#[test]
fn borrowed_write_into_reserves_once() {
    let answer = 42;
    let weight = 99.9;
    let qs = QueryString::inline::<3>()
        .with("q", "🍎 apple and a long description to force growth")
        .with("answer", &answer)
        .with("weight", &weight);
    let mut url = String::from("https://example.com/");

    let ((), allocations) = count(|| qs.write_into(&mut url));
    assert_eq!(allocations, 1, "a single exact reserve");
    assert_eq!(url, format!("https://example.com/{qs}"));
}

#[test]
fn borrowed_into_owned_with_display_values_allocates_once() {
    let answer = 42;
    let weight = 99.9;
    let qs = QueryString::inline::<3>()
        .with("q", "apple???")
        .with("answer", &answer)
        .with("weight", &weight);

    let (owned, allocations) = count(|| qs.into_owned());
    assert_eq!(allocations, 1, "Display values are measured, not guessed");
    assert_eq!(owned.capacity(), owned.as_str().len());
}

#[test]
fn borrowed_debug_does_not_allocate() {
    let answer = 42;
    let qs = QueryString::inline::<2>()
        .with("q", r#"🍎 "apple""#)
        .with("answer", &answer);
    let mut out = String::with_capacity(128);

    let ((), allocations) = count(|| write!(out, "{qs:?}").unwrap());
    assert_eq!(allocations, 0);
    assert_eq!(out, r#"{"q": "🍎 \"apple\"", "answer": "42"}"#);
}
