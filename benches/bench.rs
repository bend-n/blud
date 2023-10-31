use fimg::Image;
use umath::FF32;

fn bench_blur_image() {
    let mut image_bytes = include_bytes!("../assets/cballs.imgbuf").to_vec();
    blud::blur::<3, _>(&mut Image::build(800, 200).buf(&mut *image_bytes), unsafe {
        FF32::new(15.0)
    });
    assert_eq!(image_bytes, include_bytes!("../assets/blurred.imgbuf"));
    iai::black_box(image_bytes);
}

iai::main!(bench_blur_image);
