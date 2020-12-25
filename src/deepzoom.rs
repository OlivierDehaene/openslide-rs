//! This module provides functionality for generating Deep Zoom images from
//! OpenSlide slides.

use crate::openslide::{Address, OpenSlide, Region, Size};
use crate::{OpenSlideError, Result};
use image::imageops::{resize, FilterType};
use image::RgbaImage;

/// Support for Deep Zoom images.
pub struct DeepZoom<'a> {
    pub level_count: usize,
    pub level_tiles: Vec<Size>,
    pub level_dimensions: Vec<Size>,

    slide: &'a OpenSlide,
    tile_size: u32,
    overlap: u32,

    l0_offset: Address,
    slide_level_dimensions: Vec<Size>,
    slide_from_dz_level: Vec<usize>,
    l0_l_downsamples: Vec<f32>,
    l_z_downsamples: Vec<f32>,
}

impl<'a> DeepZoom<'a> {
    /// Create a DeepZoom wrapping an OpenSlide object.
    ///
    /// # Arguments
    ///
    /// * `slide` - a slide
    /// * `tile_size` - the width and height of a single tile.  For best viewer performance,
    /// tile_size + 2 * overlap should be a power of two.
    /// * `overlap` - the number of extra pixels to add to each interior edge of a tile.
    /// * `limit_bounds` - True to render only the non-empty slide region.
    pub fn new(
        slide: &'a OpenSlide,
        tile_size: u32,
        overlap: u32,
        limit_bounds: bool,
    ) -> Result<DeepZoom<'a>> {
        let mut slide_level_dimensions: Vec<Size> = Vec::new();
        let mut l0_offset = Address { x: 0, y: 0 };

        if limit_bounds {
            let bounds_x: u32 = match slide.property("openslide.bounds-x")? {
                Some(v) => v.parse::<u32>().unwrap(),
                None => 0,
            };

            let bounds_y: u32 = match slide.property("openslide.bounds-y")? {
                Some(v) => v.parse::<u32>().unwrap(),
                None => 0,
            };

            // Level 0 coordinate offset
            l0_offset.x = bounds_x;
            l0_offset.y = bounds_y;

            // Slide level dimensions scale factor in each axis
            let slide_dimensions = slide.dimensions().unwrap();

            let bounds_width: u32 = match slide.property("openslide.bounds-width")? {
                Some(v) => v.parse::<u32>().unwrap(),
                None => slide_dimensions.w as _,
            };

            let bounds_height: u32 = match slide.property("openslide.bounds-height")? {
                Some(v) => v.parse::<u32>().unwrap(),
                None => slide_dimensions.h as _,
            };

            let size_scale = (
                bounds_width as f32 / slide_dimensions.w as f32,
                bounds_height as f32 / slide_dimensions.h as f32,
            );

            slide_level_dimensions.extend(
                (0..slide.level_count().unwrap())
                    .map(|level| slide.level_dimensions(level).unwrap())
                    .map(|dimensions| Size {
                        w: (dimensions.w as f32 * size_scale.0).ceil() as _,
                        h: (dimensions.h as f32 * size_scale.1).ceil() as _,
                    }),
            );
        } else {
            slide_level_dimensions.extend(
                (0..slide.level_count().unwrap())
                    .map(|level| slide.level_dimensions(level).unwrap()),
            );
        }
        let slide_level0_dimensions = slide_level_dimensions[0];

        // Deep Zooom levels
        let mut z_size = Size {
            w: slide_level0_dimensions.w,
            h: slide_level0_dimensions.h,
        };
        let mut level_dimensions = vec![z_size];

        while z_size.w > 1 || z_size.h > 1 {
            z_size.w = ((z_size.w as f32 / 2.0).ceil() as u32).max(1) as _;
            z_size.h = ((z_size.h as f32 / 2.0).ceil() as u32).max(1) as _;

            level_dimensions.push(z_size);
        }
        level_dimensions.reverse();

        // Tile
        let level_tiles: Vec<Size> = level_dimensions
            .iter()
            .map(|Size { w, h }| Size {
                w: (*w as f32 / tile_size as f32).ceil() as _,
                h: (*h as f32 / tile_size as f32).ceil() as _,
            })
            .collect();

        // Deep Zoom level count
        let level_count = level_dimensions.len();

        // Total downsamples for each Deep Zoom level
        let l0_z_downsamples: Vec<f32> = (0..level_count)
            .map(|level| 2_u32.pow((level_count - level - 1) as _) as f32)
            .collect();

