use godot::engine::RefCounted;
use godot::prelude::*;
use libeq::wld::parser::{FragmentRef, MeshReferenceFragment, ModelFragment, WldDoc};
use std::sync::Arc;
extern crate owning_ref;
use super::{create_fragment_ref, S3DFragment, S3DMesh};
use crate::wld::gd_from_frag_type;
use owning_ref::ArcRef;

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
pub struct S3DActorDef {
    #[base]
    base: Base<RefCounted>,
    fragment: Option<ArcRef<WldDoc, ModelFragment>>,
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
    pub fn name(&self) -> GodotString {
        GodotString::from(
            self.get_wld()
                .get_string(self.get_frag().name_reference)
                .expect("Failed to get string from WLD!"),
        )
    }

    fn get_wld(&self) -> &Arc<WldDoc> {
        self.fragment
            .as_ref()
            .expect("Failed to get WLD reference!")
            .as_owner()
    }

    fn get_frag(&self) -> &ModelFragment {
        self.fragment
            .as_ref()
            .expect("Failed to get Fragment reference!")
    }

    #[func]
    fn meshes(&self) -> Array<Gd<S3DMesh>> {
        let wld = self.get_wld();
        self.get_frag()
            .fragment_references
            .iter()
            .filter_map(|fragment_ref| {
                let mesh_reference_ref =
                    FragmentRef::<MeshReferenceFragment>::new(*fragment_ref as i32);
                let mesh_reference = wld.get(&mesh_reference_ref)?;
                match mesh_reference.reference {
                    FragmentRef::Index(index, _) => Some(gd_from_frag_type::<S3DMesh>(wld, index)),
                    FragmentRef::Name(_, _) => None,
                }
            })
            .collect()
    }
}
