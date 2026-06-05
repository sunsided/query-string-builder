use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use query_string_builder::{QueryString, QueryStringOwned};

pub fn criterion_benchmark(c: &mut Criterion) {
    // `with` method benchmarks
    c.bench_function("borrowed_with", |b| {
        b.iter(|| {
            let qs = QueryString::new()
                .with("q", "apple???")
                .with("category", "fruits and vegetables");
            black_box(format!("{qs}"))
        })
    });

    c.bench_function("owned_with", |b| {
        b.iter(|| {
            let qs = QueryStringOwned::new()
                .with("q", "apple???")
                .with("category", "fruits and vegetables");
            black_box(format!("{qs}"))
        })
    });

    // `with_opt` method benchmarks
    c.bench_function("borrowed_with_opt", |b| {
        let tasty = true;
        let weight = 99.9;
        b.iter(|| {
            let qs = QueryString::new()
                .with("q", "celery")
                .with_opt("taste", None::<&str>)
                .with_opt("category", Some("fruits and vegetables"))
                .with_opt("tasty", Some(&tasty))
                .with_opt("weight", Some(&weight));
            black_box(format!("{qs}"))
        })
    });

    c.bench_function("owned_with_opt", |b| {
        b.iter(|| {
            let qs = QueryStringOwned::new()
                .with("q", "celery")
                .with_opt("taste", None::<String>)
                .with_opt("category", Some("fruits and vegetables"))
                .with_opt("tasty", Some(true))
                .with_opt("weight", Some(99.9));
            black_box(format!("{qs}"))
        })
    });

    // Full test including creating, pushing and appending
    c.bench_function("borrowed_push_opt_and_append", |b| {
        b.iter(|| {
            let mut qs = QueryString::new();
            qs.push("a", "apple");
            qs.push_opt("b", None::<&str>);
            qs.push_opt("c", Some("🍎 apple"));

            let more = QueryString::new().with("q", "pear");
            let qs = qs.append_into(more);

            black_box(format!("{qs}"))
        })
    });

    c.bench_function("owned_push_opt_and_append", |b| {
        b.iter(|| {
            let mut qs = QueryStringOwned::new();
            qs.push("a", "apple");
            qs.push_opt("b", None::<String>);
            qs.push_opt("c", Some("🍎 apple"));

            let more = QueryStringOwned::new().with("q", "pear");
            let qs = qs.append_into(more);

            black_box(format!("{qs}"))
        })
    });

    // Bridge cost: build borrowed, convert to owned, render
    c.bench_function("borrowed_into_owned", |b| {
        b.iter(|| {
            let qs = QueryString::new()
                .with("q", "apple???")
                .with("category", "fruits and vegetables")
                .into_owned();
            black_box(format!("{qs}"))
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
