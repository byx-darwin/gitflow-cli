use criterion::{Criterion, black_box, criterion_group, criterion_main};
use gitflow_cli_core::Config;

fn config_new(c: &mut Criterion) {
    c.bench_function("Config::new", |b| {
        b.iter(|| Config::new(black_box("bench")))
    });
}

fn config_new_with_description(c: &mut Criterion) {
    c.bench_function("Config::new + with_description", |b| {
        b.iter(|| {
            Config::new(black_box("bench")).map(|c| c.with_description(black_box("description")))
        })
    });
}

criterion_group!(benches, config_new, config_new_with_description);
criterion_main!(benches);
