use crate::fragments::{
    S3DUnknownFragment, S3DActorDef, S3DActorInstance, S3DFragment, S3DHierSprite, S3DMaterial,
    S3DMesh,
};
use godot::classes::RefCounted;
use godot::obj::bounds::{DeclUser, MemRefCounted};
use godot::obj::cap::GodotDefault;
use godot::prelude::*;
use libeq_wld::parser::{
    Actor, ActorDef, Fragment, FragmentType, HierarchicalSpriteDef,
    MaterialDef, WldDoc,
};
use std::sync::Arc;

/// Attempts to create a S3D Godot class from the given fragment index - and assert it is of the given type.
// FIXME: I feel this should return Option - it should fail if the given index is not of the correct type.
pub fn gd_from_frag_type<
    T: S3DFragment + GodotDefault<Memory = MemRefCounted, Declarer = DeclUser>,
>(
    wld: &Arc<WldDoc>,
    index: u32,
) -> Gd<T> {
    let mut obj = Gd::<T>::default();
    obj.bind_mut().load(wld, index);
    obj
}
/// Attempts to create a S3D Godot class from the given fragment index, without knowing its type, returning a Variant.
/// Note that the index supplied is the kind that starts at 1, not 0
pub fn gd_from_frag(wld: &Arc<WldDoc>, index: u32) -> Variant {
    let fragment_type = match wld.at((index - 1) as usize) {
        Some(myval) => myval,
        None => {
            godot_error!("Invalid WLD index: {index}");
            return Variant::nil();
        }
    };

    match fragment_type {
        FragmentType::DmSprite(mesh_reference) => {
            match S3DMesh::from_reference(wld, mesh_reference) {
                Some(mesh) => Variant::from(mesh),
                None => Variant::nil(),
            }
        }
        FragmentType::DmSpriteDef2(_) => Variant::from(gd_from_frag_type::<S3DMesh>(wld, index)),
        FragmentType::MaterialDef(_) => Variant::from(gd_from_frag_type::<S3DMaterial>(wld, index)),
        FragmentType::ActorDef(_) => Variant::from(gd_from_frag_type::<S3DActorDef>(wld, index)),
        FragmentType::Actor(_) => Variant::from(gd_from_frag_type::<S3DActorInstance>(wld, index)),
        FragmentType::HierarchicalSprite(reference) => 
            S3DHierSprite::from_reference(wld, reference).and_then(|frag| Some(Variant::from(frag))).unwrap_or_default(),
        FragmentType::HierarchicalSpriteDef(_) => {
            Variant::from(gd_from_frag_type::<S3DHierSprite>(wld, index))
        }
        _ => Variant::from(gd_from_frag_type::<S3DUnknownFragment>(wld, index)),
    }
}

#[derive(GodotClass)]
#[class(init)]
pub struct S3DWld {
    base: Base<RefCounted>,
    wld: Option<Arc<WldDoc>>,
}

impl S3DWld {
    pub fn load(&mut self, data: Vec<u8>) {
        //fs::write("tmp.wld", &data).expect("Unable to write file");
        self.wld = match WldDoc::parse(&data[..]) {
            Ok(wld_doc) => Some(Arc::new(wld_doc)),
            Err(err) => panic!("Failed to parse Wld: {:?}", err),
        };
    }

    fn build_fragment_type_array<
        T: S3DFragment + GodotDefault<Memory = MemRefCounted, Declarer = DeclUser>,
        T2: 'static + Fragment,
    >(
        &self,
    ) -> Array<Gd<T>> {
        let wld = self.wld.as_ref().unwrap();
        wld.iter()
            .enumerate()
            .filter_map(|(index, fragment)| {
                let fragment = fragment.as_any().downcast_ref::<T2>();
                fragment.and_then(|_| Some(gd_from_frag_type::<T>(wld, index as u32 + 1)))
            })
            .collect()
    }

    fn get_wld(&self) -> &Arc<WldDoc> {
        self.wld
            .as_ref()
            .expect("This class must be initialized with the load() function.")
    }
}

#[godot_api]
impl S3DWld {
    /// Returns an Array of all the Meshes in the WLD
    /// This should really only be used for Zone WLDS; for objects, characters etc you should get get_actors
    #[func]
    pub fn meshes(&self) -> Array<Gd<S3DMesh>> {
        let wld = self.wld.as_ref().unwrap();
        wld.iter()
            .enumerate()
            .filter_map(|(index, fragment)| {
                match fragment.as_ref() {
                    FragmentType::DmSpriteDef(_) => Some(gd_from_frag_type(wld, index as u32 + 1)),
                    FragmentType::DmSpriteDef2(_) => Some(gd_from_frag_type(wld, index as u32 + 1)),
                    _ => None

                }
            })
            .collect()
    }

    #[func]
    pub fn materials(&self) -> Array<Gd<S3DMaterial>> {
        self.build_fragment_type_array::<S3DMaterial, MaterialDef>()
    }

    #[func]
    pub fn actordefs(&self) -> Array<Gd<S3DActorDef>> {
        self.build_fragment_type_array::<S3DActorDef, ActorDef>()
    }

    #[func]
    pub fn actorinstances(&self) -> Array<Gd<S3DActorInstance>> {
        self.build_fragment_type_array::<S3DActorInstance, Actor>()
    }

    #[func]
    pub fn hiersprites(&self) -> Array<Gd<S3DHierSprite>> {
        self.build_fragment_type_array::<S3DHierSprite, HierarchicalSpriteDef>()
    }

    #[func]
    pub fn fragment_count(&self) -> u32 {
        self.get_wld().fragment_count() as u32
    }

    #[func]
    pub fn at(&self, index: u32) -> Variant {
        let wld = self.get_wld();
        gd_from_frag(&wld, index)
    }
}
