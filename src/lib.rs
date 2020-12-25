use std::collections::HashMap;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::path::Path;
use std::str;

use byteorder::ByteOrder;
use image::{Rgba, RgbaImage};
use openslide_sys as sys;
use std::ptr::null_mut;

pub struct Address {
    x: u32,
    y: u32,
}

impl Address {
    pub fn new(x: u32, y: u32) -> Address {
        Address { x: x, y: y }
    }
}

pub struct Size {
    h: u32,
    w: u32,
}

impl Size {
    pub fn new(w: u32, h: u32) -> Size {
        Size { w: w, h: h }
    }
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
    pub fn detect_vendor(path: &Path) -> Option<String> {
        let cstr = CString::new(path.to_str().unwrap()).unwrap();
        unsafe {
            let slice = sys::openslide_detect_vendor(cstr.as_ptr());

            if slice.is_null() {
                None
            } else {
                Some(CStr::from_ptr(slice).to_string_lossy().into_owned())
            }
        }
    }

    fn get_error(&self) -> Result<(), String> {
        unsafe {
            let slice = sys::openslide_get_error(self.data);

            if slice.is_null() {
                Ok(())
            } else {
                Err(CStr::from_ptr(slice).to_string_lossy().into_owned())
            }
        }
    }

    pub fn open(path: &Path) -> Result<OpenSlide, String> {
        if !path.exists() {
            return Err(String::from(format!(
                "Missing image file: {}",
                path.display()
            )));
        }

        let cstr = CString::new(path.to_str().unwrap()).unwrap();
        let res = unsafe { sys::openslide_open(cstr.as_ptr()) };

        if res.is_null() {
            return Err(String::from("Unsupported image file"));
        }

        let slide = OpenSlide { data: res };
        slide.get_error()?;

        Ok(slide)
    }

    /// The number of levels in the image.
    pub fn level_count(&self) -> Result<u32, String> {
        let level_count = unsafe { sys::openslide_get_level_count(self.data) as u32 };
        self.get_error()?;

        Ok(level_count)
    }

    pub fn level_dimensions(&self, level: u32) -> Result<(u64, u64), String> {
        if level >= self.level_count()? {
            return Err(format!("Level {} out of range", level));
        }

        let mut w = 0;
        let mut h = 0;
        unsafe {
            sys::openslide_get_level_dimensions(self.data, level as _, &mut w, &mut h);
        }

        self.get_error()?;

        Ok((w as _, h as _))
    }

    pub fn level_downsample(&self, level: u32) -> Result<f64, String> {
        if level >= self.level_count()? {
            return Err(format!("Level {} out of range", level));
        }

        let level_downsample =
            unsafe { sys::openslide_get_level_downsample(self.data, level as _) };
        self.get_error()?;

        Ok(level_downsample)
    }

    pub fn best_level_for_downsample(&self, downsample: f64) -> Result<u32, String> {
        let best_level =
            unsafe { sys::openslide_get_best_level_for_downsample(self.data, downsample) };
        self.get_error()?;

        Ok(best_level as _)
    }

    pub fn read_region(
        &self,
        address: Address,
        level: u32,
        size: Size,
    ) -> Result<RgbaImage, String> {
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
        self.get_error()?;

        Ok(decode_buffer(
            &dest,
            size.w,
            size.h,
            WordRepresentation::BigEndian,
        ))
    }

    fn property(&self, name: &str) -> Result<String, String> {
        let cstr = CString::new(name).unwrap();
        let value = unsafe {
            let slice = sys::openslide_get_property_value(self.data, cstr.as_ptr());

            if slice.is_null() {
                None
            } else {
                Some(CStr::from_ptr(slice).to_string_lossy().into_owned())
            }
        };
        self.get_error()?;

        match value {
            None => Err(format!("Property {} doesn't exist.", name)),
            Some(value) => Ok(value),
        }
    }

    pub fn properties(&self) -> Result<HashMap<String, String>, String> {
        unsafe {
            let name_array = sys::openslide_get_property_names(self.data);
            self.get_error()?;

            let mut properties = HashMap::new();

            for name in parse_null_terminated_array(name_array) {
                let value = self.property(&name)?;
                properties.insert(name, value);
            }
            Ok(properties)
        }
    }

    pub fn associated_image_names(&self) -> Result<Vec<String>, String> {
        unsafe {
            let name_array = sys::openslide_get_associated_image_names(self.data);
            self.get_error()?;

            Ok(parse_null_terminated_array(name_array).collect())
        }
    }

    pub fn associated_image(&self, name: &str) -> Result<RgbaImage, String> {
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

        self.get_error()?;

        let mut dest = vec![0u32; (w * h) as _];

        unsafe {
            sys::openslide_read_associated_image(self.data, cstr.as_ptr(), dest.as_mut_ptr());
        }
        self.get_error()?;

        Ok(decode_buffer(
            &dest,
            w as _,
            h as _,
            WordRepresentation::BigEndian,
        ))
    }
}

