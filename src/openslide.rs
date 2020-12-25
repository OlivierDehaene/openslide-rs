use std::cmp::Ordering;

use std::ffi::{CStr, CString};
use std::fmt;
use std::path::Path;
use std::str;

use image::imageops::{resize, FilterType};
use image::RgbaImage;
use openslide_sys as sys;
use std::ptr::null_mut;

use crate::utils::{decode_buffer, parse_null_terminated_array, resize_dimensions};
use crate::{OpenSlideError, Result};

/// A basic x/y type
#[derive(Debug, PartialEq)]
pub struct Address {
    /// x coordinate
    pub x: u32,
    /// y coordinate
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

/// A basic width/height type.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Size {
    /// Height
    pub h: u32,
    /// Width
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

/// The coordinates of a region of a whole slide image.
#[derive(Debug, PartialEq)]
pub struct Region {
    /// The top left coordinates
    pub address: Address,
    /// The whole slide image level
    pub level: usize,
    /// The size of the region
    pub size: Size,
}

/// The main OpenSlide type.
pub struct OpenSlide {
    data: *mut sys::_openslide,
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

/// # Examples
///
/// ```
/// use std::path::Path;
/// use openslide_rs::{OpenSlide, OpenSlideError};
///
/// fn main() -> Result<(), OpenSlideError> {
///     let filename = Path::new("tests/assets/default.svs");
///     let os = OpenSlide::open(&filename)?;
///     let num_levels = os.level_count()?;
///     println!("Slide has {} levels", num_levels);
///
///     Ok(())
/// }
/// ```
impl OpenSlide {
    /// Quickly determine whether a whole slide image is recognized.
    ///
    /// Returns a string identifying the slide format vendor. This is equivalent
    /// to the value of the #OPENSLIDE_PROPERTY_NAME_VENDOR property.
    ///
    /// # Arguments
    ///
    /// * `path`: path to a valid whole slide image.
    ///
    /// # Errors
    ///
    /// * [`OpenSlideError::MissingFile`](enum.OpenSlideError.html#variant.MissingFile): the file does not exist
    /// * [`OpenSlideError::UnsupportedFile`](enum.OpenSlideError.html#variant.UnsupportedFile): the file is not a valid whole slide image.
    /// * [`OpenSlideError::InternalError`](enum.OpenSlideError.html#variant.InternalError): an error occured in the C codebase.
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

    /// Open a whole slide image.
    ///
    /// # Arguments
    ///
    /// * `path`: path to a valid whole slide image.
    ///
    /// # Errors
    ///
    /// * [`OpenSlideError::MissingFile`](enum.OpenSlideError.html#variant.MissingFile): the file does not exist
    /// * [`OpenSlideError::UnsupportedFile`](enum.OpenSlideError.html#variant.UnsupportedFile): the file is not a valid whole slide image.
    /// * [`OpenSlideError::InternalError`](enum.OpenSlideError.html#variant.InternalError): an error occured in the C codebase.
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

        let slide = OpenSlide { data: slide_ptr };

        Ok(slide)
    }

    /// Set the cache size of the whole slide image
    ///
    /// # Arguments
    ///
    /// * `cache_size`: cache size in bytes
    ///
    /// # Errors
    ///
    /// * [`OpenSlideError::InternalError`](enum.OpenSlideError.html#variant.InternalError): an error occured in the C codebase.
    pub fn set_cache_size(&mut self, cache_size: u32) -> Result<()> {
        unsafe {
            let cache = sys::openslide_cache_create(cache_size as _);
            sys::openslide_set_cache(self.data, cache);
        }
        get_error(self.data)
    }

    /// Get the number of levels in the whole slide image.
    ///
    /// # Errors
    ///
    /// * [`OpenSlideError::InternalError`](enum.OpenSlideError.html#variant.InternalError): an error occured in the C codebase.
    pub fn level_count(&self) -> Result<u32> {
        let level_count = unsafe { sys::openslide_get_level_count(self.data) as u32 };
        get_error(self.data)?;

        Ok(level_count)
    }

    /// Get the dimensions of level 0 (the largest level). Exactly equivalent
    /// to calling [`level_dimensions(0)`](struct.OpenSlide.html#method.level_dimensions).
    ///
    /// # Arguments
    ///
    /// * `level`: The desired level.
    ///
    /// # Errors
    ///
    /// * [`OpenSlideError::IndexError`](enum.OpenSlideError.html#variant.IndexError): level out of range
    /// * [`OpenSlideError::InternalError`](enum.OpenSlideError.html#variant.InternalError): an error occured in the C codebase.
    pub fn dimensions(&self) -> Result<Size> {
        self.level_dimensions(0)
    }

