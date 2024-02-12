mod actordef;
mod actorinst;
mod hiersprite;
mod material;
mod mesh;
pub use actordef::*;
pub use actorinst::*;
use godot::engine::RefCounted;
use godot::prelude::*;
pub use hiersprite::*;
use libeq_wld::parser::{Fragment, FragmentType, WldDoc};
pub use material::*;
pub use mesh::*;
use owning_ref::ArcRef;
use std::sync::Arc;

#[cfg(feature = "serde")]
fn frag_to_dict<T: 'static + Fragment + serde::ser::Serialize>(wld: &WldDoc, fragment: &T) -> Dictionary {
    use godot::engine::Json;
    let frag = fragment.as_any().downcast_ref::<T>().unwrap();
    let json_string =
        serde_json::to_string(frag).unwrap();
    let mut json = Json::new_gd();
    let _result = json.parse(GString::from(json_string));
    let variant = json.get_data();
    let mut d = variant.to::<Dictionary>();
    d.set("type_id", frag.type_id());
    d.set("name", wld.get_string(*frag.name_ref()).unwrap());
    d
}

/// Create a reference to a particular fragment by pairing it with its parent WLD in an OwnedRef.
fn create_fragment_ref<T: 'static + Fragment>(wld: Arc<WldDoc>, index: u32) -> ArcRef<WldDoc, T> {
    ArcRef::new(wld).map(|wld| {
        wld.at((index - 1) as usize)
            .expect(format!("Fragment index {index} is out of bounds!").as_str())
            .as_any()
            .downcast_ref()
            .expect(format!("Fragment at index {index} is not of the requested type!").as_str())
    })
}

pub trait S3DFragment {
    fn load(&mut self, wld: &Arc<WldDoc>, index: u32);
}

#[derive(GodotClass)]
#[class(init)]
pub struct S3DUnknownFragment {
    base: Base<RefCounted>,
    /// Index within the WLD - note that indices begin at 1.
    index: u32,
    /// Reference to the WLD that contains this fragment
    wld: Option<Arc<WldDoc>>,
}

impl S3DFragment for S3DUnknownFragment {
    fn load(&mut self, wld: &Arc<WldDoc>, index: u32) {
        self.index = index;
        self.wld = Some(wld.clone())
    }
}



/// A temporary placeholder for unsupported fragment types.
#[godot_api]
impl S3DUnknownFragment {

    #[cfg(feature = "serde")]
    #[func]
    pub fn as_dict(&self) -> Dictionary {
        let wld = self.get_wld().as_ref();
        let fragment_type = wld.at(self.index as usize - 1).unwrap();
        match fragment_type {
            FragmentType::DmSpriteDef(f) => frag_to_dict(wld, f),
            FragmentType::AmbientLight(f) => frag_to_dict(wld, f),
            FragmentType::BlitSpriteDef(f) => frag_to_dict(wld, f),
            FragmentType::BlitSprite(f) => frag_to_dict(wld, f),
            FragmentType::Region(f) => frag_to_dict(wld, f),
            FragmentType::WorldTree(f) => frag_to_dict(wld, f),
            FragmentType::Sprite3DDef(f) => frag_to_dict(wld, f),
            FragmentType::Sprite3D(f) => frag_to_dict(wld, f),
            FragmentType::GlobalAmbientLightDef(f) => frag_to_dict(wld, f),
            FragmentType::Sprite4D(f) => frag_to_dict(wld, f),
            FragmentType::Sprite4DDef(f) => frag_to_dict(wld, f),
            FragmentType::PointLight(f) => frag_to_dict(wld, f),
            FragmentType::LightDef(f) => frag_to_dict(wld, f),
            FragmentType::Light(f) => frag_to_dict(wld, f),
            FragmentType::MaterialDef(f) => frag_to_dict(wld, f),
            FragmentType::MaterialPalette(f) => frag_to_dict(wld, f),
            FragmentType::DmSpriteDef2(f) => frag_to_dict(wld, f),
            FragmentType::DmTrackDef2(f) => frag_to_dict(wld, f),
            FragmentType::DmTrack(f) => frag_to_dict(wld, f),
            FragmentType::DmSprite(f) => frag_to_dict(wld, f),
            FragmentType::TrackDef(f) => frag_to_dict(wld, f),
            FragmentType::Track(f) => frag_to_dict(wld, f),
            FragmentType::ActorDef(f) => frag_to_dict(wld, f),
            FragmentType::Actor(f) => frag_to_dict(wld, f),
            FragmentType::ParticleSprite(f) => frag_to_dict(wld, f),
            FragmentType::ParticleSpriteDef(f) => frag_to_dict(wld, f),
            FragmentType::ParticleCloudDef(f) => frag_to_dict(wld, f),
            FragmentType::DefaultPaletteFile(f) => frag_to_dict(wld, f),
            FragmentType::PolyhedronDef(f) => frag_to_dict(wld, f),
            FragmentType::Polyhedron(f) => frag_to_dict(wld, f),
            FragmentType::Zone(f) => frag_to_dict(wld, f),
            FragmentType::HierarchicalSpriteDef(f) => frag_to_dict(wld, f),
            FragmentType::HierarchicalSprite(f) => frag_to_dict(wld, f),
            FragmentType::SphereList(f) => frag_to_dict(wld, f),
            FragmentType::SphereListDef(f) => frag_to_dict(wld, f),
            FragmentType::SimpleSpriteDef(f) => frag_to_dict(wld, f),
            FragmentType::BmInfo(f) => frag_to_dict(wld, f),
            FragmentType::BmInfoRtk(f) => frag_to_dict(wld, f),
            FragmentType::SimpleSprite(f) => frag_to_dict(wld, f),
            FragmentType::Sprite2DDef(f) => frag_to_dict(wld, f),
            FragmentType::Sprite2D(f) => frag_to_dict(wld, f),
            FragmentType::DmTrackDef(f) => frag_to_dict(wld, f),
            FragmentType::DmRGBTrackDef(f) => frag_to_dict(wld, f),
            FragmentType::DmRGBTrack(f) => frag_to_dict(wld, f),
            FragmentType::WorldVertices(f) => frag_to_dict(wld, f),
            FragmentType::Sphere(f) => frag_to_dict(wld, f),
            FragmentType::DirectionalLight(f) => frag_to_dict(wld, f),
        }
    }
}

impl S3DUnknownFragment {
    fn get_wld(&self) -> &Arc<WldDoc> {
        self.wld
            .as_ref()
            .expect("Failed to get WLD reference!")
    }
}
