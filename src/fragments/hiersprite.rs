use godot::engine::animation::{InterpolationType, LoopMode, TrackType};

use godot::engine::{Animation, AnimationLibrary, RefCounted};
use godot::prelude::*;
use libeq::wld::parser::{
    Dag, FragmentRef, FragmentType, FrameTransform, MobSkeletonPieceTrackFragment,
    MobSkeletonPieceTrackReferenceFragment, SkeletonTrackSetFragment, StringReference, WldDoc,
};
use std::collections::HashMap;
use std::sync::Arc;
extern crate owning_ref;
use super::{create_fragment_ref, S3DFragment, S3DMesh};
use crate::util::wld_f32_pos_to_gd;
use crate::wld::{gd_from_frag, gd_from_frag_type};
use owning_ref::ArcRef;

/// The rest animation is unnamed in the EQ data.  We need to give it a name.
const REST_ANIMATION_NAME: &str = "REST";
/// The root bone of the skeleton is unnamed in the EQ data.  We need to give it a name.
const ROOT_BONE_NAME: &str = "ROOT";

pub struct Bone {
    full_name: String,
    name: String,
    bone_index: u32,
    parent_index: i32,
    attachment_ref: u32,
    rest_position: Vector3,
    rest_quaternion: Quaternion,
}
#[derive(GodotClass)]
#[class(init, base=RefCounted)]
pub struct S3DBone {
    #[base]
    base: Base<RefCounted>,
    _bone: Option<Bone>,
    wld: Option<Arc<WldDoc>>,
}

#[godot_api]
impl S3DBone {
    /// The generic name of the bone, excluding the actor tag.
    #[func]
    pub fn name(&self) -> GString {
        GString::from(&self.bone().name)
    }

    /// The full name of the bone, including the actor tag, from the original DAG
    #[func]
    pub fn full_name(&self) -> GString {
        GString::from(&self.bone().full_name)
    }

    /// The bone index, which corresponds to the mesh bone weights
    #[func]
    pub fn bone_index(&self) -> u32 {
        self.bone().bone_index
    }

    /// The parent index of this bone
    #[func]
    pub fn parent_index(&self) -> i32 {
        self.bone().parent_index
    }

    #[func]
    pub fn rest_position(&self) -> Vector3 {
        self.bone().rest_position
    }

    #[func]
    pub fn rest_quaternion(&self) -> Quaternion {
        self.bone().rest_quaternion
    }

    /// If there is an attachment, return it as a Godot class reprsentation of the fragment.  It is usually a Mesh.
    #[func]
    pub fn attachment(&self) -> Variant {
        if self.bone().attachment_ref > 0 {
            gd_from_frag(self.wld.as_ref().unwrap(), self.bone().attachment_ref)
        } else {
            Variant::nil()
        }
    }

    /// If there is an attachment and it is a Mesh, return a S3DMesh
    #[func]
    pub fn attachment_mesh(&self) -> Option<Gd<S3DMesh>> {
        if self.bone().attachment_ref > 0 {
            Some(gd_from_frag_type::<S3DMesh>(
                self.wld.as_ref().unwrap(),
                self.bone().attachment_ref,
            ))
        } else {
            None
        }
    }
}

impl S3DBone {
    pub fn bone(&self) -> &Bone {
        &self._bone.as_ref().unwrap()
    }
    pub fn load(&mut self, wld: &Arc<WldDoc>, bone: Bone) {
        self.wld = Some(wld.clone());
        self._bone = Some(bone);
    }
}

/// The HierSprite (HIERARCHICALSPRITE_DEF)
/// represents a rigged 3D model, usually a character but sometimes other things.
///
/// With this class you can build a character's skeleton, meshes and animations.
#[derive(GodotClass)]
#[class(init, base=RefCounted)]
pub struct S3DHierSprite {
    #[base]
    base: Base<RefCounted>,
    fragment: Option<ArcRef<WldDoc, SkeletonTrackSetFragment>>,
}

impl S3DFragment for S3DHierSprite {
    fn load(&mut self, wld: &Arc<WldDoc>, index: u32) {
        self.fragment = Some(create_fragment_ref(wld.clone(), index));
    }
}

#[godot_api]
impl S3DHierSprite {
    #[func]
    pub fn name(&self) -> GString {
        GString::from(self._name())
    }

    #[func]
    pub fn tag(&self) -> GString {
        GString::from(self._tag())
    }

