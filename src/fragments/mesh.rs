use godot::engine::RefCounted;
use godot::prelude::*;
use libeq_wld::parser::{DmSpriteDef, DmSpriteDef2, FragmentType, MaterialDef, WldDoc, DmSprite, FragmentRef};
use std::sync::Arc;
extern crate owning_ref;
use super::{create_fragment_ref, S3DFragment};
use crate::util::{u32_to_color, wld_f32_pos_to_gd, wld_i16_pos_to_gd};
use crate::wld::gd_from_frag_type;
use owning_ref::ArcRef;
#[cfg(feature = "serde")]
use super::frag_to_dict;

trait MeshProvider {
    fn name(&self) -> GString;
    fn flags(&self) -> u32;
    fn get_wld(&self) -> &Arc<WldDoc>;
    fn center(&self) -> Vector3;
    fn vertices(&self) -> PackedVector3Array;
    fn normals(&self) -> PackedVector3Array; 
    fn vertex_colors(&self) -> PackedColorArray;
    fn uvs(&self) -> PackedVector2Array; 
    fn bone_indices(&self) -> PackedInt32Array;   
    fn bone_weights(&self) -> PackedFloat32Array;
    fn face_material_groups(&self) -> Array<VariantArray>; 
    fn indices(&self) -> PackedInt32Array;   
    fn collision_vertices(&self) -> PackedVector3Array;
    #[cfg(feature = "serde")]
    fn as_dict(&self) -> Dictionary;
}

struct DmSprite2Provider {
   fragment: ArcRef<WldDoc, DmSpriteDef2>
}

impl MeshProvider for DmSprite2Provider {

    fn name(&self) -> GString {
        GString::from(
            self.get_wld()
                .get_string(self.get_frag().name_reference)
                .expect("Failed to get string from WLD!"),
        )
    }

    fn flags(&self) -> u32 {
        self.fragment.as_ref().flags
    }
    fn get_wld(&self) -> &Arc<WldDoc> {
        self.fragment
            .as_owner()
    }

    fn center(&self) -> Vector3 {
        wld_f32_pos_to_gd(&self.get_frag().center)
    }


    fn vertices(&self) -> PackedVector3Array {
        let frag = self.get_frag();
        let scale = 1.0 / (1 << frag.scale) as f32;
        frag.positions
            .iter()
            .map(|p| wld_i16_pos_to_gd(&p, scale))
            .collect::<PackedVector3Array>()
    }


    fn normals(&self) -> PackedVector3Array {
        self.get_frag()
            .vertex_normals
            .iter()
            .map(|p| Vector3::new(p.0 as f32 / 127., p.2 as f32 / 127., p.1 as f32 / 127.))
            .collect::<PackedVector3Array>()
    }

 
    fn vertex_colors(&self) -> PackedColorArray {
        self.get_frag()
            .vertex_colors
            .iter()
            .map(u32_to_color)
            .collect::<PackedColorArray>()
    }


    fn uvs(&self) -> PackedVector2Array {
        self.get_frag()
            .texture_coordinates
            .iter()
            .map(|p| Vector2::new(1.0 - p.0 as f32 / 256. * -1., 1.0 - p.1 as f32 / 256.))
            .collect::<PackedVector2Array>()
    }

 
    fn bone_indices(&self) -> PackedInt32Array {
        self.get_frag()
            .skin_assignment_groups
            .iter()
            .flat_map(|(num_verts, bone_idx)| {
                vec![*bone_idx as i32, 0, 0, 0].repeat(*num_verts as usize)
            })
            .collect()
    }

   
    fn bone_weights(&self) -> PackedFloat32Array {
        self.get_frag()
            .skin_assignment_groups
            .iter()
            .flat_map(|(num_verts, _bone_idx)| vec![1., 0., 0., 0.].repeat(*num_verts as usize))
            .collect()
    }


