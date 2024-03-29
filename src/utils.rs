use std::ffi::CStr;
use std::str;

use byteorder::ByteOrder;
use image::{Rgba, RgbaImage};

/// Calculates the width and height an image should be resized to.
/// This preserves aspect ratio, and based on the `fill` parameter
/// will either fill the dimensions to fit inside the smaller constraint
/// (will overflow the specified bounds on one axis to preserve
/// aspect ratio), or will shrink so that both dimensions are
/// completely contained with in the given `width` and `height`,
/// with empty space on one axis.
pub(crate) fn resize_dimensions(
    width: u32,
    height: u32,
    nwidth: u32,
    nheight: u32,
    fill: bool,
) -> (u32, u32) {
    let ratio = u64::from(width) * u64::from(nheight);
    let nratio = u64::from(nwidth) * u64::from(height);

    let use_width = if fill {
        nratio > ratio
    } else {
        nratio <= ratio
    };
    let intermediate = if use_width {
        u64::from(height) * u64::from(nwidth) / u64::from(width)
    } else {
        u64::from(width) * u64::from(nheight) / u64::from(height)
    };
    if use_width {
        if intermediate <= u64::from(::std::u32::MAX) {
            (nwidth, intermediate as u32)
        } else {
            (
                (u64::from(nwidth) * u64::from(::std::u32::MAX) / intermediate) as u32,
                ::std::u32::MAX,
            )
        }
    } else if intermediate <= u64::from(::std::u32::MAX) {
        (intermediate as u32, nheight)
    } else {
        (
            ::std::u32::MAX,
            (u64::from(nheight) * u64::from(::std::u32::MAX) / intermediate) as u32,
        )
    }
}

/// Transform a null terminate array into an Iterator<String>
pub(crate) fn parse_null_terminated_array(
    array: *const *const ::std::os::raw::c_char,
) -> impl Iterator<Item = String> {
    unsafe {
        let mut counter = 0;
        let mut loc = array;
        while !(*loc).is_null() {
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

/// This function takes a buffer, as the one obtained from `openslide::read_region`, and decodes into
/// an Rgba image buffer.
pub(crate) fn decode_buffer(buffer: &[u32], width: u32, height: u32) -> RgbaImage {
    let mut rgba_image = image::RgbaImage::new(width as _, height as _);

    for (col, row, pixel) in rgba_image.enumerate_pixels_mut() {
        let curr_pos = row * width + col;
        let value = buffer[curr_pos as usize];

        let mut buf = [0; 4];
        byteorder::BigEndian::write_u32(&mut buf, value);
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
            // TODO: get background color from properties
            red = 255;
            green = 255;
            blue = 255;
            alpha = 255;
        }

        *pixel = Rgba([red, green, blue, alpha]);
    }

    rgba_image
}
