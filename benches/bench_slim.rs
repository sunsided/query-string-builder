use criterion::{criterion_group, criterion_main, Criterion};

use query_string_builder::QueryString;

pub fn criterion_benchmark(c: &mut Criterion) {
    // `with_value` method benchmark
    c.bench_function("with_value (slim)", |b| {
        b.iter(|| {
            let qs = QueryString::simple()
                .with_value("q", "apple???")
                .with_value("category", "fruits and vegetables");
            format!("{qs}")
        })
    });

    // `with_opt_value` method benchmark
    c.bench_function("with_opt_value (slim)", |b| {
        b.iter(|| {
            let qs = QueryString::simple()
                .with_value("q", "celery")
                .with_opt_value("taste", None::<String>)
                .with_opt_value("category", Some("fruits and vegetables"))
                .with_opt_value("tasty", Some(true))
                .with_opt_value("weight", Some(99.9));
            format!("{qs}")
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
