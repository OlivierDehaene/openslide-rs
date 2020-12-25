# OpenSlide RS

Rust bindings and wrapper to the original OpenSlide C code.

This work has no affiliations with the official OpenSlide project.

## Example

 ```rust
 use std::path::Path;
 use openslide_rs::{OpenSlide, OpenSlideError, Region, Address, Size};

 fn main() -> Result<(), OpenSlideError> {
     let path = Path::new("tests/assets/default.svs");
     let slide = OpenSlide::open(&path)?;
     
     let region = slide
        .read_region(Region {
            address: Address { x: 512, y: 512 },
            level: 0,
            size: Size { w: 512, h: 512 },
        })
        .unwrap();
     region.save(Path::new("tests/artifacts/example_read_region.png")).unwrap();
         
     Ok(())
 }
 ```

## Install

### Linux

```bash
make install-apt
make build
```

### Mac

```bash
make install-brew
make build
```

## Test

```bash
make test
```

## Benchmark reads

```bash
make bench
```