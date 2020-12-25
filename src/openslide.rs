use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::path::Path;
use std::str;

use image::RgbaImage;
use openslide_sys as sys;
use std::ptr::null_mut;

use crate::utils::{decode_buffer, parse_null_terminated_array, WordRepresentation};

pub struct Address {
    pub x: u32,
    pub y: u32,
}

pub struct Size {
    pub h: u32,
    pub w: u32,
}

pub struct OpenSlide {
    /// OpenSlide sys
    data: *mut sys::OpenSlide,
    /// Properties
    pub properties: HashMap<String, String>,
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

    pub fn open(path: &Path) -> Result<OpenSlide, String> {
        if !path.exists() {
            return Err(String::from(format!(
                "Missing image file: {}",
                path.display()
            )));
        }

        let path_cstr = CString::new(path.to_str().unwrap()).unwrap();
        let slide_ptr = unsafe { sys::openslide_open(path_cstr.as_ptr()) };

        if slide_ptr.is_null() {
            return Err(String::from("Unsupported image file"));
        }
        get_error(slide_ptr)?;

        let slide = OpenSlide {
            data: slide_ptr,
            properties: get_properties(slide_ptr)?,
        };

        Ok(slide)
    }

    /// The number of levels in the image.
    pub fn level_count(&self) -> Result<u32, String> {
        let level_count = unsafe { sys::openslide_get_level_count(self.data) as u32 };
        get_error(self.data)?;

        Ok(level_count)
    }

    pub fn dimensions(&self) -> Result<(u64, u64), String> {
        self.level_dimensions(0)
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

        get_error(self.data)?;

        Ok((w as _, h as _))
    }

    pub fn level_downsample(&self, level: u32) -> Result<f64, String> {
        if level >= self.level_count()? {
            return Err(format!("Level {} out of range", level));
        }

        let level_downsample =
            unsafe { sys::openslide_get_level_downsample(self.data, level as _) };
        get_error(self.data)?;

        Ok(level_downsample)
    }

    pub fn best_level_for_downsample(&self, downsample: f64) -> Result<u32, String> {
        let best_level =
            unsafe { sys::openslide_get_best_level_for_downsample(self.data, downsample) };
        get_error(self.data)?;

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
        get_error(self.data)?;

        Ok(decode_buffer(
            &dest,
            size.w,
            size.h,
            WordRepresentation::BigEndian,
        ))
    }

    pub fn associated_image_names(&self) -> Result<Vec<String>, String> {
        unsafe {
            let name_array = sys::openslide_get_associated_image_names(self.data);
            get_error(self.data)?;

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
}

fn get_error(slide_ptr: *mut sys::OpenSlide) -> Result<(), String> {
    unsafe {
        let slice = sys::openslide_get_error(slide_ptr);

        if slice.is_null() {
            Ok(())
        } else {
            Err(CStr::from_ptr(slice).to_string_lossy().into_owned())
        }
    }
}

fn get_property(slide_ptr: *mut sys::OpenSlide, name: &str) -> Result<String, String> {
    let cstr = CString::new(name).unwrap();
    let value = unsafe {
        let slice = sys::openslide_get_property_value(slide_ptr, cstr.as_ptr());

        if slice.is_null() {
            None
        } else {
            Some(CStr::from_ptr(slice).to_string_lossy().into_owned())
        }
    };
    get_error(slide_ptr)?;

    match value {
        None => Err(format!("Property {} doesn't exist.", name)),
        Some(value) => Ok(value),
    }
}

fn get_properties(slide_ptr: *mut sys::OpenSlide) -> Result<HashMap<String, String>, String> {
    unsafe {
        let name_array = sys::openslide_get_property_names(slide_ptr);
        get_error(slide_ptr)?;

        let mut properties = HashMap::new();

        for name in parse_null_terminated_array(name_array) {
            let value = get_property(slide_ptr, &name)?;
            properties.insert(name, value);
        }
        Ok(properties)
    }
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
            .read_region(Address { x: 0, y: 0 }, 0, Size { w: 3000, h: 3000 })
            .unwrap();

        region.save("wsi_region_2.png").unwrap();
    }

    #[test]
    fn test_properties() {
        let slide = OpenSlide::open(Path::new("tests/assets/default.svs")).unwrap();

        assert_eq!(slide.properties.get("aperio.MPP").unwrap(), "0.4990");
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
