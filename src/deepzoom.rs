use std::path::Path;

use crate::openslide::{Address, OpenSlide, Size};

pub struct DeepZoom<'a> {
    slide: &'a OpenSlide,
    tile_size: u32,
    overlap: u32,
    limit_bounds: bool,
}

impl<'a> DeepZoom<'a> {
    pub fn new(
        slide: &'a OpenSlide,
        tile_size: u32,
        overlap: u32,
        limit_bounds: bool,
    ) -> DeepZoom<'a> {
        if limit_bounds {
            let bounds_x: u32 = match slide.properties.get("openslide.bounds-x") {
                Some(v) => v.parse::<u32>().unwrap(),
                None => 0,
            };

            let bounds_y: u32 = match slide.properties.get("openslide.bounds-y") {
                Some(v) => v.parse::<u32>().unwrap(),
                None => 0,
            };

            // Level 0 coordinate offset
            let l0_offset = (bounds_x, bounds_y);

            // Slide level dimensions scale factor in each axis
            let slide_dimensions = slide.dimensions().unwrap();

            let bounds_width: u32 = match slide.properties.get("openslide.bounds-width") {
                Some(v) => v.parse::<u32>().unwrap(),
                None => slide_dimensions.0 as _,
            };

            let bounds_height: u32 = match slide.properties.get("openslide.bounds-height") {
                Some(v) => v.parse::<u32>().unwrap(),
                None => slide_dimensions.1 as _,
            };

            let size_scale = (
                bounds_width / slide_dimensions.0 as u32,
                bounds_height / slide_dimensions.1 as u32,
            );
            //TODO FINISH
        }

        DeepZoom {
            slide: slide,
            tile_size: tile_size,
            overlap: overlap,
            limit_bounds: limit_bounds,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open() {
        let slide = OpenSlide::open(Path::new("tests/assets/default.svs")).unwrap();
        let dz = DeepZoom::new(&slide, 224, 0, false);
    }
}
