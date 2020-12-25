use std::path::Path;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use openslide_rs::{Address, DeepZoom, OpenSlide, Region, Size};

fn read_region_benchmark(c: &mut Criterion) {
    let mut slide = OpenSlide::open(Path::new("tests/assets/default.svs")).unwrap();
    slide.set_cache_size(0).unwrap();

    c.bench_function("read_region", |b| {
        b.iter(|| {
            slide.read_region(black_box(Region {
                address: Address { x: 0, y: 0 },
                level: 0,
                size: Size { w: 512, h: 512 },
            }))
        })
    });

    let dz = DeepZoom::new(&slide, 224, 0, false).unwrap();

    c.bench_function("dz_read_tile", |b| {
        b.iter(|| dz.read_tile(black_box(9), black_box(Address { x: 0, y: 0 })))
    });
}

criterion_group!(benches, read_region_benchmark);
criterion_main!(benches);
