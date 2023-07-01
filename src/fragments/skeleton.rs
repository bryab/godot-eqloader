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

    /// The meshes used by this Skeleton (usually a head and a body)
    /// These meshes should have bone assignments that correspond to the bone indices of the skeleton.
    #[func]
    fn meshes(&self) -> Array<Gd<S3DMesh>> {
        let wld = self.get_wld();
        let meshes = self.get_frag().dm_sprites.as_ref();

        match meshes {
            Some(meshes) => meshes
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
                .collect(),
            None => Array::new(),
        }
    }

    /// Generate a Skeleton3D from this EQ Skeleton
    /// The bones will have generic names, so that animations from other characters can be used with it.
    /// Any attachments are also created, but the referenced meshes are not instantiated.
    #[func]
    pub fn skeleton(&self) -> Gd<Skeleton3D> {
        let frag = self.get_frag();
        let mut skel = Skeleton3D::new_alloc();
        let root_dag = &frag.dags[0];

        self.traverse_dag_tree(&root_dag, &frag.dags, &mut skel, -1);

        skel
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

    /// A recursive function to traverse the DAG tree, creating bones along the way on the referenced skeleton.
    fn traverse_dag_tree(
        &self,
        dag: &Dag,
        dags: &Vec<Dag>,
        skel: &mut Gd<Skeleton3D>,
        parent_idx: i64,
    ) {
        let wld = self.get_wld();
        let dag_name = wld
            .get_string(StringReference::new(dag.name_reference))
            .expect("Dag should have a name");
        // The bone name is the name of the DAG with the actor's tag removed.
        // This is important for retargeting animation onto a different actor - the skeleton must have generic bone names.
        let mut bone_name = bone_name_from_dag(&self._tag(), &dag_name);
        if bone_name == "" {
            bone_name = String::from("ROOT");
        }
        // FIXME: Need to check if there is already a bone with this name.
        skel.add_bone(GodotString::from(&bone_name));
        let bone_idx = skel.get_bone_count() - 1;

        let track_ref = wld
            .get(&FragmentRef::<MobSkeletonPieceTrackReferenceFragment>::new(
                dag.track_reference as i32,
            ))
            .expect("DAG should have a valid MobSkeletonPieceTrackReferenceFragment");
        let track = wld
            .get(&track_ref.reference)
            .expect("SkeletonTrackSetReference should reference a SkeletonTrackSet");

        let frames = &track.frame_transforms;
        let frame = &frames[0];
        if parent_idx > -1 && parent_idx != bone_idx {
            skel.set_bone_parent(bone_idx, parent_idx);
        }

        if dag.mesh_or_sprite_reference > 0 {
            // NOTE: This bone references a specific mesh, but since we are making a "vanilla skeleton" here
            // I am not instantiating it
            // This will have to be handled in some way...
            let mesh_reference =
                &FragmentRef::<MeshReferenceFragment>::new(dag.mesh_or_sprite_reference as i32);
            let mut bone_attachment = BoneAttachment3D::new_alloc();
            bone_attachment.set_name(GodotString::from(format!("BONE_{0}", &bone_name)));
            bone_attachment.set_bone_name(GodotString::from(&bone_name));
            skel.add_child(
                bone_attachment.upcast(),
                false,
                InternalMode::INTERNAL_MODE_DISABLED,
            );
        }

        for dag_id in &dag.sub_dags {
            self.traverse_dag_tree(&dags[*dag_id as usize], dags, skel, bone_idx);
        }
    }
}

fn bone_name_from_dag(actor_tag: &str, dag_name: &str) -> String {
    dag_name.replace(&format!("{actor_tag}_"), "")
}
