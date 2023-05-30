use criterion::{criterion_group, criterion_main, Criterion};

pub fn insert_order(c: &mut Criterion) {
    // c.bench_function("", |b| b.iter(||));
}

pub fn remove_order(c: &mut Criterion) {
    // c.bench_function("", |b| b.iter(||));
}

pub fn update_order(c: &mut Criterion) {
    // c.bench_function("", |b| b.iter(||));
}

criterion_group!(benches, insert_order, remove_order, update_order);
criterion_main!(benches);
