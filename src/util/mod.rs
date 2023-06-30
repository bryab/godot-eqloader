pub mod sound;
pub mod texture;
use godot::prelude::*;
use std::f32::consts::PI;
pub fn wld_f32_pos_to_gd(tup: &(f32, f32, f32)) -> Vector3 {
    Vector3::new(tup.0 * -1., tup.2, tup.1)
}

pub fn wld_i16_pos_to_gd(p: &(i16, i16, i16), scale: f32) -> Vector3 {
    Vector3::new(
        p.0 as f32 * scale * -1.,
        p.2 as f32 * scale,
        p.1 as f32 * scale,
    )
}

pub fn wld_rot_to_quat(rot: &(f32, f32, f32)) -> Quaternion {
    // The quaternion must be created with the native EQ XYZ first, due to rotation order.

    // FIXME: from_euler should be a static function (it is in GDScript)
    let q = Quaternion::new(1., 1., 1., 1.).from_euler(Vector3::new(
        rot.0 / 512. * 360.0 * PI / 180.,
        rot.1 / 512. * 360.0 * PI / 180.,
        rot.2 / 512. * 360.0 * PI / 180.,
    ));

    // Then we flip axes
    // FIXME: This can probably be expressed without these two separate transformations
    Quaternion::new(-q.x, q.z, -q.y, q.w)
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
