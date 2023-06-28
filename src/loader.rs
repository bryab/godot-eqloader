use crate::archive::EQArchive;
use godot::engine::Node;
use godot::prelude::*;
#[derive(GodotClass)]
#[class(init,base=Node)] // FIXME: I don't want this to be a Node unless it must be.  This should just be a globally available static class somehow?
pub struct EQArchiveLoader {
    #[base]
    base: Base<Node>,
}

#[godot_api]
impl EQArchiveLoader {
    /// Load an Everquest .s3d archive, returning an EQArchive object.
    #[func]
    fn load_archive(&self, filename: GodotString) -> Gd<EQArchive> {
        let mut obj: Gd<EQArchive> = Gd::new_default();
        obj.bind_mut().load(&String::from(&filename));
        obj
    }
}
