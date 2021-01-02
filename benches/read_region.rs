use std::path::Path;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use openslide_rs::{Address, OpenSlide, Region, Size};

fn read_region_benchmark(c: &mut Criterion) {
    let slide = OpenSlide::open(Path::new("tests/assets/default.svs")).unwrap();

    c.bench_function("read_region", |b| {
        b.iter(|| {
            slide.read_region(black_box(Region {
                address: Address { x: 0, y: 0 },
                level: 0,
                size: Size { w: 512, h: 512 },
            }))
        })
    });
}

criterion_group!(benches, read_region_benchmark);
criterion_main!(benches);
