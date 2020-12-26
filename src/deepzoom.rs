use math::round;
use std::cmp;
use std::path::Path;

use crate::openslide::{Address, OpenSlide, Region, Size};
use image::imageops::thumbnail;
use image::RgbaImage;

pub struct DeepZoom<'a> {
    slide: &'a OpenSlide,
    tile_size: u32,
    overlap: u32,
    limit_bounds: bool,

    l0_offset: Address,
    slide_level_dimensions: Vec<Size>,
    slide_level0_dimensions: Size,
    level_dimensions: Vec<Size>,
    level_tiles: Vec<Size>,
    level_count: usize,
    slide_from_dz_level: Vec<usize>,
    l0_l_downsamples: Vec<f64>,
    l_z_downsamples: Vec<f64>,
    bg_color: u32,
}

impl<'a> DeepZoom<'a> {
    pub fn new(
        slide: &'a OpenSlide,
        tile_size: u32,
        overlap: u32,
        limit_bounds: bool,
    ) -> DeepZoom<'a> {
        let mut slide_level_dimensions: Vec<Size> = Vec::new();
        let mut l0_offset = Address { x: 0, y: 0 };

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
            l0_offset.x = bounds_x;
            l0_offset.y = bounds_y;

            // Slide level dimensions scale factor in each axis
            let slide_dimensions = slide.dimensions().unwrap();

            let bounds_width: u32 = match slide.properties.get("openslide.bounds-width") {
                Some(v) => v.parse::<u32>().unwrap(),
                None => slide_dimensions.w as _,
            };

            let bounds_height: u32 = match slide.properties.get("openslide.bounds-height") {
                Some(v) => v.parse::<u32>().unwrap(),
                None => slide_dimensions.h as _,
            };

            let size_scale = (
                bounds_width as f64 / slide_dimensions.w as f64,
                bounds_height as f64 / slide_dimensions.h as f64,
            );

