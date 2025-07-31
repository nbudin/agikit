use std::{fs::File, io::Write, time::UNIX_EPOCH};

use image::{ImageBuffer, Pixel, Rgba, RgbaImage};

use crate::{color_palettes::ColorPalette, picture::render::PixelBuffer};

pub fn write_and_edit(file_ext: &str, content: &str) {
    let bytes = format!("{}", content).as_str().bytes().collect::<Vec<_>>();
    write_and_edit_bytes(file_ext, &bytes);
}

pub fn write_and_edit_bytes(file_ext: &str, bytes: &[u8]) {
    let tmp_path = std::env::temp_dir().join(format!(
        "debug-{}{}",
        std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        file_ext
    ));
    File::create(&tmp_path).unwrap().write(bytes).unwrap();
    std::process::Command::new("code")
        .arg(&tmp_path)
        .spawn()
        .unwrap();
}

pub fn render_image(
    pixel_buffer: &PixelBuffer<u8>,
    palette: &ColorPalette,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut image = RgbaImage::new(pixel_buffer.width as u32, pixel_buffer.height as u32);
    for (index, pixel) in pixel_buffer.buffer.iter().enumerate() {
        image.put_pixel(
            (index % pixel_buffer.width) as u32,
            (index / pixel_buffer.height) as u32,
            *Rgba::from_slice(&palette.colors[*pixel as usize]),
        );
    }

    image
}