    #[func]
    pub fn bones(&self) -> Array<Gd<S3DBone>> {
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

                let trackdef = self.get_dag_rest_trackdef(&dag);
                let rest_frame = &trackdef.frame_transforms.as_ref().unwrap()[0];

                Bone {
                    bone_index: index as u32,
                    full_name: String::from(dag_name),
                    name: bone_name,
                    parent_index: -1,
                    attachment_ref: dag.mesh_or_sprite_reference,
                    rest_quaternion: frame_quaternion(&rest_frame),
                    rest_position: frame_position(&rest_frame),
                }
            })
            .collect();

        // Now set the parent_index of each bone

        for (index, dag) in frag.dags.iter().enumerate() {
            for sub_dag in &dag.sub_dags {
                bones[*sub_dag as usize].parent_index = index as i32;
            }
        }

        // Now convert to GodotClasses

        bones
            .into_iter()
            .map(|bone| {
                let mut gdbone = Gd::<S3DBone>::default();
                gdbone.bind_mut().load(&wld, bone);
                gdbone
            })
            .collect()
    }

    /// The meshes used by this Skeleton (usually a head and a body)
    /// These meshes should have bone assignments that correspond to the bone indices of the skeleton.
    #[func]
    pub fn meshes(&self) -> Array<Gd<S3DMesh>> {
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
                                        Some(gd_from_frag_type::<S3DMesh>(wld, index))
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

    // Returns a dictionary, where keys are animation names and values are frame tranforms for each DAG
    #[func]
    pub fn animation_dict(&self) -> Dictionary {
        self._animations().into_iter().collect()
    }

    // Returns a dictionary, where keys are animation names and values are frame tranforms for each DAG
    #[func]
    pub fn animation_library(&self) -> Gd<AnimationLibrary> {
        let mut library = AnimationLibrary::new_gd();
        for (animation_name, animation) in self._animations() {
            library.add_animation(StringName::from(animation_name), animation);
        }
        library
    }
}

impl S3DHierSprite {
    // Returns a HashMap, where keys are animation names and values are frame tranforms for each DAG
    fn _animations(&self) -> HashMap<String, Gd<Animation>> {
        let actor_tag = self._tag();
        let wld = self.get_wld();
        let frag = self.get_frag();

        let mut animations: HashMap<String, Gd<Animation>> = HashMap::new();

        let skeleton_path = "";

        // Animations are organized in a strange fashion.
        // It's likely that the animation lookups are heavily hard-coded in the EQ client.
        // However for my purposes I would like to 'discover' all the animations in the WLD
        // And present them in an easy-to-use way.

        // Each DAG (bone) references an animation track for that bone in a single animation.
        // That animation is the "rest" animation, but there are others in the file, and they are not referenced by anything.

        // To find the other animations, you first get the referenced track and get its name,
        // something like "HUM_BL_R_TRACKDEF" where HUM is the "actor tag" and "BL_R" is the bone name.
        // Other animations will end with this same suffix but will have a new prefix for the animation,
        // Something like D02HUM_BL_R_TRACKDEF, where D02 is the animation name.

        // For this reason, we construct our animations in parallel, looping over the DAGs rather than the animations.

        // Get a cache of all Trackdefs beforehand - might speed things up a little.
        // It would be better if this could be cached on the whole wld.

        let all_trackdefs: Vec<&MobSkeletonPieceTrackFragment> = wld
            .fragment_iter::<MobSkeletonPieceTrackFragment>()
            .collect();

        for dag in &frag.dags {
            let rest_trackdef = self.get_dag_rest_trackdef(&dag);
            let rest_trackdef_name = wld.get_string(rest_trackdef.name_reference).unwrap();
            let dag_name = wld
                .get_string(StringReference::new(dag.name_reference))
                .expect("Dag should have a name");
            let bone_name = bone_name_from_dag(&actor_tag, dag_name);

            let matching_trackdefs: Vec<&&MobSkeletonPieceTrackFragment> = all_trackdefs
                .iter()
                .filter_map(|trackdef| {
                    let trackdef_name = wld
                        .get_string(trackdef.name_reference)
                        .expect("TRACKDEFs should have names");
                    if trackdef_name.ends_with(rest_trackdef_name) {
                        Some(trackdef)
                    } else {
                        None
                    }
                })
                .collect();

            for dag_trackdef in matching_trackdefs {
                let dag_trackdef_name = wld.get_string(dag_trackdef.name_reference).unwrap();
                let mut animation_name = dag_trackdef_name.replace(rest_trackdef_name, "");
                if animation_name == "" {
                    animation_name = String::from(REST_ANIMATION_NAME);
                }
                if !animations.contains_key(&animation_name) {
                    let mut anim = Animation::new_gd();
                    anim.set_loop_mode(LoopMode::LINEAR); // FIXME: Playback mode may be in fragment.  Defaulting to looping.
                    animations.insert(animation_name.clone(), anim);
                }
                let anim = animations.get_mut(&animation_name).unwrap();
                let pos_track_idx = anim.add_track(TrackType::POSITION_3D);
                let rot_track_idx = anim.add_track(TrackType::ROTATION_3D);

                let bone_path = NodePath::from(format!("{0}:{1}", skeleton_path, bone_name));
                anim.track_set_path(pos_track_idx, bone_path.clone());
                // This is the default
                // anim.track_set_interpolation_type(
                //     pos_track_idx,
                //     InterpolationType::INTERPOLATION_LINEAR,
                // );
                anim.track_set_path(rot_track_idx, bone_path);
                anim.track_set_interpolation_type(
                    rot_track_idx,
                    InterpolationType::LINEAR_ANGLE, // Linear interpolation with shortest path rotation.  This seems to match EQ better, but there are problems.
                );

                let frame_transforms = &dag_trackdef.frame_transforms.as_ref().unwrap();

                let secs_per_frame: f64 = 0.1; // FIXME: 100 ms is the default, but it can be different - and this is in TRACK not TRACKDEF

                for (frame_num, frame_transform) in frame_transforms.iter().enumerate() {
                    let frame_secs = frame_num as f64 * secs_per_frame;
                    anim.position_track_insert_key(
                        pos_track_idx,
                        frame_secs,
                        frame_position(frame_transform),
                    );
                    anim.rotation_track_insert_key(
                        rot_track_idx,
                        frame_secs,
                        frame_quaternion(frame_transform),
                    );
                }

                // NOTE: Some tracks of the animation will be shorter than others, or have only a single keyframe.
                // For this reason we cannot set the duration on the first DAG we find, but rather make sure it's as long
                // As the longest DAG animation.

                // NOTE: The final keyframe in the animation seems to always be a duplicate of the first keyframe (sometimes with some errors)
                // For this reason, the duration should be one frame shorter.

                let duration = (frame_transforms.len() - 1) as f32 * secs_per_frame as f32;

                // ALSO NOTE: Godot seems to automatically extend the duration of the animation if you add keys beyond it.
                // But since the final keyframe is a duplicate of the first, we must force the duration.
                // FIXME: This should probably only be done for looping animations.  One-offs likely require the final keyframe

                anim.set_length(duration); // FIXME: This is getting run repeatedly for no reason
            }
        }

        animations
    }

