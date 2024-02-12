use godot::engine::image::Format;
use godot::engine::{Image, ImageTexture};
use godot::prelude::*;
use image::codecs::bmp::BmpDecoder;
use image::DynamicImage;
use std::io::Cursor;

/// Creates an Image from the bytes representing a BMP file.
/// The image is converted to RGB8 if it is a format that is unsupported in Godot.
/// The "key color" for cutout transparency is the first color in the BMP palette.  This is stored as metadata in the Godot texture to be used later.
pub fn image_from_bmp(bmp_data: Vec<u8>) -> Result<Gd<Image>, &'static str> {
    let mut file = Cursor::new(bmp_data);
    let decoder = BmpDecoder::new(&mut file).map_err(|_| "Invalid bitmap data!")?;
    // NOTE: It is not necessary to get the BMP palette except for images with cutout transparency.
    // Possibly this operation should be optional if it is expensive, but it doesn't seem to be.
    let key_color = match decoder.get_palette() {
        Some(palette) => Variant::from(Color::from_rgb(
            palette[0][0] as f32 / 255.0,
            palette[0][1] as f32 / 255.0,
            palette[0][2] as f32 / 255.0,
        )),
        None => Variant::nil(),
    };
    let bmp = DynamicImage::from_decoder(decoder).map_err(|_| "Failed to decode BMP data!")?;
    let (width, height, image_format, buffer) = match bmp {
        DynamicImage::ImageRgb8(buffer) => (
            buffer.width(),
            buffer.height(),
            Format::RGB8,
            buffer.into_raw(),
        ),
        DynamicImage::ImageRgba8(buffer) => (
            buffer.width(),
            buffer.height(),
            Format::RGBA8,
            buffer.into_raw(),
        ),
        _ => {
            godot_warn!(
                "Unsupported image type: {:?}  Converting image to RGB8",
                bmp
            );
            let buffer = bmp.into_rgb8();
            (
                buffer.width(),
                buffer.height(),
                Format::RGB8,
                buffer.into_raw(),
            )
        }
    };
    let mut image = Image::create_from_data(
        width as i32,
        height as i32,
        false,
        image_format,
        PackedByteArray::from(&buffer[..]),
    )
    .ok_or_else(|| "Failed to create Godot Image from Image")?;
    image.set_meta(StringName::from("key_color"), key_color);
    Ok(image)
}

/// Creates an ImageTexture from the bytes representing a BMP file.
/// The image is converted to RGB8 if it is a format that is unsupported in Godot.
/// The "key color" for cutout transparency is the first color in the BMP palette.  This is stored as metadata in the Godot image to be used later.
pub fn tex_from_bmp(bmp_data: Vec<u8>) -> Result<Gd<ImageTexture>, &'static str> {
    let image = image_from_bmp(bmp_data)?;
    let key_color = image.get_meta("key_color".into());
    let mut tex = ImageTexture::create_from_image(image)
        .ok_or_else(|| "Failed to create Godot ImageTexture from Godot Image")?;
    tex.set_meta(StringName::from("key_color"), key_color);
    Ok(tex)
}

// For testing only - load the BMP using Godot's build in BMP decoder.
// This is much slower than using the image crate, in my tests.
// fn tex_from_bmp_gd(bmp_data: Vec<u8>) -> Result<Gd<ImageTexture>, &'static str> {
//     let mut image = Image::new();
//     image.load_bmp_from_buffer(PackedByteArray::from(&bmp_data[..]));
//     Ok(ImageTexture::create_from_image(image).unwrap())
// }
