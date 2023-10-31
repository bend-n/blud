#![doc = include_str!("../README.md")]
mod fastblur;

use fimg::Image;
use umath::FF32;

/// Blur a image.
pub fn blur<const CHANNELS: usize, T: AsRef<[u8]> + AsMut<[u8]>>(
    image: &mut Image<T, CHANNELS>,
    radius: FF32,
) {
    let pixels: &mut [[u8; CHANNELS]] = unsafe {
        std::slice::from_raw_parts_mut(
            image.buffer_mut().as_mut().as_mut_ptr().cast(),
            image.len() / CHANNELS,
        )
    };
    unsafe {
        fastblur::gaussian_blur(
            pixels,
            image.width() as usize,
            image.height() as usize,
            radius,
        )
    };
}
