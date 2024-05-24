use criterion::{black_box, criterion_group, criterion_main, Criterion};

use query_string_builder::QueryString;

pub fn criterion_benchmark(c: &mut Criterion) {
    // `with_value` method benchmark
    c.bench_function("with_value", |b| {
        b.iter(|| {
            let qs = QueryString::new()
                .with_value("q", "apple???")
                .with_value("category", "fruits and vegetables");
            format!("{qs}")
        })
    });

    // `with_opt_value` method benchmark
    c.bench_function("with_opt_value", |b| {
        b.iter(|| {
            let qs = QueryString::new()
                .with_value("q", "celery")
                .with_opt_value("taste", None::<String>)
                .with_opt_value("category", Some("fruits and vegetables"))
                .with_opt_value("tasty", Some(true))
                .with_opt_value("weight", Some(99.9));
            format!("{qs}")
        })
    });

    // Full test including creating, pushing and appending
    c.bench_function("push_opt_and_append", |b| {
        b.iter(|| {
            let mut qs = QueryString::new();
            qs.push("a", "apple");
            qs.push_opt("b", None::<String>);
            qs.push_opt("c", Some("🍎 apple"));

            let more = QueryString::new().with_value("q", "pear");
            let qs = qs.append_into(more);

            format!("{qs}")
        })
    });

    // Test focusing on printing more often than creating.
    c.bench_function("print_alot", |b| {
        let qs = QueryString::new()
            .with_value("q", "apple???")
            .with_value("category", "fruits and vegetables");
        b.iter(|| format!("{}", black_box(&qs)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