    fn face_material_groups(&self) -> Array<VariantArray> {
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

 
    fn indices(&self) -> PackedInt32Array {
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

   
    fn collision_vertices(&self) -> PackedVector3Array {
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

    
        // pub fn faces(&self) -> Array<VariantArray> {
        //     let frag = self.get_frag();
        //     let scale = 1.0 / (1 << frag.scale) as f32;
        //     frag.faces
        //         .iter()
        //         .map(|face| {
        //             let mut arr = VariantArray::new();
        //             arr.push(Variant::from(face.flags));
        //             arr.push(Variant::from(PackedVector3Array::from(&[
        //                 wld_i16_pos_to_gd(&frag.positions[face.vertex_indexes.2 as usize], scale),
        //                 wld_i16_pos_to_gd(&frag.positions[face.vertex_indexes.1 as usize], scale),
        //                 wld_i16_pos_to_gd(&frag.positions[face.vertex_indexes.0 as usize], scale),
        //             ])));
        //             arr
        //         })
        //         .collect()
        // }

    #[cfg(feature = "serde")]
    fn as_dict(&self) -> Dictionary {
        let frag = self.get_frag();
        let wld = self.get_wld();
        frag_to_dict(wld, frag)
    }
    
}

impl DmSprite2Provider {
    fn get_frag(&self) -> &DmSpriteDef2 {
        self.fragment
            .as_ref()
    }

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


}

struct DmSpriteProvider {
    fragment: ArcRef<WldDoc, DmSpriteDef>
 }
 
 impl MeshProvider for DmSpriteProvider {
    
    fn name(&self) -> GString {
        GString::from(
            self.get_wld()
                .get_string(self.get_frag().name_reference)
                .expect("Failed to get string from WLD!"),
        )
    }

     fn flags(&self) -> u32 {
         self.fragment.as_ref().flags
     }

     fn get_wld(&self) -> &Arc<WldDoc> {
        self.fragment
            .as_owner()
    }

    fn center(&self) -> Vector3 {
        wld_f32_pos_to_gd(&self.get_frag().center)
    }


    fn vertices(&self) -> PackedVector3Array {
        self.get_frag().vertices.iter().map(|p| wld_f32_pos_to_gd(p)).collect()
    }


    fn normals(&self) -> PackedVector3Array {
        self.get_frag()
            .vertex_normals
            .iter()
            .map(|p| Vector3::new(p.0 as f32 / 127., p.2 as f32 / 127., p.1 as f32 / 127.))
            .collect::<PackedVector3Array>()
    }

 
    fn vertex_colors(&self) -> PackedColorArray {
        self.get_frag()
            .vertex_colors
            .iter()
            .map(u32_to_color)
            .collect::<PackedColorArray>()
    }


    fn uvs(&self) -> PackedVector2Array {
        self.get_frag()
            .texture_coordinates
            .iter()
            .map(|p| Vector2::new(1.0 - p.0 as f32 / 256. * -1., 1.0 - p.1 as f32 / 256.))
            .collect::<PackedVector2Array>()
    }

 
    fn bone_indices(&self) -> PackedInt32Array {
        self.get_frag()
            .skin_assignment_groups
            .iter()
            .flat_map(|(num_verts, bone_idx)| {
                vec![*bone_idx as i32, 0, 0, 0].repeat(*num_verts as usize)
            })
            .collect()
    }

   
    fn bone_weights(&self) -> PackedFloat32Array {
        self.get_frag()
            .skin_assignment_groups
            .iter()
            .flat_map(|(num_verts, _bone_idx)| vec![1., 0., 0., 0.].repeat(*num_verts as usize))
            .collect()
    }


    fn face_material_groups(&self) -> Array<VariantArray> {
        let wld = self.get_wld();
        let materials = self.materials();
        let mut pos = 0;
        let frag = self.get_frag();
        match &frag.face_material_groups {
            Some(face_material_groups) => {
                face_material_groups
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
            None => Array::new()
        }
    }

 
    fn indices(&self) -> PackedInt32Array {
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

   
    fn collision_vertices(&self) -> PackedVector3Array {
        let frag = self.get_frag();
        frag.faces
            .iter()
            .filter(|face| face.flags & 0x10 == 0)
            .flat_map(|face| {
                vec![
                    wld_f32_pos_to_gd(&frag.vertices[face.vertex_indexes.2 as usize]),
                    wld_f32_pos_to_gd(&frag.vertices[face.vertex_indexes.1 as usize]),
                    wld_f32_pos_to_gd(&frag.vertices[face.vertex_indexes.0 as usize]),
                ]
            })
            .collect()
    }

    #[cfg(feature = "serde")]
    fn as_dict(&self) -> Dictionary {
        let frag = self.get_frag();
        let wld = self.get_wld();
        frag_to_dict(wld, frag)
    }
 }

 impl DmSpriteProvider {
    fn get_frag(&self) -> &DmSpriteDef {
        self.fragment
            .as_ref()
    }

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

}

 #[derive(GodotClass)]
#[class(init)]
pub struct S3DMesh {
    base: Base<RefCounted>,
    provider: Option<Box<dyn MeshProvider>>,
}



impl S3DFragment for S3DMesh {
    fn load(&mut self, wld: &Arc<WldDoc>, index: u32) {
        let fragment = wld.as_ref().at(index as usize - 1).unwrap();
        let provider: Box<dyn MeshProvider> = match fragment {
            FragmentType::DmSpriteDef(_) => {
                Box::new(DmSpriteProvider {
                    fragment: create_fragment_ref(wld.clone(), index)
                })
            },
            FragmentType::DmSpriteDef2(_) => {
                Box::new(DmSprite2Provider {
                    fragment: create_fragment_ref(wld.clone(), index)
                })
            },
            _ => panic!("S3DMesh trying to wrap a non-mesh fragment!")
        };
        self.provider = Some(provider);
    }
}

#[godot_api]
impl S3DMesh {


    #[func]
    pub fn name(&self) -> GString {
        self.get_provider().name()
    }

    #[func]
    pub fn flags(&self) -> u32 {
        self.get_provider().flags()
    }

    #[func]
    pub fn center(&self) -> Vector3 {
        self.get_provider().center()
    }

    /// Returns the vertex positions of the mesh, converted into Godot format.
    #[func]
    pub fn vertices(&self) -> PackedVector3Array {
        self.get_provider().vertices()
    }

    /// Returns the vertex normals of the mesh, converted into Godot format.
    #[func]
    pub fn normals(&self) -> PackedVector3Array {
        self.get_provider().normals()
    }

    /// Returns the vertex colors of the mesh (if present), converted into Godot format.
    /// Note that for re-usable actors such as trees, the vertex colors are not part of the mesh definition, but instead part of the object definition.
    #[func]
    pub fn vertex_colors(&self) -> PackedColorArray {
        self.get_provider().vertex_colors()
    }

    /// Returns the UV coordinates of the mesh, converted into Godot format.
    #[func]
    pub fn uvs(&self) -> PackedVector2Array {
        self.get_provider().uvs()
    }

    /// Returns skin assignment groups, converted into Godot format.
    /// The Godot format requires that there are 4 bone indices per vertex, but we only use the first of the four.
    #[func]
    pub fn bone_indices(&self) -> PackedInt32Array {
        self.get_provider().bone_indices()
    }

    /// Returns a bone weights array that matches the `bone_indices` array.
    /// The Godot format requires that there are 4 bone indices per vertex, but we only use the first of the four.
    #[func]
    pub fn bone_weights(&self) -> PackedFloat32Array {
        self.get_provider().bone_weights()
    }

    /// Returns an array of material groups.  
    /// Material groups are two-tuples.  The first element is the name of the material.  
    /// The second element is the array of indices for the polygons that use this material.
    ///
    /// Invisible materials are skipped entirely.
    #[func]
    pub fn face_material_groups(&self) -> Array<VariantArray> {
        self.get_provider().face_material_groups()
    }

    /// Get all the indices that form polygons of the mesh.
    /// NOTE: This should not normally be used if you wish to actually apply materials to surfaces.
    /// To do so, you must get the indices of each material group, and add each material group as a separate surface.
    #[func]
    pub fn indices(&self) -> PackedInt32Array {
        self.get_provider().indices()
    }

    /// Construct an array of Vector3s that represent the concave collision mesh for this mesh
    #[func]
    pub fn collision_vertices(&self) -> PackedVector3Array {
        self.get_provider().collision_vertices()
    }
    #[cfg(feature = "serde")]
    #[func]
    pub fn as_dict(&self) -> Dictionary {
        self.get_provider().as_dict()
    }
}

impl S3DMesh {

    fn get_provider(&self) -> &Box<dyn MeshProvider> {
        self.provider.as_ref().unwrap()
    }

    pub fn from_reference(wld: &Arc<WldDoc>, mesh_reference: &DmSprite) -> Option<Gd<Self>> {
        match mesh_reference.reference {
            FragmentRef::Index(index, _) => {
                let fragment = wld.at(index as usize - 1).unwrap();
                match fragment {
                    FragmentType::DmSpriteDef2(_) => Some(gd_from_frag_type::<S3DMesh>(wld, index)),
                    FragmentType::DmSpriteDef(_) => Some(gd_from_frag_type::<S3DMesh>(wld, index)),
                    _ => None,
                }
            }
            FragmentRef::Name(_, _) => None,
        }
    }

    
}
