mod actordef;
mod actorinst;
mod material;
mod mesh;
mod skeleton;
pub use actordef::*;
pub use actorinst::*;
use godot::engine::RefCounted;
use godot::prelude::*;
use libeq::wld::parser::{Fragment, WldDoc};
pub use material::*;
pub use mesh::*;
use owning_ref::ArcRef;
pub use skeleton::*;
use std::sync::Arc;

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
#[class(init, base=RefCounted)]
pub struct EQFragmentUnknown {
    #[base]
    base: Base<RefCounted>,
    /// Index within the WLD - note that indices begin at 1.
    index: u32,
    /// Reference to the WLD that contains this fragment
    wld: Option<Arc<WldDoc>>,
}

impl S3DFragment for EQFragmentUnknown {
    fn load(&mut self, wld: &Arc<WldDoc>, index: u32) {
        self.index = index;
        self.wld = Some(wld.clone())
    }
}

/// A temporary placeholder for unsupported fragment types.
/// This should, in the future, provide the ability to get the data from the fragment using JSON
#[godot_api]
impl EQFragmentUnknown {}
