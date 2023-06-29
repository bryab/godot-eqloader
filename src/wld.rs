use crate::fragments::{EQFragmentUnknown, S3DFragment, S3DMaterial, S3DMesh};
use godot::engine::RefCounted;
use godot::obj::cap::GodotInit;
use godot::obj::dom::UserDomain;
use godot::prelude::*;
use libeq::wld::parser::{Fragment, FragmentType, MaterialFragment, MeshFragment, WldDoc};
use std::sync::Arc;

/// Creates one of the GodotClass wrappers around the given Fragment
pub fn create_fragment<T: S3DFragment + GodotInit<Declarer = UserDomain>>(
    wld: &Arc<WldDoc>,
    index: u32,
) -> Gd<T> {
    let mut obj = Gd::<T>::new_default();
    obj.bind_mut().load(wld, index);
    obj
}

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
pub struct S3DWld {
    #[base]
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
        T: S3DFragment + GodotInit<Declarer = UserDomain>,
        T2: 'static + Fragment,
    >(
        &self,
    ) -> Array<Gd<T>> {
        let wld = self.wld.as_ref().unwrap();
        wld.iter()
            .enumerate()
            .filter_map(|(index, fragment)| {
                let fragment = fragment.as_any().downcast_ref::<T2>();
                fragment.and_then(|_| Some(create_fragment::<T>(wld, index as u32 + 1)))
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
    pub fn get_meshes(&self) -> Array<Gd<S3DMesh>> {
        self.build_fragment_type_array::<S3DMesh, MeshFragment>()
    }

    /// Returns an Array of all the Meshes in the WLD
    /// This should really only be used for Zone WLDS; for objects, characters etc you should get get_actors
    #[func]
    pub fn get_materials(&self) -> Array<Gd<S3DMaterial>> {
        self.build_fragment_type_array::<S3DMaterial, MaterialFragment>()
    }

    #[func]
    pub fn fragment_count(&self) -> u32 {
        self.get_wld().fragment_count() as u32
    }

    #[func]
    pub fn get_fragment(&self, index: u32) -> Variant {
        let wld = self.get_wld();

        let fragment_type = match wld.at((index - 1) as usize) {
            Some(myval) => myval,
            None => {
                godot_error!("Invalid WLD index: {index}");
                return Variant::nil();
            }
        };

        match fragment_type {
            FragmentType::Mesh(_) => Variant::from(create_fragment::<S3DMesh>(wld, index)),

            FragmentType::Material(_) => Variant::from(create_fragment::<S3DMaterial>(wld, index)),
            _ => Variant::from(create_fragment::<EQFragmentUnknown>(wld, index)),
        }
    }
}