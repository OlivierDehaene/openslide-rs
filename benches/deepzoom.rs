use std::path::Path;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use openslide_rs::{Address, DeepZoom, OpenSlide};

fn criterion_benchmark(c: &mut Criterion) {
    let slide = OpenSlide::open(Path::new("tests/assets/default.svs")).unwrap();
    let dz = DeepZoom::new(&slide, 1024, 0, false);

    c.bench_function("read_tile", |b| {
        b.iter(|| dz.read_tile(black_box(9), black_box(Address { x: 0, y: 0 })))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