            &slide_level_dimensions.extend(
                (0..slide.level_count().unwrap())
                    .map(|level| slide.level_dimensions(level).unwrap())
                    .map(|dimensions| Size {
                        w: round::ceil(dimensions.w as f64 * size_scale.0, 0) as _,
                        h: round::ceil(dimensions.h as f64 * size_scale.1, 0) as _,
                    }),
            );
        } else {
            &slide_level_dimensions.extend(
                (0..slide.level_count().unwrap())
                    .map(|level| slide.level_dimensions(level).unwrap()),
            );
        }
        let slide_level0_dimensions = slide_level_dimensions[0].clone();

        // Deep Zooom levels
        let mut z_size = Size {
            w: slide_level0_dimensions.w,
            h: slide_level0_dimensions.h,
        };
        let mut level_dimensions = vec![z_size.clone()];

        while z_size.w > 1 || z_size.h > 1 {
            z_size.w = cmp::max(1, round::ceil(z_size.w as f64 / 2.0, 0) as _) as _;
            z_size.h = cmp::max(1, round::ceil(z_size.h as f64 / 2.0, 0) as _) as _;

            level_dimensions.push(z_size.clone());
        }
        level_dimensions.reverse();

        // Tile
        let level_tiles: Vec<Size> = level_dimensions
            .iter()
            .map(|Size { w, h }| Size {
                w: round::ceil(*w as f64 / tile_size as f64, 0) as _,
                h: round::ceil(*w as f64 / tile_size as f64, 0) as _,
            })
            .collect();

        // Deep Zoom level count
        let level_count = level_dimensions.len() as usize;

        // Total downsamples for each Deep Zoom level
        let l0_z_downsamples: Vec<f64> = (0..level_count)
            .map(|level| 2_u32.pow((level_count - level - 1) as _) as f64)
            .collect();

        // Preferred slide levels for each Deep Zoom level
        let slide_from_dz_level: Vec<usize> = l0_z_downsamples
            .iter()
            .map(|downsample| slide.best_level_for_downsample(*downsample).unwrap() as _)
            .collect();

        // Piecewise downsamples
        let l0_l_downsamples: Vec<f64> = (0..slide.level_count().unwrap())
            .map(|level| slide.level_downsample(level).unwrap())
            .collect();

        let l_z_downsamples: Vec<f64> = (0..level_count)
            .map(|dz_level| {
                l0_z_downsamples[dz_level] / l0_l_downsamples[slide_from_dz_level[dz_level]]
            })
            .collect();

        // Background color
        // TODO: parse from slide properties
        let bg_color: u32 = 255;

        DeepZoom {
            slide,
            tile_size,
            overlap,
            limit_bounds,
            l0_offset,
            level_dimensions,
            slide_level0_dimensions,
            slide_level_dimensions,
            level_tiles,
            level_count,
            slide_from_dz_level,
            l0_l_downsamples,
            l_z_downsamples,
            bg_color,
        }
    }

    fn tile_info(&self, level: u32, address: Address) -> Result<(Region, Size), String> {
        if level as usize >= self.level_count {
            return Err(format!("Level {} out of range", level));
        }

        let level_dimensions = self.level_dimensions[level as usize];

        if address.x >= level_dimensions.w || address.y > level_dimensions.h {
            return Err(format!("Address {} out of range", address));
        }

        // Get preferred slide level
        let slide_level = self.slide_from_dz_level[level as usize];
        let slide_level_dimensions = self.slide.level_dimensions(slide_level as _)?;

        // Calculate top/left and bottom/right overlap
        let z_overlap_topleft = Address {
            x: if address.x != 0 { self.overlap } else { 0 },
            y: if address.y != 0 { self.overlap } else { 0 },
        };

        // Calculate top/left and bottom/right overlap
        let z_overlap_bottomright = Address {
            x: if address.x != (level_dimensions.w - 1) {
                self.overlap
            } else {
                0
            },
            y: if address.y != (level_dimensions.h - 1) {
                self.overlap
            } else {
                0
            },
        };

        // Get final size of the tile
        let z_size = Size {
            w: cmp::min(
                self.tile_size,
                level_dimensions.w - self.tile_size * address.x,
            ) + z_overlap_topleft.x
                + z_overlap_bottomright.x,
            h: cmp::min(
                self.tile_size,
                level_dimensions.h - self.tile_size * address.y,
            ) + z_overlap_topleft.y
                + z_overlap_bottomright.y,
        };

        // Obtain the region coordinates
        let z_location = Address {
            x: address.x * self.tile_size,
            y: address.y * self.tile_size,
        };

        let l_location = Address {
            x: round::ceil(
                self.l_z_downsamples[level as usize] * (z_location.x - z_overlap_topleft.x) as f64,
                0,
            ) as _,
            y: round::ceil(
                self.l_z_downsamples[level as usize] * (z_location.y - z_overlap_topleft.y) as f64,
                0,
            ) as _,
        };

        // Round location down and size up, and add offset of active area
        let l0_location = Address {
            x: (self.l0_l_downsamples[slide_level] * l_location.x as f64 + self.l0_offset.x as f64)
                as _,
            y: (self.l0_l_downsamples[slide_level] * l_location.y as f64 + self.l0_offset.y as f64)
                as _,
        };

        let l_size = Size {
            w: cmp::min(
                round::ceil(self.l_z_downsamples[level as usize] * z_size.w as f64, 0) as _,
                slide_level_dimensions.w - l_location.x,
            ),
            h: cmp::min(
                round::ceil(self.l_z_downsamples[level as usize] * z_size.h as f64, 0) as _,
                slide_level_dimensions.h - l_location.y,
            ),
        };

        let region = Region {
            address: l0_location,
            level: slide_level,
            size: l_size,
        };

        Ok((region, z_size))
    }

    pub fn tile_region(&self, level: u32, address: Address) -> Result<Region, String> {
        let (region, _) = self.tile_info(level, address)?;
        Ok(region)
    }

    pub fn tile_size(&self, level: u32, address: Address) -> Result<Size, String> {
        let (_, size) = self.tile_info(level, address)?;
        Ok(size)
    }

    pub fn read_tile(&self, level: u32, address: Address) -> Result<RgbaImage, String> {
        let (region, size) = self.tile_info(level, address)?;
        let mut tile = self.slide.read_region(region)?;

        if tile.dimensions() != (size.w, size.h) {
            tile = thumbnail(&tile, size.w, size.h);
        }
        Ok(tile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open() {
        let slide = OpenSlide::open(Path::new("tests/assets/default.svs")).unwrap();
        let dz = DeepZoom::new(&slide, 224, 0, false);

        let tile = dz.read_tile(9, Address { x: 0, y: 0 }).unwrap();
    }
}
