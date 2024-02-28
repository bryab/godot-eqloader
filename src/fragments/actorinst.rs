use godot::engine::RefCounted;
use godot::prelude::*;
use libeq_wld::parser::{Location, Actor, WldDoc};
use std::sync::Arc;
extern crate owning_ref;
use crate::wld::gd_from_frag;
use super::{create_fragment_ref, S3DFragment};
use crate::util::{u32_to_color, wld_degrees_rot_to_quat, wld_f32_pos_to_gd};
use owning_ref::ArcRef;
#[cfg(feature = "serde")]
use super::frag_to_dict;

#[derive(GodotClass)]
#[class(init)]
pub struct S3DActorInstance {
    base: Base<RefCounted>,
    fragment: Option<ArcRef<WldDoc, Actor>>,
}

#[godot_api]
impl S3DActorInstance {
    // FIXME: This appears to be empty
    #[func]
    pub fn name(&self) -> GString {
        GString::from(
            self.get_wld()
                .get_string(self.get_frag().name_reference)
                .expect("Failed to get string from WLD!"),
        )
    }

    #[func]
    pub fn actordef_name(&self) -> GString {
        // Note - If this is an invalid string reference,
        // Then it is probably actually a fragment reference.
        // The referenced fragment can be obtained via zone_actordef()
        GString::from(
            self.get_wld()
                .get_string(self.get_frag().actor_def_reference)
                .unwrap_or("")
        )
    }


    /// In a Zone WLD, the Actor can be obtained directly from this fragment,
    /// Unlike in placeable objects that refer to an actor defined in a different WLd
    /// by name.
    /// This method returns S3DActorDef or nil
    #[func]
    pub fn zone_actordef(&self) -> Variant {
        let wld = self.get_wld();
        let index = self.get_frag().actor_def_reference.0;
        if index <= 0 {
            return Variant::nil()
        }
        gd_from_frag(wld, index as u32)
    }

    /// Returns the vertex colors to be used for this instance, converted into Godot format.
    #[func]
    pub fn vertex_colors(&self) -> PackedColorArray {
        let wld = self.get_wld();
        let reference = match wld.get(&self.get_frag().vertex_color_reference.as_ref().unwrap()) {
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
        wld_degrees_rot_to_quat(loc.rotate_x, loc.rotate_y, loc.rotate_z)
    }

    #[func]
    pub fn rotation(&self) -> Vector3 {
        self.quaternion().to_euler(EulerOrder::XYZ)
    }

    #[cfg(feature = "serde")]
    #[func]
    pub fn as_dict(&self) -> Dictionary {
        let frag = self.get_frag();
        let wld = self.get_wld();
        frag_to_dict(wld, frag)
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

    fn get_frag(&self) -> &Actor {
        self.fragment
            .as_ref()
            .expect("Failed to get Fragment reference!")
    }
}
