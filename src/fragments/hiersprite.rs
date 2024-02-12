use godot::engine::animation::{InterpolationType, TrackType};

use godot::engine::{Animation, AnimationLibrary, RefCounted};
use godot::prelude::*;
use libeq_wld::parser::{
    Dag, FragmentRef, FragmentType, FrameTransform, HierarchicalSpriteDef, LegacyFrameTransform,
    MaterialDef, StringReference, Track, TrackDef, WldDoc,
};
use std::collections::HashMap;
use std::sync::Arc;
extern crate owning_ref;
#[cfg(feature = "serde")]
use super::frag_to_dict;
use super::{create_fragment_ref, S3DFragment, S3DMesh};
use crate::util::wld_f32_pos_to_gd;
use crate::wld::gd_from_frag;
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
#[class(init)]
pub struct S3DBone {
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

    /// If there is an attachment, return it as a Godot class reprsentation of the fragment.  It is usually a MeshReference
    #[func]
    pub fn attachment(&self) -> Variant {
        if self.bone().attachment_ref > 0 {
            gd_from_frag(self.wld.as_ref().unwrap(), self.bone().attachment_ref)
        } else {
            Variant::nil()
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
#[class(init)]
pub struct S3DHierSprite {
    base: Base<RefCounted>,
    fragment: Option<ArcRef<WldDoc, HierarchicalSpriteDef>>,
}

impl S3DFragment for S3DHierSprite {
    fn load(&mut self, wld: &Arc<WldDoc>, index: u32) {
        self.fragment = Some(create_fragment_ref(wld.clone(), index));
    }
}

fn parse_frame_transform(trackdef: &TrackDef, index: usize) -> (Quaternion, Vector3) {
    match &trackdef.frame_transforms {
        Some(frame_transforms) => {
            let rest_frame = &frame_transforms[index];
            (frame_quaternion(&rest_frame), frame_position(&rest_frame))
        }
        None => {
            let rest_frame = &trackdef.legacy_frame_transforms.as_ref().unwrap()[index];
            (
                legacy_frame_quaternion(&rest_frame),
                legacy_frame_position(&rest_frame),
            )
        }
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
            .filter_map(|(index, dag)| {
                let dag_name = wld
                    .get_string(StringReference::new(dag.name_reference))
                    .expect("Dag should have a name");
                let bone_name = bone_name_from_dag(&self._tag(), &dag_name);

                let track = self.get_dag_rest_track(&dag);
                let trackdef = wld.get(&track.reference)?;
                let (rest_quaternion, rest_position) = parse_frame_transform(trackdef, 0);

                Some(Bone {
                    bone_index: index as u32,
                    full_name: String::from(dag_name),
                    name: bone_name,
                    parent_index: -1,
                    attachment_ref: dag.mesh_or_sprite_reference,
                    rest_quaternion,
                    rest_position,
                })
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
                    FragmentType::DmSprite(mesh_reference) => {
                        S3DMesh::from_reference(wld, mesh_reference)
                    }
                    _ => {
                        godot_print!("Hiersprite references a non-mesh: {:?}", fragment);
                        None
                    }
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

    // Returns a list of material names that correspond to this actor, for different skin variations
    #[func]
    pub fn skin_material_names(&self) -> PackedStringArray {
        let actor_tag = self._tag();
        let wld = self.get_wld();

        wld.fragment_iter::<MaterialDef>()
            .filter_map(|material| {
                let name = wld.get_string(material.name_reference).unwrap();
                if name.starts_with(&actor_tag) {
                    Some(GString::from(name))
                } else {
                    None
                }
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

        // Get a cache of all Tracks beforehand - might speed things up a little.
        // It would be better if this could be cached on the whole wld.

        let all_tracks: Vec<&Track> = wld.fragment_iter::<Track>().collect();

        for dag in &frag.dags {
            let rest_track = self.get_dag_rest_track(&dag);
            let rest_track_name = wld.get_string(rest_track.name_reference).unwrap();
            let dag_name = wld
                .get_string(StringReference::new(dag.name_reference))
                .expect("Dag should have a name");
            let bone_name = bone_name_from_dag(&actor_tag, dag_name);

            let matching_tracks: Vec<&&Track> = all_tracks
                .iter()
                .filter_map(|track| {
                    let trackdef_name = wld
                        .get_string(track.name_reference)
                        .expect("TRACKs should have names");
                    if trackdef_name.ends_with(rest_track_name) {
                        Some(track)
                    } else {
                        None
                    }
                })
                .collect();

            for dag_track in matching_tracks {
                let dag_track_name = wld.get_string(dag_track.name_reference).unwrap();

                //let dag_trackdef_name = wld.get_string(dag_trackdef.name_reference).unwrap();
                let mut animation_name = dag_track_name.replace(rest_track_name, "");
                if animation_name == "" {
                    animation_name = String::from(REST_ANIMATION_NAME);
                }
                if !animations.contains_key(&animation_name) {
                    let mut anim = Animation::new_gd();
                    if !(dag_track.flags.interpolate()) {
                        godot_error!("Track no interpolate: {}", dag_track_name);
                    }
                    anim.set_length(0.); // Default length is 1 second.  Set to 0, as we calculate length later.
                                         //anim.set_loop_mode(LoopMode::LINEAR); // FIXME: Playback mode may be in fragment.  Defaulting to loopingw.
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

                let dag_trackdef = wld
                    .get(&dag_track.reference)
                    .expect("TRACK should reference TRACKDEF");

                let sleep = dag_track.sleep.unwrap_or(100); // 100 ms is the default - sometimes this is explicit, sometimes it is not.
                let secs_per_frame: f64 = sleep as f64 * 0.001;

                for frame_index in 0..dag_trackdef.frame_count {
                    let (rotation, position) =
                        parse_frame_transform(&dag_trackdef, frame_index as usize);
                    let frame_secs = frame_index as f64 * secs_per_frame;
                    anim.position_track_insert_key(pos_track_idx, frame_secs, position);
                    anim.rotation_track_insert_key(rot_track_idx, frame_secs, rotation);
                }

                // NOTE: Some tracks of the animation will be shorter than others, or have only a single keyframe.
                // For this reason we cannot set the duration on the first DAG we find, but rather make sure it's as long
                // As the longest DAG animation.

                let duration = dag_trackdef.frame_count as f32 * secs_per_frame as f32;

                if anim.get_length() < duration {
                    anim.set_length(duration);
                }
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

    fn get_frag(&self) -> &HierarchicalSpriteDef {
        self.fragment
            .as_ref()
            .expect("Failed to get Fragment reference!")
    }

    /// Each DAG will reference the animation track for the rest-pose animation.
    /// Return that trackdef
    fn get_dag_rest_track(&self, dag: &Dag) -> &Track {
        let wld = self.get_wld();
        wld.get(&FragmentRef::<Track>::new(dag.track_reference as i32))
            .expect("DAG should have a valid Track")
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

fn legacy_frame_position(transform: &LegacyFrameTransform) -> Vector3 {
    if transform.shift_denominator == 0. {
        return Vector3::ZERO;
    }
    let shift_denominator = transform.shift_denominator;
    wld_f32_pos_to_gd(&(
        transform.shift_x_numerator / shift_denominator,
        transform.shift_y_numerator / shift_denominator,
        transform.shift_z_numerator / shift_denominator,
    ))
}

fn legacy_frame_quaternion(transform: &LegacyFrameTransform) -> Quaternion {
    Quaternion::new(
        transform.rotate_x * -1.,
        transform.rotate_z,
        transform.rotate_y,
        transform.rotate_w,
    )
    .normalized()
}
