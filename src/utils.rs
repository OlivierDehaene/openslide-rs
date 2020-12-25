use std::ffi::CStr;
use std::str;

use byteorder::ByteOrder;
use image::{Rgba, RgbaImage};

pub fn parse_null_terminated_array(array: *const *const i8) -> impl Iterator<Item = String> {
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
        let [mut alpha, mut red, mut green, mut blue] = buf;

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
        } else if alpha == 0 {
            // TODO: parse background color from properties
            red = 255;
            green = 255;
            blue = 255;
            alpha = 255;
        }

        *pixel = Rgba([red, green, blue, alpha]);
    }

    rgba_image
}
