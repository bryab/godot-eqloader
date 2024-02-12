use godot::engine::RefCounted;
use godot::prelude::*;
use libeq_wld::parser::{MaterialDef, RenderMethod, SimpleSpriteDef, WldDoc};
use std::sync::Arc;
extern crate owning_ref;
use super::{create_fragment_ref, S3DFragment};
use owning_ref::ArcRef;
#[cfg(feature = "serde")]
use super::frag_to_dict;
// FIXME: Enums are not yet supported in gdext rust.  For now just handle in GDScript
/// Source: LanternExtractor
/// (https://github.com/LanternEQ/LanternExtractor/blob/afe174b71ac9f9ab75e259bac2282735b093426d/LanternExtractor/EQ/Wld/DataTypes/MaterialType.cs)
// pub enum MaterialType {
//     /// Used for boundaries that are not rendered. TextInfoReference can be null or have reference.
//     Boundary = 0x0,
//     /// Standard diffuse shader
//     Diffuse = 0x01,
//     /// Diffuse variant
//     Diffuse2 = 0x02,
//     //// Transparent with 0.5 blend strength
//     Transparent50 = 0x05,
//     /// Transparent with 0.25 blend strength
//     Transparent25 = 0x09,
//     /// Transparent with 0.75 blend strength
//     Transparent75 = 0x0A,
//     /// Non solid surfaces that shouldn't really be masked
//     TransparentMaskedPassable = 0x07,
//     TransparentAdditiveUnlit = 0x0B,
//     TransparentMasked = 0x13,
//     Diffuse3 = 0x14,
//     Diffuse4 = 0x15,
//     TransparentAdditive = 0x17,
//     Diffuse5 = 0x19,
//     InvisibleUnknown = 0x53,
//     Diffuse6 = 0x553,
//     CompleteUnknown = 0x1A, // TODO: Analyze this
//     Diffuse7 = 0x12,
//     Diffuse8 = 0x31,
//     InvisibleUnknown2 = 0x4B,
//     DiffuseSkydome = 0x0D,     // Need to confirm
//     TransparentSkydome = 0x0F, // Need to confirm
//     TransparentAdditiveUnlitSkydome = 0x10,
//     InvisibleUnknown3 = 0x03,
//     CompleteUnknown2 = 0x06, // Found on a "floor" wall in tanarus 'thecity'
// }

#[derive(GodotClass)]
#[class(init)]
pub struct S3DMaterial {
    base: Base<RefCounted>,
    fragment: Option<ArcRef<WldDoc, MaterialDef>>,
}

impl S3DFragment for S3DMaterial {
    fn load(&mut self, wld: &Arc<WldDoc>, index: u32) {
        self.fragment = Some(create_fragment_ref(wld.clone(), index));
    }
}

/// The S3DMaterial object simplifies the Materials and Textures system in S3D files, flattening it into something that is easy to use in Godot.
#[godot_api]
impl S3DMaterial {
    #[func]
    pub fn name(&self) -> GString {
        GString::from(
            self.get_wld()
                .get_string(self.get_frag().name_reference)
                .expect("Failed to get string from WLD!"),
        )
    }

    #[func]
    pub fn flags(&self) -> u32 {
        self.get_frag().flags
    }

    /// Returns true if the material is visible.  Invisible materials refer to polygons that have collision but are invisible.
    #[func]
    pub fn visible(&self) -> bool {
        return self.get_frag().render_method.as_u32() != 0;
    }

    /// Returns the index number of the correct shader for this material.
    /// This must be mapped to a shader created in Godot to be used.
    #[func]
    pub fn shader_type_id(&self) -> u32 {
        match self.get_frag().render_method {
            RenderMethod::UserDefined { material_type } => material_type as u32,
            _ => 0,
        }
    }

    /// For animated textures, there will be multiple filenames.
    #[func]
    fn texture_filenames(&self) -> PackedStringArray {
        self.iter_texture_filenames().collect()
    }

    /// The filename for the material's color texture.
    #[func]
    fn texture_filename(&self) -> GString {
        self.iter_texture_filenames()
            .nth(0)
            .expect("No texture filename in Texture")
    }

    /// For animated textures, the delay between each frame in seconds
    #[func]
    fn delay(&self) -> f32 {
        match self.get_simple_sprite().sleep {
            Some(sleep) => sleep as f32 * 0.001,
            None => 0.,
        }
    }

    #[cfg(feature = "serde")]
    #[func]
    pub fn as_dict(&self) -> Dictionary {
        let frag = self.get_frag();
        let wld = self.get_wld();
        frag_to_dict(wld, frag)
    } 
    
    
}

impl S3DMaterial {
    fn get_wld(&self) -> &Arc<WldDoc> {
        self.fragment
            .as_ref()
            .expect("Failed to get WLD reference!")
            .as_owner()
    }

    fn get_frag(&self) -> &MaterialDef {
        self.fragment
            .as_ref()
            .expect("Failed to get Fragment reference!")
    }

    fn iter_texture_filenames(&self) -> impl Iterator<Item = GString> + '_ {
        let wld = self.get_wld();
        let simplesprite = self.get_simple_sprite();
        simplesprite
            .frame_references
            .iter()
            // [TextureFragment]s reference a [TextureImagesFragment]
            .map(move |r| wld.get(&r))
            .flat_map(|image| match image {
                // The [TextureImagesFragment] itself contains a collection of filenames. In
                // practice this seems to always be just a single filename.
                Some(i) => i
                    .entries
                    .iter()
                    // These also seem to be stored in all caps. The s3d files however store
                    // filenames in lowercase. This accounts for that.
                    .map(|e| GString::from(e.file_name.to_lowercase()))
                    .collect::<Vec<_>>(),
                None => vec![],
            })
    }

    fn get_simple_sprite(&self) -> &SimpleSpriteDef {
        let wld = self.get_wld();
        let simplespriteref = wld
            .get(&self.get_frag().reference)
            .expect("Invalid TextureReference");
        wld.get(&simplespriteref.reference)
            .expect("Invalid SimpleSprite")
    }
}
