use godot::engine::node::InternalMode;
use godot::engine::{BoneAttachment3D, RefCounted, Skeleton3D};
use godot::prelude::*;
use libeq::wld::parser::{
    Dag, FragmentRef, FragmentType, MeshReferenceFragment, MobSkeletonPieceTrackReferenceFragment,
    SkeletonTrackSetFragment, StringReference, WldDoc,
};
use std::sync::Arc;
extern crate owning_ref;
use super::{create_fragment_ref, S3DFragment, S3DMesh};
use crate::wld::create_fragment;
use owning_ref::ArcRef;

pub struct Bone {
    raw_name: String,
    name: String,
    bone_index: u32,
    parent_index: u32,
    attachment_ref: u32,
}
#[derive(GodotClass)]
#[class(init, base=RefCounted)]
pub struct S3DBone {
    #[base]
    base: Base<RefCounted>,
    _bone: Option<Bone>,
}

#[godot_api]
impl S3DBone {
    #[func]
    pub fn name(&self) -> GodotString {
        GodotString::from(&self.bone().name)
    }

    #[func]
    pub fn raw_name(&self) -> GodotString {
        GodotString::from(&self.bone().raw_name)
    }

    #[func]
    pub fn bone_index(&self) -> u32 {
        self.bone().bone_index
    }

    #[func]
    pub fn parent_index(&self) -> u32 {
        self.bone().parent_index
    }

    /// FIXME: This should get a GodotClass of the attached Mesh or other object
    #[func]
    pub fn attachment(&self) -> Variant {
        Variant::nil()
    }
}

impl S3DBone {
    pub fn bone(&self) -> &Bone {
        &self._bone.as_ref().unwrap()
    }
    pub fn load(&mut self, bone: Bone) {
        self._bone = Some(bone);
    }
}

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
pub struct S3DSkeleton {
    #[base]
    base: Base<RefCounted>,
    fragment: Option<ArcRef<WldDoc, SkeletonTrackSetFragment>>,
}

impl S3DFragment for S3DSkeleton {
    fn load(&mut self, wld: &Arc<WldDoc>, index: u32) {
        self.fragment = Some(create_fragment_ref(wld.clone(), index));
    }
}

#[godot_api]
impl S3DSkeleton {
    #[func]
    pub fn name(&self) -> GodotString {
        GodotString::from(self._name())
    }

    #[func]
    pub fn tag(&self) -> GodotString {
        GodotString::from(self._tag())
    }

    #[func]
    fn bones(&self) -> Array<Gd<S3DBone>> {
        let wld = self.get_wld();
        let frag = self.get_frag();

        // First make a flat list of the bones

        let mut bones: Vec<Bone> = frag
            .dags
            .iter()
            .enumerate()
            .map(|(index, dag)| {
                let dag_name = wld
                    .get_string(StringReference::new(dag.name_reference))
                    .expect("Dag should have a name");
                let bone_name = bone_name_from_dag(&self._tag(), &dag_name);
                Bone {
                    bone_index: index as u32,
                    raw_name: String::from(dag_name),
                    name: bone_name,
                    parent_index: 0,
                    attachment_ref: dag.mesh_or_sprite_reference,
                }
            })
            .collect();

        // Now set the parent_index of each bone

        for (index, dag) in frag.dags.iter().enumerate() {
            for sub_dag in &dag.sub_dags {
                bones[*sub_dag as usize].parent_index = index as u32;
            }
        }

        // Now convert to GodotClasses

        bones
            .into_iter()
            .map(|bone| {
                let mut gdbone = Gd::<S3DBone>::new_default();
                gdbone.bind_mut().load(bone);
                gdbone
            })
            .collect()
    }

    /// The meshes used by this Skeleton (usually a head and a body)
    /// These meshes should have bone assignments that correspond to the bone indices of the skeleton.
    #[func]
    fn meshes(&self) -> Array<Gd<S3DMesh>> {
        let wld = self.get_wld();
        let meshes = match self.get_frag().dm_sprites.as_ref() {
            Some(meshes) => meshes,
            None => return Array::new(),
        };

        meshes
            .iter()
            .filter_map(|fragment_ref| {
                // This could be a MeshReference or something else.
                // We ignore everything except meshes.
                let fragment = wld
                    .at(*fragment_ref as usize - 1)
                    .expect("Fragment index should exist in wld");
                match &fragment {
                    FragmentType::MeshReference(mesh_reference) => {
                        // FIXME: MeshReferenceFragment can reference an AlternateMesh.
                        // This occurs in global_chr, resulting in a panic in create_fragment
                        // As a quick fix I am re-checking the actual type of the underlying index to make sure it's Mesh, not AlternateMesh
                        match mesh_reference.reference {
                            FragmentRef::Index(index, _) => {
                                let fragment = wld.at(index as usize - 1).unwrap();
                                match fragment {
                                    FragmentType::Mesh(_) => {
                                        Some(create_fragment::<S3DMesh>(wld, index))
                                    }
                                    _ => None,
                                }
                            }
                            FragmentRef::Name(_, _) => None,
                        }
                    }
                    _ => None,
                }
            })
            .collect()
    }
}

impl S3DSkeleton {
    fn _tag(&self) -> String {
        self._name().replace("ACTORDEF_", "")
    }

    fn _name(&self) -> &str {
        self.get_wld()
            .get_string(self.get_frag().name_reference)
            .expect("Failed to get string from WLD!")
    }
    fn get_wld(&self) -> &Arc<WldDoc> {
        self.fragment
            .as_ref()
            .expect("Failed to get WLD reference!")
            .as_owner()
    }

    fn get_frag(&self) -> &SkeletonTrackSetFragment {
        self.fragment
            .as_ref()
            .expect("Failed to get Fragment reference!")
    }
}

fn bone_name_from_dag(actor_tag: &str, dag_name: &str) -> String {
    dag_name.replace(&format!("{actor_tag}_"), "")
}
