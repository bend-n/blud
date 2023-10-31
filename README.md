# blud

A fast linear-time gaussian blur based on
<http://blog.ivank.net/fastest-gaussian-blur.html>.

This implementation was based on <https://github.com/lsr0/fastblur>.

The function in this crate blurs a image, with any number of channels, and the given blur radius.
Performance is roughly linear time, space is O(_n_).

# Example

Blur an [`Image`](<https://docs.rs/fimg/latest/fimg/struct.Image.html>)

```rust
fn blur_fast(image: &mut fimg::Image<&mut [u8], 4>, radius: f32) {
    blud::blur(image, unsafe { umath::FF32::new(radius) })
}
```

# Changes:
  - Optimization (fmath)
  - Optimization (bounds)

