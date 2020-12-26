use std::path::Path;

pub fn missing_file() -> &'static Path {
    Path::new("__missing")
}

pub fn unsupported_file() -> &'static Path {
    Path::new("Cargo.toml")
}

pub fn boxes_tiff() -> &'static Path {
    Path::new("tests/assets/boxes.tiff")
}

pub fn unopenable_tiff() -> &'static Path {
    Path::new("tests/assets/unopenable.tiff")
}

pub fn small_svs() -> &'static Path {
    Path::new("tests/assets/small.svs")
}

pub fn unreadable_svs() -> &'static Path {
    Path::new("tests/assets/unreadable.svs")
}
