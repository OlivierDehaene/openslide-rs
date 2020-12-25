# OpenSlide Py

Python bindings to `openslide-rs` Rust code. Exposes the same interface and very 
close to the [original Python bindings](https://github.com/openslide/openslide-python).

## Install

```bash
pip install .
```

### M1 Macs

```bash
pip install maturin
maturin develop
```

## Test

```bash
tox tests
```