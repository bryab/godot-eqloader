use godot::engine::RefCounted;
use godot::prelude::*;
use libeq::wld::parser::{MaterialDef, DmSpriteDef2, WldDoc};
use std::sync::Arc;
extern crate owning_ref;
use super::{create_fragment_ref, S3DFragment};
use crate::util::{u32_to_color, wld_f32_pos_to_gd, wld_i16_pos_to_gd};
use owning_ref::ArcRef;

/// A wrapper around the Mesh fragment the WLD.  In the future this will also wrap around the AlternateMesh fragment.
/// This class provides convenient functions for getting mesh data in a format that is usable in Godot.
#[derive(GodotClass)]
#[class(init, base=RefCounted)]
pub struct S3DMesh {
    #[base]
    base: Base<RefCounted>,
    fragment: Option<ArcRef<WldDoc, DmSpriteDef2>>,
}

impl S3DFragment for S3DMesh {
    fn load(&mut self, wld: &Arc<WldDoc>, index: u32) {
        self.fragment = Some(create_fragment_ref(wld.clone(), index));
    }
}

#[godot_api]
impl S3DMesh {
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
        self.fragment.as_ref().unwrap().flags
    }

    #[func]
    pub fn center(&self) -> Vector3 {
        wld_f32_pos_to_gd(&self.get_frag().center)
    }

    /// Returns the vertex positions of the mesh, converted into Godot format.
    #[func]
    pub fn vertices(&self) -> PackedVector3Array {
        let frag = self.get_frag();
        let scale = 1.0 / (1 << frag.scale) as f32;
        frag.positions
            .iter()
            .map(|p| wld_i16_pos_to_gd(&p, scale))
            .collect::<PackedVector3Array>()
    }

    /// Returns the vertex normals of the mesh, converted into Godot format.
    #[func]
    pub fn normals(&self) -> PackedVector3Array {
        self.get_frag()
            .vertex_normals
            .iter()
            .map(|p| Vector3::new(p.0 as f32 / 127., p.2 as f32 / 127., p.1 as f32 / 127.))
            .collect::<PackedVector3Array>()
    }

    /// Returns the vertex colors of the mesh (if present), converted into Godot format.
    /// Note that for re-usable actors such as trees, the vertex colors are not part of the mesh definition, but instead part of the object definition.
    #[func]
    pub fn vertex_colors(&self) -> PackedColorArray {
        self.get_frag()
            .vertex_colors
            .iter()
            .map(u32_to_color)
            .collect::<PackedColorArray>()
    }

    /// Returns the UV coordinates of the mesh, converted into Godot format.
    #[func]
    pub fn uvs(&self) -> PackedVector2Array {
        self.get_frag()
            .texture_coordinates
            .iter()
            .map(|p| Vector2::new(1.0 - p.0 as f32 / 256. * -1., 1.0 - p.1 as f32 / 256.))
            .collect::<PackedVector2Array>()
    }

    /// Returns skin assignment groups, converted into Godot format.
    /// The Godot format requires that there are 4 bone indices per vertex, but we only use the first of the four.
    #[func]
    pub fn bone_indices(&self) -> PackedInt32Array {
        self.get_frag()
            .skin_assignment_groups
            .iter()
            .flat_map(|(num_verts, bone_idx)| {
                vec![*bone_idx as i32, 0, 0, 0].repeat(*num_verts as usize)
            })
            .collect()
    }

    /// Returns a bone weights array that matches the `bone_indices` array.
    /// The Godot format requires that there are 4 bone indices per vertex, but we only use the first of the four.
    #[func]
    pub fn bone_weights(&self) -> PackedFloat32Array {
        self.get_frag()
            .skin_assignment_groups
            .iter()
            .flat_map(|(num_verts, _bone_idx)| vec![1., 0., 0., 0.].repeat(*num_verts as usize))
            .collect()
    }

    /// Returns an array of material groups.  
    /// Material groups are two-tuples.  The first element is the name of the material.  
    /// The second element is the array of indices for the polygons that use this material.
    ///
    /// Invisible materials are skipped entirely.
    #[func]
    pub fn face_material_groups(&self) -> Array<VariantArray> {
        let wld = self.get_wld();
        let materials = self.materials();
        let mut pos = 0;
        let frag = self.get_frag();
        frag.face_material_groups
            .iter()
            .enumerate()
            .filter_map(|(_, (poly_count, ref material_idx))| {
                let material = materials[*material_idx as usize];

                let count = *poly_count as usize;
                let next_pos = pos + count;
                let batch = pos..next_pos;
                pos = next_pos;

                // If the material flags are 0, this is an invisible material.
                // Since we are dealing with collision separately, we can simply omit these polygons as they serve no purpose for rendering.
                // FIXME: It may be desirable to keep these for debugging purposes.  It would be wise to provide a flag for this.
                // if material.render_method.as_u32() == 0 {
                //     return None;
                // }

                let indices: PackedInt32Array = frag
                    .faces
                    .get(batch)
                    .expect("Tried to get a Face from a Mesh that does not exist!")
                    .iter()
                    .flat_map(|face| {
                        vec![
                            face.vertex_indexes.0 as i32,
                            face.vertex_indexes.1 as i32,
                            face.vertex_indexes.2 as i32,
                        ]
                    })
                    .collect();

                let mut array = VariantArray::new();
                array.push(Variant::from(GString::from(
                    wld.get_string(material.name_reference)
                        .expect("Material name should be a valid string"),
                )));
                array.push(Variant::from(indices));
                return Some(array);
            })
            .collect()
    }

    /// Get all the indices that form polygons of the mesh.
    /// NOTE: This should not normally be used if you wish to actually apply materials to surfaces.
    /// To do so, you must get the indices of each material group, and add each material group as a separate surface.
    #[func]
    pub fn indices(&self) -> PackedInt32Array {
        self.get_frag()
            .faces
            .iter()
            .flat_map(|v| {
                vec![
                    v.vertex_indexes.2 as i32,
                    v.vertex_indexes.1 as i32,
                    v.vertex_indexes.0 as i32,
                ]
                .into_iter()
            })
            .collect()
    }

    /// Construct an array of Vector3s that represent the concave collision mesh for this mesh
    #[func]
    pub fn collision_vertices(&self) -> PackedVector3Array {
        let frag = self.get_frag();
        let scale = 1.0 / (1 << frag.scale) as f32;
        frag.faces
            .iter()
            .filter(|face| face.flags & 0x10 == 0)
            .flat_map(|face| {
                vec![
                    wld_i16_pos_to_gd(&frag.positions[face.vertex_indexes.2 as usize], scale),
                    wld_i16_pos_to_gd(&frag.positions[face.vertex_indexes.1 as usize], scale),
                    wld_i16_pos_to_gd(&frag.positions[face.vertex_indexes.0 as usize], scale),
                ]
            })
            .collect()
    }
}

impl S3DMesh {
    fn materials(&self) -> Vec<&MaterialDef> {
        let wld = self.get_wld();
        wld.get(&self.get_frag().material_list_ref)
            .expect("Invalid material list reference")
            .fragments
            .iter()
            .map(|fragment_ref| {
                wld.get(fragment_ref)
                    .expect("Material should exist - it's in the material list")
            })
            .collect()
    }

    fn get_wld(&self) -> &Arc<WldDoc> {
        self.fragment
            .as_ref()
            .expect("Failed to get WLD reference!")
            .as_owner()
    }

    fn get_frag(&self) -> &DmSpriteDef2 {
        self.fragment
            .as_ref()
            .expect("Failed to get Fragment reference!")
    }
}
