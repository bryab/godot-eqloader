pub mod sound;
pub mod texture;
use godot::prelude::*;

pub fn wld_f32_pos_to_gd(tup: &(f32, f32, f32)) -> Vector3 {
    Vector3::new(tup.0, tup.2, tup.1)
}

pub fn wld_i16_pos_to_gd(p: &(i16, i16, i16), scale: f32) -> Vector3 {
    Vector3::new(p.0 as f32 * scale, p.2 as f32 * scale, p.1 as f32 * scale)
}

// fn f32_tup_to_vec2(tup: &(f32, f32)) -> Vector2 {
//     Vector2::new(tup.0, tup.1)
// }

// fn i16_tup_to_vec3(tup: &(i16, i16, i16)) -> Vector3 {
//     Vector3::new(tup.0 as f32, tup.1 as f32, tup.2 as f32)
// }

// fn i8_tup_to_vec3(tup: &(i8, i8, i8)) -> Vector3 {
//     Vector3::new(tup.0 as f32, tup.1 as f32, tup.2 as f32)
// }

// fn i16_tup_to_vec2(tup: &(i16, i16)) -> Vector2 {
//     Vector2::new(tup.0 as f32, tup.1 as f32)
// }

/// Convert an RGBA color value from u32 to Color
pub fn u32_to_color(num: &u32) -> Color {
    let red = (((num >> 24) & 0xff) as f32) / 255.0; // red
    let green = (((num >> 16) & 0xff) as f32) / 255.0; // green
    let blue = (((num >> 8) & 0xff) as f32) / 255.0; // blue
    let alpha = ((num & 0xff) as f32) / 255.0; // alpha

    Color::from_rgba(red, green, blue, alpha)
}