        // Preferred slide levels for each Deep Zoom level
        let slide_from_dz_level: Vec<usize> = l0_z_downsamples
            .iter()
            .map(|downsample| slide.best_level_for_downsample(*downsample).unwrap() as _)
            .collect();

        // Piecewise downsamples
        let l0_l_downsamples: Vec<f32> = (0..slide.level_count().unwrap())
            .map(|level| slide.level_downsample(level).unwrap())
            .collect();

        let l_z_downsamples: Vec<f32> = (0..level_count)
            .map(|dz_level| {
                l0_z_downsamples[dz_level] / l0_l_downsamples[slide_from_dz_level[dz_level]]
            })
            .collect();

        Ok(DeepZoom {
            slide,
            tile_size,
            overlap,
            l0_offset,
            level_dimensions,
            slide_level_dimensions,
            level_tiles,
            level_count,
            slide_from_dz_level,
            l0_l_downsamples,
            l_z_downsamples,
        })
    }

    fn tile_info(&self, level: usize, address: Address) -> Result<(Region, Size)> {
        if level >= self.level_count {
            return Err(OpenSlideError::InternalError(format!(
                "Level {} out of range",
                level
            )));
        }

        let level_tiles = self.level_tiles[level];
        let level_dimensions = self.level_dimensions[level];

        if address.x >= level_tiles.w || address.y > level_tiles.h {
            return Err(OpenSlideError::InternalError(format!(
                "Address {} out of range",
                address
            )));
        }

        // Get preferred slide level
        let slide_level = self.slide_from_dz_level[level];
        let slide_level_dimensions = self.slide_level_dimensions[slide_level];

        // Calculate top/left and bottom/right overlap
        let z_overlap_topleft = Address {
            x: if address.x != 0 { self.overlap } else { 0 },
            y: if address.y != 0 { self.overlap } else { 0 },
        };

        // Calculate top/left and bottom/right overlap
        let z_overlap_bottomright = Address {
            x: if address.x != (level_tiles.w - 1) {
                self.overlap
            } else {
                0
            },
            y: if address.y != (level_tiles.h - 1) {
                self.overlap
            } else {
                0
            },
        };

        // Get final size of the tile
        let z_size = Size {
            w: self
                .tile_size
                .min(level_dimensions.w - self.tile_size * address.x)
                + z_overlap_topleft.x
                + z_overlap_bottomright.x,
            h: self
                .tile_size
                .min(level_dimensions.h - self.tile_size * address.y)
                + z_overlap_topleft.y
                + z_overlap_bottomright.y,
        };

        // Obtain the region coordinates
        let z_location = Address {
            x: address.x * self.tile_size,
            y: address.y * self.tile_size,
        };

        let l_location = Address {
            x: (self.l_z_downsamples[level] * (z_location.x - z_overlap_topleft.x) as f32).ceil()
                as _,
            y: (self.l_z_downsamples[level] * (z_location.y - z_overlap_topleft.y) as f32).ceil()
                as _,
        };

        // Round location down and size up, and add offset of active area
        let l0_location = Address {
            x: (self.l0_l_downsamples[slide_level] * l_location.x as f32 + self.l0_offset.x as f32)
                as _,
            y: (self.l0_l_downsamples[slide_level] * l_location.y as f32 + self.l0_offset.y as f32)
                as _,
        };

        let l_size = Size {
            w: (slide_level_dimensions.w - l_location.x)
                .min((self.l_z_downsamples[level] * z_size.w as f32).ceil() as _),
            h: (slide_level_dimensions.h - l_location.y)
                .min((self.l_z_downsamples[level] * z_size.h as f32).ceil() as _),
        };

        let region = Region {
            address: l0_location,
            level: slide_level,
            size: l_size,
        };

        Ok((region, z_size))
    }

    /// Return the `openslide::Openslide::read_region` arguments for the specified tile.
    pub fn tile_region(&self, level: usize, address: Address) -> Result<Region> {
        let (region, _) = self.tile_info(level, address)?;
        Ok(region)
    }

    /// Return the tile final size for the specified tile
    pub fn tile_size(&self, level: usize, address: Address) -> Result<Size> {
        let (_, size) = self.tile_info(level, address)?;
        Ok(size)
    }

    /// Return a RGB tile
    pub fn read_tile(&self, level: usize, address: Address) -> Result<RgbaImage> {
        let (region, size) = self.tile_info(level, address)?;
        let mut tile = self.slide.read_region(region)?;

        if tile.dimensions() != (size.w, size.h) {
            tile = resize(&tile, size.w, size.h, FilterType::Lanczos3);
        }
        Ok(tile)
    }
}