    fn _tag(&self) -> String {
        self._name().replace("_HS_DEF", "")
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

    /// Each DAG will reference the animation track for the rest-pose animation.
    /// Return that trackdef
    fn get_dag_rest_trackdef(&self, dag: &Dag) -> &MobSkeletonPieceTrackFragment {
        let wld = self.get_wld();
        let track_ref = wld
            .get(&FragmentRef::<MobSkeletonPieceTrackReferenceFragment>::new(
                dag.track_reference as i32,
            ))
            .expect("DAG should have a valid MobSkeletonPieceTrackReferenceFragment");
        wld.get(&track_ref.reference)
            .expect("SkeletonTrackSetReference should reference a SkeletonTrackSet")
    }
}

/// Extracts the generic bone name from the DAG name.
/// If the root bone, return "ROOT"
fn bone_name_from_dag(actor_tag: &str, dag_name: &str) -> String {
    let bone_name = dag_name.replace(actor_tag, "").replace("_DAG", "");
    if bone_name == "" {
        return String::from(ROOT_BONE_NAME);
    }
    bone_name
}

fn frame_position(transform: &FrameTransform) -> Vector3 {
    if transform.shift_denominator == 0 {
        return Vector3::ZERO;
    }
    let shift_denominator = transform.shift_denominator as f32;
    wld_f32_pos_to_gd(&(
        transform.shift_x_numerator as f32 / shift_denominator,
        transform.shift_y_numerator as f32 / shift_denominator,
        transform.shift_z_numerator as f32 / shift_denominator,
    ))
}

fn frame_quaternion(transform: &FrameTransform) -> Quaternion {
    Quaternion::new(
        transform.rotate_x_numerator as f32 * -1.,
        transform.rotate_z_numerator as f32,
        transform.rotate_y_numerator as f32,
        transform.rotate_denominator as f32,
    )
    .normalized()
}
