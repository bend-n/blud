// Forked from <https://github.com/fschutt/fastblur>, in turn based on
// the article in <http://blog.ivank.net/fastest-gaussian-blur.html>

use std::cmp::min;
use umath::FF32;

/// Blur an image slice of pixel arrays
///
/// In-place blur image provided image pixel data, with any number of channels. Will make a single
/// allocation for a backing buffer. Expects pixel data as a slice of CHANNELS sized arrays, for
/// use with a byte slice, use [`gaussian_blur_bytes`][super::gaussian_blur_bytes].
///
/// # Arguments
/// * `CHANNELS`: number of channels in the image data, e.g. 3 for RGB, 4 for RGBA, 1 for luminance
/// * `data`: pixel data, `width` x `height` in length
/// * `data` will be modified in-place
/// * `width`, `height`: in pixels
///
/// # Safety
///
/// fast math go brr, data must be width * height sized
pub unsafe fn gaussian_blur<const CHANNELS: usize>(
    data: &mut [[u8; CHANNELS]],
    width: usize,
    height: usize,
    blur_radius: FF32,
) {
    let boxes = create_box_gauss::<CHANNELS>(blur_radius);
    let mut backbuf = data.to_owned();

    for &box_size in boxes.iter() {
        let radius = ((box_size - 1) / 2) as usize;
        box_blur(&mut backbuf, data, width, height, radius, radius);
    }
}

#[inline]
unsafe fn create_box_gauss<const N: usize>(sigma: FF32) -> [i32; N] {
    if sigma > 0.0 {
        let n_float = FF32::new(N as f32);

        let w_ideal = (FF32::new(12.0) * sigma * sigma / n_float).sqrt() + 1.0;
        let mut wl: i32 = w_ideal.floor() as i32;

        if wl % 2 == 0 {
            wl -= 1;
        };

        let wu = wl + 2;

        let wl_float = FF32::new(wl as f32);
        let m_ideal = (FF32::new(12.0) * sigma * sigma
            - n_float * wl_float * wl_float
            - FF32::new(4.0) * n_float * wl_float
            - FF32::new(3.0) * n_float)
            / (FF32::new(-4.0) * wl_float - FF32::new(4.0));
        let m: usize = m_ideal.round() as usize;

        let mut sizes = [0; N];

        for (i, pass) in sizes.iter_mut().enumerate() {
            if i < m {
                *pass = wl;
            } else {
                *pass = wu;
            }
        }
        sizes
    } else {
        [1; N]
    }
}

#[inline]
fn box_blur<const CHANNELS: usize>(
    backbuf: &mut [[u8; CHANNELS]],
    frontbuf: &mut [[u8; CHANNELS]],
    width: usize,
    height: usize,
    blur_radius_horz: usize,
    blur_radius_vert: usize,
) {
    box_blur_horz(backbuf, frontbuf, width, height, blur_radius_horz);
    box_blur_vert(frontbuf, backbuf, width, height, blur_radius_vert);
}

macro_rules! C {
    ($buf:ident[$n:expr]) => {
        unsafe { *$buf.get_unchecked($n) }
    };
    ($buf:ident[$n:expr] = $e:expr) => {
        *unsafe { $buf.get_unchecked_mut($n) } = $e
    };
    ($buf:ident[$a:expr][$b:expr]) => {
        unsafe { *$buf.get_unchecked($a).get_unchecked($b) }
    };
    ($buf:ident[$a:expr][$b:expr] = $c:expr) => {
        *unsafe { $buf.get_unchecked_mut($a).get_unchecked_mut($b) } = unsafe { $c }
    };
}