    /// Get the dimensions of a level.
    ///
    /// # Arguments
    ///
    /// * `level`: The desired level.
    ///
    /// # Errors
    ///
    /// * [`OpenSlideError::IndexError`](enum.OpenSlideError.html#variant.IndexError): level out of range
    /// * [`OpenSlideError::InternalError`](enum.OpenSlideError.html#variant.InternalError): an error occured in the C codebase.
    pub fn level_dimensions(&self, level: u32) -> Result<Size> {
        if level >= self.level_count()? {
            return Err(OpenSlideError::IndexError(level.to_string()));
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

    /// Get the downsampling factor of a given level.Address
    ///
    /// # Arguments
    ///
    /// * `level`: The desired level.
    ///
    /// # Errors
    ///
    /// * [`OpenSlideError::IndexError`](enum.OpenSlideError.html#variant.IndexError): level out of range
    /// * [`OpenSlideError::InternalError`](enum.OpenSlideError.html#variant.InternalError): an error occured in the C codebase.
    pub fn level_downsample(&self, level: u32) -> Result<f32> {
        if level >= self.level_count()? {
            return Err(OpenSlideError::IndexError(level.to_string()));
        }

        let level_downsample =
            unsafe { sys::openslide_get_level_downsample(self.data, level as _) };
        get_error(self.data)?;

        Ok(level_downsample as _)
    }

    /// Get the best level to use for displaying the given downsample.
    ///
    /// # Arguments
    ///
    /// * `downsample`: The downsample factor.
    ///
    /// # Errors
    ///
    /// * [`OpenSlideError::InternalError`](enum.OpenSlideError.html#variant.InternalError): an error occured in the C codebase.
    pub fn best_level_for_downsample(&self, downsample: f32) -> Result<u32> {
        let best_level =
            unsafe { sys::openslide_get_best_level_for_downsample(self.data, downsample as _) };
        get_error(self.data)?;

        Ok(best_level as _)
    }

    /// This function reads and decompresses a region of a whole slide image into
    /// a `RgbaImage`.
    ///
    /// # Arguments
    ///
    /// * `region`: the coordinates of the region to read.
    ///
    /// # Errors
    ///
    /// * [`OpenSlideError::InternalError`](enum.OpenSlideError.html#variant.InternalError): an error occured in the C codebase.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use openslide_rs::{OpenSlide, OpenSlideError, Region, Address, Size};
    ///
    /// fn main() -> Result<(), OpenSlideError> {
    ///     let path = Path::new("tests/assets/default.svs");
    ///     let slide = OpenSlide::open(&path)?;
    ///
    ///     let region = slide
    ///        .read_region(Region {
    ///            address: Address { x: 512, y: 512 },
    ///            level: 0,
    ///            size: Size { w: 512, h: 512 },
    ///        })
    ///        .unwrap();
    ///     region.save(Path::new("tests/artifacts/example_read_region.png")).unwrap();
    ///
    ///     Ok(())
    ///  }
    /// ```
    ///
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

        Ok(decode_buffer(&dest, size.w, size.h))
    }

    /// Get the property names vector.Address
    ///
    /// Certain vendor-specific metadata properties may exist within
    /// a whole slide image. They are encoded as key-value pairs. This call provides
    /// a vector of names as strings that can be used to read properties with
    /// [`property()`](struct.OpenSlide.html#method.property).
    ///
    /// # Errors
    ///
    /// * [`OpenSlideError::InternalError`](enum.OpenSlideError.html#variant.InternalError): an error occured in the C codebase.
    pub fn property_names(&self) -> Result<Vec<String>> {
        unsafe {
            let name_array = sys::openslide_get_property_names(self.data);
            get_error(self.data)?;

            Ok(parse_null_terminated_array(name_array).collect())
        }
    }

    /// Get the value of a single property.Address
    ///
    /// Certain vendor-specific metadata properties may exist within a
    /// whole slide image. They are encoded as key-value paris. This call
    /// provides the value of the property given by `name`.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the desired property. Must be a valid name
    /// as given by [`property_names()`](struct.OpenSlide.html#method.property_names).
    ///
    /// # Errors
    ///
    /// * [`OpenSlideError::InternalError`](enum.OpenSlideError.html#variant.InternalError): an error occured in the C codebase.
    pub fn property(&self, name: &str) -> Result<Option<String>> {
        if !self.property_names()?.iter().any(|n| n == name) {
            return Ok(None);
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

        Ok(value)
    }

    /// Get the associated image names vector.
    ///
    /// Certain vendor-specific associated images may exist within a whole slide image. They are
    /// encoded as key-value pairs. This call provides a vector of names as strings that can be used
    /// to read associated images with [`associated_image()`](struct.OpenSlide.html#method.associated_image).
    ///
    /// # Errors
    ///
    /// * [`OpenSlideError::InternalError`](enum.OpenSlideError.html#variant.InternalError): an error occured in the C codebase.
    pub fn associated_image_names(&self) -> Result<Vec<String>> {
        unsafe {
            let name_array = sys::openslide_get_associated_image_names(self.data);
            get_error(self.data)?;

            Ok(parse_null_terminated_array(name_array).collect())
        }
    }

    /// Reads and decompresses an associated image associated with a whole slide image.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the desired associated image. Must be a valid name
    /// as given by [`associated_image_names()`](struct.OpenSlide.html#method.associated_image_names).
    ///
    /// # Errors
    ///
    /// * [`OpenSlideError::InternalError`](enum.OpenSlideError.html#variant.InternalError): an error occured in the C codebase.
    pub fn associated_image(&self, name: &str) -> Result<Option<RgbaImage>> {
        if !self.associated_image_names()?.iter().any(|n| n == name) {
            return Ok(None);
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

        Ok(Some(decode_buffer(&dest, w as _, h as _)))
    }

    pub fn thumbnail(&self, size: Size) -> Result<RgbaImage> {
        let dimensions = self.dimensions()?;
        let downsample_w = dimensions.w as f32 / size.w as f32;
        let downsample_h = dimensions.h as f32 / size.h as f32;

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
        Ok(resize(&tile, new_width, new_height, FilterType::Lanczos3))
    }
}

/// Get the current error string.
///
/// # Errors
///
/// * [`OpenSlideError::InternalError`](enum.OpenSlideError.html#variant.InternalError): an error occured in the C codebase.
fn get_error(slide_ptr: *mut sys::_openslide) -> Result<()> {
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
