use godot::engine::RefCounted;
use godot::prelude::*;
use libeq::wld::parser::{Location, ObjectLocationFragment, WldDoc};
use std::sync::Arc;
extern crate owning_ref;
use super::{create_fragment_ref, S3DFragment};
use crate::util::{u32_to_color, wld_f32_pos_to_gd, wld_rot_to_quat};
use owning_ref::ArcRef;

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
pub struct S3DActorInstance {
    #[base]
    base: Base<RefCounted>,
    fragment: Option<ArcRef<WldDoc, ObjectLocationFragment>>,
}

#[godot_api]
impl S3DActorInstance {
    // FIXME: This appears to be empty
    #[func]
    pub fn name(&self) -> GodotString {
        GodotString::from(
            self.get_wld()
                .get_string(self.get_frag().name_reference)
                .expect("Failed to get string from WLD!"),
        )
    }

    #[func]
    pub fn actordef_name(&self) -> GodotString {
        GodotString::from(
            self.get_wld()
                .get_string(self.get_frag().actor_def_reference)
                .expect("Failed to get string from WLD!"),
        )
    }

    /// Returns the vertex colors to be used for this instance, converted into Godot format.
    #[func]
    pub fn vertex_colors(&self) -> PackedColorArray {
        let wld = self.get_wld();
        let reference = match wld.get(&self.get_frag().vertex_color_reference) {
            Some(reference) => reference,
            None => {
                return PackedColorArray::new(); // FIXME: Should return Variant::nil()
            }
        };
        wld.get(&reference.reference)
            .expect("VertexColorReferenceFragment should always reference a VertexColorFragment")
            .vertex_colors
            .iter()
            .map(u32_to_color)
            .collect::<PackedColorArray>()
    }

    #[func]
    pub fn position(&self) -> Vector3 {
        let loc = self.get_loc();
        wld_f32_pos_to_gd(&(loc.x, loc.y, loc.z))
    }

    #[func]
    pub fn scale(&self) -> Vector3 {
        let frag = self.get_frag();
        let scale_factor = frag
            .scale_factor
            .expect("EQ ActorInstance should have scale_factor");
        let bounding_radius = frag
            .scale_factor
            .expect("EQ ActorInstance should have bounding_radius");
        Vector3::new(scale_factor, bounding_radius, scale_factor)
    }

    #[func]
    pub fn quaternion(&self) -> Quaternion {
        let loc = self.get_loc();
        wld_rot_to_quat(&(loc.rotate_x, loc.rotate_y, loc.rotate_z))
    }

    #[func]
    pub fn rotation(&self) -> Vector3 {
        self.quaternion().to_euler(EulerOrder::XYZ)
    }
}

impl S3DFragment for S3DActorInstance {
    fn load(&mut self, wld: &Arc<WldDoc>, index: u32) {
        self.fragment = Some(create_fragment_ref(wld.clone(), index));
    }
}

impl S3DActorInstance {
    fn get_loc(&self) -> &Location {
        self.get_frag()
            .location
            .as_ref()
            .expect("ActorInstanceFragment should always have Location")
    }

    fn get_wld(&self) -> &Arc<WldDoc> {
        self.fragment
            .as_ref()
            .expect("Failed to get WLD reference!")
            .as_owner()
    }

    fn get_frag(&self) -> &ObjectLocationFragment {
        self.fragment
            .as_ref()
            .expect("Failed to get Fragment reference!")
    }
}