fn parse_null_terminated_array(array: *const *const i8) -> impl Iterator<Item = String> {
    unsafe {
        let mut counter = 0;
        let mut loc = array;
        while *loc != std::ptr::null() {
            counter += 1;
            loc = loc.offset(1);
        }
        let parts = std::slice::from_raw_parts(array, counter as usize);

        parts
            .iter()
            .map(|&p| CStr::from_ptr(p)) // iterator of &CStr
            .map(|cs| cs.to_bytes()) // iterator of &[u8]
            .map(|bs| str::from_utf8(bs).unwrap()) // iterator of &str
            .map(|ss| ss.to_owned())
    }
}

/// The different ways the u8 color values are encoded into a u32 value.
///
/// A successfull reading from OpenSlide's `read_region()` will result in a buffer of `u32` with
/// `height * width` elements, where `height` and `width` is the shape (in pixels) of the read
/// region. This `u32` value consist of four `u8` values which are the red, green, blue, and alpha
/// value of a certain pixel. This enum determines in which order to arange these channels within
/// one element.
#[derive(Clone, Debug)]
pub enum WordRepresentation {
    /// From most significant bit to least significant bit: `[alpha, red, green, blue]`
    BigEndian,
    /// From most significant bit to least significant bit: `[blue, green, red, alpha]`
    LittleEndian,
}

/// This function takes a buffer, as the one obtained from openslide::read_region, and decodes into
/// an Rgba image buffer.
pub fn decode_buffer(
    buffer: &Vec<u32>,
    width: u32,
    height: u32,
    word_representation: WordRepresentation,
) -> RgbaImage {
    let mut rgba_image = image::RgbaImage::new(width as _, height as _);

    for (col, row, pixel) in rgba_image.enumerate_pixels_mut() {
        let curr_pos = row * width + col;
        let value = buffer[curr_pos as usize];

        let mut buf = [0; 4];
        match word_representation {
            WordRepresentation::BigEndian => byteorder::BigEndian::write_u32(&mut buf, value),
            WordRepresentation::LittleEndian => byteorder::BigEndian::write_u32(&mut buf, value),
        };
        let [alpha, mut red, mut green, mut blue] = buf;

        if alpha != 0 && alpha != 255 {
            red = (red as f32 * (255.0 / alpha as f32))
                .round()
                .max(0.0)
                .min(255.0) as u8;
            green = (green as f32 * (255.0 / alpha as f32))
                .round()
                .max(0.0)
                .min(255.0) as u8;
            blue = (blue as f32 * (255.0 / alpha as f32))
                .round()
                .max(0.0)
                .min(255.0) as u8;
        }

        *pixel = Rgba([red, green, blue, alpha]);
    }

    rgba_image
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open() {
        let res = OpenSlide::open(Path::new("tests/assets/default.svs"));
        match res {
            Ok(_) => (),
            Err(message) => panic!(message),
        }
    }

    #[test]
    fn test_detect_vendor() {
        let format = OpenSlide::detect_vendor(Path::new("tests/assets/default.svs"));
        assert_eq!(format.unwrap(), "aperio");
    }

    #[test]
    fn test_level_dimensions() {
        let slide = OpenSlide::open(Path::new("tests/assets/default.svs")).unwrap();
        let dimensions = slide.level_dimensions(0).unwrap();
        assert_eq!(dimensions, (2220, 2967));
    }

    #[test]
    fn test_read_region() {
        let slide = OpenSlide::open(Path::new("tests/assets/default.svs")).unwrap();
        let region = slide
            .read_region(Address { x: 0, y: 0 }, 0, Size { w: 200, h: 200 })
            .unwrap();

        region.save("wsi_region_2.png").unwrap();
    }

    #[test]
    fn test_properties() {
        let slide = OpenSlide::open(Path::new("tests/assets/default.svs")).unwrap();
        let properties = slide.properties().unwrap();

        assert_eq!(properties.get("aperio.MPP").unwrap(), "0.4990");
    }

    #[test]
    fn test_associated_image_names() {
        let slide = OpenSlide::open(Path::new("tests/assets/default.svs")).unwrap();
        let associated_image_names = slide.associated_image_names().unwrap();

        assert_eq!(associated_image_names, vec!["label", "macro", "thumbnail"]);
    }

    #[test]
    fn test_associated_image() {
        let slide = OpenSlide::open(Path::new("tests/assets/default.svs")).unwrap();
        let associated_image = slide.associated_image("label").unwrap();

        associated_image.save("associated_image.png").unwrap();
    }
}
