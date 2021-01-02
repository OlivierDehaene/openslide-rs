use std::cmp::Ordering;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fmt;
use std::path::Path;
use std::str;

use image::imageops::thumbnail;
use image::RgbaImage;
use openslide_sys as sys;
use std::ptr::null_mut;

use crate::utils::{
    decode_buffer, parse_null_terminated_array, resize_dimensions, WordRepresentation,
};
use crate::{OpenSlideError, Result};

#[derive(Debug, PartialEq)]
pub struct Address {
    pub x: u32,
    pub y: u32,
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl<T> From<(T, T)> for Address
    where
        T: Clone + Into<u32>,
{
    fn from(address: (T, T)) -> Self {
        Address {
            x: address.0.into(),
            y: address.1.into(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Size {
    pub h: u32,
    pub w: u32,
}

impl<T> From<(T, T)> for Size
    where
        T: Clone + Into<u32>,
{
    fn from(size: (T, T)) -> Self {
        Size {
            w: size.0.into(),
            h: size.1.into(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Region {
    pub address: Address,
    pub level: usize,
    pub size: Size,
}

pub struct OpenSlide {
    data: *mut sys::OpenSlide,
}

unsafe impl Send for OpenSlide {}

impl Drop for OpenSlide {
    fn drop(&mut self) {
        unsafe {
            sys::openslide_close(self.data);
        }
        self.data = null_mut();
    }
}

impl OpenSlide {
    pub fn detect_vendor(path: &Path) -> Result<String> {
        if !path.exists() {
            return Err(OpenSlideError::MissingFile(path.display().to_string()));
        }

        let cstr = CString::new(path.to_str().unwrap()).unwrap();
        unsafe {
            let slice = sys::openslide_detect_vendor(cstr.as_ptr());

            if slice.is_null() {
                Err(OpenSlideError::UnsupportedFile(path.display().to_string()))
            } else {
                Ok(CStr::from_ptr(slice).to_string_lossy().into_owned())
            }
        }
    }

    pub fn open(path: &Path) -> Result<OpenSlide> {
        if !path.exists() {
            return Err(OpenSlideError::MissingFile(path.display().to_string()));
        }

        let path_cstr = CString::new(path.to_str().unwrap()).unwrap();
        let slide_ptr = unsafe { sys::openslide_open(path_cstr.as_ptr()) };

        if slide_ptr.is_null() {
            return Err(OpenSlideError::UnsupportedFile(path.display().to_string()));
        }
        get_error(slide_ptr)?;

        let slide = OpenSlide {
            data: slide_ptr,
        };

        Ok(slide)
    }

    pub fn set_cache_size(&self, cache_size: i32) {
        unsafe {
            sys::openslide_set_cache_size(self.data, cache_size);
        }
    }

    /// The number of levels in the image.
    pub fn level_count(&self) -> Result<u32> {
        let level_count = unsafe { sys::openslide_get_level_count(self.data) as u32 };
        get_error(self.data)?;

        Ok(level_count)
    }

    pub fn dimensions(&self) -> Result<Size> {
        self.level_dimensions(0)
    }

    pub fn level_dimensions(&self, level: u32) -> Result<Size> {
        if level >= self.level_count()? {
            return Err(OpenSlideError::InternalError(format!(
                "Level {} out of range",
                level
            )));
        }

        let mut w = 0;
        let mut h = 0;
        unsafe {
            sys::openslide_get_level_dimensions(self.data, level as _, &mut w, &mut h);
        }

        get_error(self.data)?;

        Ok(Size {
            w: w as _,
            h: h as _,
        })
    }

    pub fn level_downsample(&self, level: u32) -> Result<f64> {
        if level >= self.level_count()? {
            return Err(OpenSlideError::InternalError(format!(
                "Level {} out of range",
                level
            )));
        }

        let level_downsample =
            unsafe { sys::openslide_get_level_downsample(self.data, level as _) };
        get_error(self.data)?;

        Ok(level_downsample)
    }

    pub fn best_level_for_downsample(&self, downsample: f64) -> Result<u32> {
        let best_level =
            unsafe { sys::openslide_get_best_level_for_downsample(self.data, downsample) };
        get_error(self.data)?;

        Ok(best_level as _)
    }

    pub fn read_region(&self, region: Region) -> Result<RgbaImage> {
        let Region {
            address,
            level,
            size,
        } = region;

        let mut dest = vec![0u32; (size.w * size.h) as _];

        unsafe {
            openslide_sys::openslide_read_region(
                self.data,
                dest.as_mut_ptr(),
                address.x as _,
                address.y as _,
                level as _,
                size.w as _,
                size.h as _,
            )
        }
        get_error(self.data)?;

        Ok(decode_buffer(
            &dest,
            size.w,
            size.h,
            WordRepresentation::BigEndian,
        ))
    }

    pub fn property_names(&self) -> Result<Vec<String>> {
        unsafe {
            let name_array = sys::openslide_get_property_names(self.data);
            get_error(self.data)?;

            Ok(parse_null_terminated_array(name_array).collect())
        }
    }

    pub fn property(&self, name: &str) -> Result<String> {
        if !self.property_names()?.iter().any(|n| n == name) {
            return Err(OpenSlideError::KeyError(name.into()
            ));
        };

        let cstr = CString::new(name).unwrap();
        let value = unsafe {
            let slice = sys::openslide_get_property_value(self.data, cstr.as_ptr());

            if slice.is_null() {
                None
            } else {
                Some(CStr::from_ptr(slice).to_string_lossy().into_owned())
            }
        };
        get_error(self.data)?;

        match value {
            None => Err(OpenSlideError::KeyError(name.into())),
            Some(value) => Ok(value),
        }
    }


    pub fn associated_image_names(&self) -> Result<Vec<String>> {
        unsafe {
            let name_array = sys::openslide_get_associated_image_names(self.data);
            get_error(self.data)?;

            Ok(parse_null_terminated_array(name_array).collect())
        }
    }

    pub fn associated_image(&self, name: &str) -> Result<RgbaImage> {
        if !self.associated_image_names()?.iter().any(|n| n == name) {
            return Err(OpenSlideError::KeyError(name.into()
            ));
        };

        let cstr = CString::new(name).unwrap();

        let mut w = 0;
        let mut h = 0;
        unsafe {
            sys::openslide_get_associated_image_dimensions(
                self.data,
                cstr.as_ptr(),
                &mut w,
                &mut h,
            );
        }

        get_error(self.data)?;

        let mut dest = vec![0u32; (w * h) as _];

        unsafe {
            sys::openslide_read_associated_image(self.data, cstr.as_ptr(), dest.as_mut_ptr());
        }
        get_error(self.data)?;

        Ok(decode_buffer(
            &dest,
            w as _,
            h as _,
            WordRepresentation::BigEndian,
        ))
    }

    pub fn thumbnail(&self, size: Size) -> Result<RgbaImage> {
        let dimensions = self.dimensions()?;
        let downsample_w = dimensions.w as f64 / size.w as f64;
        let downsample_h = dimensions.h as f64 / size.h as f64;

        let max_downsample = match downsample_w.partial_cmp(&downsample_h).unwrap() {
            Ordering::Less => downsample_h,
            Ordering::Equal => downsample_w,
            Ordering::Greater => downsample_w,
        };

        let level = self.best_level_for_downsample(max_downsample)?;

        let tile = self.read_region(Region {
            address: Address { x: 0, y: 0 },
            level: level as _,
            size: self.level_dimensions(level)?,
        })?;
        let (new_width, new_height) =
            resize_dimensions(tile.width(), tile.height(), size.w, size.h, false);
        Ok(thumbnail(&tile, new_width, new_height))
    }
}

fn get_error(slide_ptr: *mut sys::OpenSlide) -> Result<()> {
    unsafe {
        let slice = sys::openslide_get_error(slide_ptr);

        if slice.is_null() {
            Ok(())
        } else {
            Err(OpenSlideError::InternalError(
                CStr::from_ptr(slice).to_string_lossy().into_owned(),
            ))
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Unsupported TIFF compression: 52479")]
    fn test_get_error() {
        let path_cstr = CString::new("tests/assets/unopenable.tiff").unwrap();
        let slide_ptr = unsafe { sys::openslide_open(path_cstr.as_ptr()) };

        get_error(slide_ptr).unwrap();
    }
}
