use godot::engine::image::Format;
use godot::engine::{Image, ImageTexture};
use godot::prelude::*;
use image::codecs::bmp::BmpDecoder;
use image::DynamicImage;
use std::io::Cursor;

/// Loads raw BMP data of any format, converts to RGB8 if necessary, and creates ImageTexture
/// Note this does a lossy conversion from 16bit to 8bit for some UI textures - but not sure it matters for Everquest.
/// The "key color" for cutout transparency is the first color in the BMP palette.  This is stored as metadata in the Godot texture to be used later.
pub fn tex_from_bmp(bmp_data: Vec<u8>) -> Result<Gd<ImageTexture>, &'static str> {
    let mut file = Cursor::new(bmp_data);
    let decoder = BmpDecoder::new(&mut file).map_err(|_| "Invalid bitmap data!")?;
    let palette = decoder.get_palette().unwrap();
    let key_color = palette[0];
    //godot_print!("Key color: {:?}", key_color);
    let bmp = DynamicImage::from_decoder(decoder).unwrap();
    // Note: EQ BMPs seem to have an unused alpha channel.  It is discarded here.
    let bmp = bmp.into_rgb8();
    let image = Image::create_from_data(
        i64::from(bmp.width()),
        i64::from(bmp.height()),
        false,
        Format::FORMAT_RGB8,
        PackedByteArray::from(bmp.into_raw().as_slice()),
    )
    .unwrap();
    let mut tex = ImageTexture::create_from_image(image).unwrap();
    tex.set_meta(
        StringName::from("key_color"),
        Variant::from(Color::from_rgb(
            key_color[0] as f32 / 255.0,
            key_color[1] as f32 / 255.0,
            key_color[2] as f32 / 255.0,
        )),
    );
    Ok(tex)
}