#[inline]
fn box_blur_vert<const CHANNELS: usize>(
    backbuf: &[[u8; CHANNELS]],
    frontbuf: &mut [[u8; CHANNELS]],
    width: usize,
    height: usize,
    blur_radius: usize,
) {
    if blur_radius == 0 {
        frontbuf.copy_from_slice(backbuf);
        return;
    }

    let iarr = 1.0 / (blur_radius + blur_radius + 1) as f32;

    for i in 0..width {
        let col_start = i;
        let col_end = i + width * (height - 1);
        let mut ti: usize = i;
        let mut li: usize = ti;
        let mut ri: usize = ti + blur_radius * width;

        let fv: [u8; CHANNELS] = C!(backbuf[col_start]);
        let lv: [u8; CHANNELS] = C!(backbuf[col_end]);

        let mut vals: [isize; CHANNELS] = [0; CHANNELS];
        for i in 0..CHANNELS {
            vals[i] = (blur_radius as isize + 1) * isize::from(fv[i]);
        }

        let get_top = |i: usize| {
            if i < col_start {
                fv
            } else {
                C! { backbuf[i] }
            }
        };

        let get_bottom = |i: usize| {
            if i > col_end {
                lv
            } else {
                C! { backbuf[i] }
            }
        };

        for j in 0..min(blur_radius, height) {
            let bb = C! { backbuf[ti + j * width] };
            for i in 0..CHANNELS {
                vals[i] += isize::from(bb[i]);
            }
        }
        if blur_radius > height {
            for i in 0..CHANNELS {
                vals[i] += (blur_radius - height) as isize * isize::from(lv[i]);
            }
        }

        for _ in 0..min(height, blur_radius + 1) {
            let bb = get_bottom(ri);
            ri += width;
            for i in 0..CHANNELS {
                vals[i] += isize::from(bb[i]) - isize::from(fv[i]);
            }

            for i in 0..CHANNELS {
                C! { frontbuf[ti][i] = *round(FF32::new(vals[i] as f32) * iarr) as u8 };
            }
            ti += width;
        }

        if height > blur_radius {
            for _ in (blur_radius + 1)..(height - blur_radius) {
                let bb1 = C! { backbuf[ri] };
                ri += width;
                let bb2 = C! { backbuf[li] };
                li += width;

                for i in 0..CHANNELS {
                    vals[i] += isize::from(bb1[i]) - isize::from(bb2[i]);
                }

                for i in 0..CHANNELS {
                    C! { frontbuf[ti][i] = *round(FF32::new(vals[i] as f32) * iarr) as u8 };
                }
                ti += width;
            }

            for _ in 0..min(height - blur_radius - 1, blur_radius) {
                let bb = get_top(li);
                li += width;

                for i in 0..CHANNELS {
                    vals[i] += isize::from(lv[i]) - isize::from(bb[i]);
                }

                for i in 0..CHANNELS {
                    C! { frontbuf[ti][i] = *round(FF32::new(vals[i] as f32) * iarr) as u8 };
                }
                ti += width;
            }
        }
    }
}

#[inline]
fn box_blur_horz<const CHANNELS: usize>(
    backbuf: &[[u8; CHANNELS]],
    frontbuf: &mut [[u8; CHANNELS]],
    width: usize,
    height: usize,
    blur_radius: usize,
) {
    if blur_radius == 0 {
        frontbuf.copy_from_slice(backbuf);
        return;
    }

    let iarr = 1.0 / (blur_radius + blur_radius + 1) as f32;

    for i in 0..height {
        let row_start: usize = i * width;
        let row_end: usize = i * width + width - 1;
        let mut ti: usize = i * width;
        let mut li: usize = ti;
        let mut ri: usize = ti + blur_radius;

        let fv: [u8; CHANNELS] = C! { backbuf[row_start] };
        let lv: [u8; CHANNELS] = C! { backbuf[row_end] };

        let mut vals: [isize; CHANNELS] = [0; CHANNELS];
        for i in 0..CHANNELS {
            vals[i] = (blur_radius as isize + 1) * isize::from(fv[i]);
        }

        let get_left = |i: usize| {
            if i < row_start {
                fv
            } else {
                C! { backbuf[i] }
            }
        };

        let get_right = |i: usize| {
            if i > row_end {
                lv
            } else {
                C! { backbuf[i] }
            }
        };

        for j in 0..min(blur_radius, width) {
            let bb = C! { backbuf[ti + j] };
            for i in 0..CHANNELS {
                vals[i] += isize::from(bb[i]);
            }
        }
        if blur_radius > width {
            for i in 0..CHANNELS {
                vals[i] += (blur_radius - height) as isize * isize::from(lv[i]);
            }
        }

        for _ in 0..min(width, blur_radius + 1) {
            let bb = get_right(ri);
            ri += 1;
            for i in 0..CHANNELS {
                vals[i] += isize::from(bb[i]) - isize::from(fv[i]);
            }

            for i in 0..CHANNELS {
                C! { frontbuf[ti][i] = *round(FF32::new(vals[i] as f32) * iarr) as u8 };
            }
            ti += 1;
        }

        if width > blur_radius {
            for _ in (blur_radius + 1)..(width - blur_radius) {
                let bb1 = C! { backbuf[ri] };
                ri += 1;
                let bb2 = C! { backbuf[li] };
                li += 1;

                for i in 0..CHANNELS {
                    vals[i] += isize::from(bb1[i]) - isize::from(bb2[i]);
                }

                for i in 0..CHANNELS {
                    C! { frontbuf[ti][i] = *round(FF32::new(vals[i] as f32) * iarr) as u8 };
                }
                ti += 1;
            }

            for _ in 0..min(width - blur_radius - 1, blur_radius) {
                let bb = get_left(li);
                li += 1;

                for i in 0..CHANNELS {
                    vals[i] += isize::from(lv[i]) - isize::from(bb[i]);
                }

                for i in 0..CHANNELS {
                    C! { frontbuf[ti][i] = *round(FF32::new(vals[i] as f32) * iarr) as u8 };
                }
                ti += 1;
            }
        }
    }
}

#[inline]
/// Source: https://stackoverflow.com/a/42386149/585725
fn round(mut x: FF32) -> FF32 {
    x += 12582912.0;
    x -= 12582912.0;
    x
}
