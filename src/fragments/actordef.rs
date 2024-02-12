use godot::engine::RefCounted;
use godot::prelude::*;
use libeq_wld::parser::{FragmentRef, DmSprite, ActorDef, WldDoc};
use std::sync::Arc;
extern crate owning_ref;
use super::{create_fragment_ref, S3DFragment, S3DMesh};
use owning_ref::ArcRef;
#[cfg(feature = "serde")]
use super::frag_to_dict;

#[derive(GodotClass)]
#[class(init)]
pub struct S3DActorDef {
    base: Base<RefCounted>,
    fragment: Option<ArcRef<WldDoc, ActorDef>>,
}

impl S3DFragment for S3DActorDef {
    fn load(&mut self, wld: &Arc<WldDoc>, index: u32) {
        self.fragment = Some(create_fragment_ref(wld.clone(), index));
    }
    
}

/// The S3DMaterial object simplifies the Materials and Textures system in S3D files, flattening it into something that is easy to use in Godot.
#[godot_api]
impl S3DActorDef {
    #[func]
    pub fn name(&self) -> GString {
        GString::from(
            self.get_wld()
                .get_string(self.get_frag().name_reference)
                .expect("Failed to get string from WLD!")
        )
    }

    #[func]
    fn callback_name(&self) -> GString {
        GString::from(
            self.get_wld()
            .get_string(self.get_frag().callback_name_reference)
            .expect("Failed to get string from WLD!")
        )
    }

    #[func]
    fn meshes(&self) -> Array<Gd<S3DMesh>> {
        let wld = self.get_wld();
        self.get_frag()
            .fragment_references
            .iter()
            .filter_map(|fragment_ref| {
                let mesh_reference_ref =
                    FragmentRef::<DmSprite>::new(*fragment_ref as i32);
                let mesh_reference = wld.get(&mesh_reference_ref)?;
                S3DMesh::from_reference(wld, mesh_reference)
                
            })
            .collect()
    }

    #[cfg(feature = "serde")]
    #[func]
    pub fn as_dict(&self) -> Dictionary {
        let frag = self.get_frag();
        let wld = self.get_wld();
        frag_to_dict(wld, frag)
    }
}

impl S3DActorDef {

    fn get_wld(&self) -> &Arc<WldDoc> {
        self.fragment
            .as_ref()
            .expect("Failed to get WLD reference!")
            .as_owner()
    }

    fn get_frag(&self) -> &ActorDef {
        self.fragment
            .as_ref()
            .expect("Failed to get Fragment reference!")
    }
}
