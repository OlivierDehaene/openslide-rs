fn probe(s: &str) -> pkg_config::Library {
    pkg_config::Config::new()
        .cargo_metadata(false)
        .probe(s)
        .unwrap()
}

fn link_library(s: &str) {
    pkg_config::Config::new().statik(true).probe(s).unwrap();
}

fn main() {
    let glib2 = probe("glib-2.0");
    let cairo = probe("cairo");
    let openjpeg = probe("libopenjp2");
    let xml = probe("libxml-2.0");
    let pixbuf = probe("gdk-pixbuf-2.0");

    cc::Build::new()
        .include("c-code")
        .includes(glib2.include_paths)
        .includes(cairo.include_paths)
        .includes(openjpeg.include_paths)
        .includes(xml.include_paths)
        .includes(pixbuf.include_paths)
        .file("c-code/openslide-cache.c")
        .file("c-code/openslide-decode-gdkpixbuf.c")
        .file("c-code/openslide-decode-jp2k.c")
        .file("c-code/openslide-decode-jpeg.c")
        .file("c-code/openslide-decode-png.c")
        .file("c-code/openslide-decode-sqlite.c")
        .file("c-code/openslide-decode-tiff.c")
        .file("c-code/openslide-decode-tifflike.c")
        .file("c-code/openslide-decode-xml.c")
        .file("c-code/openslide-error.c")
        .file("c-code/openslide-grid.c")
        .file("c-code/openslide-hash.c")
        .file("c-code/openslide-jdatasrc.c")
        .file("c-code/openslide-tables.c")
        .file("c-code/openslide-util.c")
        .file("c-code/openslide-vendor-aperio.c")
        .file("c-code/openslide-vendor-generic-tiff.c")
        .file("c-code/openslide-vendor-hamamatsu.c")
        .file("c-code/openslide-vendor-leica.c")
        .file("c-code/openslide-vendor-mirax.c")
        .file("c-code/openslide-vendor-philips.c")
        .file("c-code/openslide-vendor-sakura.c")
        .file("c-code/openslide-vendor-trestle.c")
        .file("c-code/openslide-vendor-ventana.c")
        .file("c-code/openslide.c")
        .compile("libopenslide.a");

    link_library("gdk-pixbuf-2.0");
    link_library("cairo");
    link_library("libopenjp2");
    link_library("libxml-2.0");
    link_library("libpng16");
    link_library("libtiff-4");
    link_library("libjpeg");
    link_library("sqlite3");
    link_library("gio-2.0");
    link_library("glib-2.0");
}
