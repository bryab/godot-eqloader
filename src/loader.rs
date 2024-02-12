use crate::archive::EQArchive;
use godot::engine::{RefCounted, ProjectSettings};
use godot::prelude::*;
#[derive(GodotClass)]
#[class(init)]
pub struct EQArchiveLoader {
    base: Base<RefCounted>,
}

#[godot_api]
impl EQArchiveLoader {
    /// Load an Everquest .s3d archive, returning an EQArchive object.
    #[func]
    fn load_archive(&self, filename: GString) -> Gd<EQArchive> {
        let filename = String::from(ProjectSettings::singleton().globalize_path(filename));
        let mut obj: Gd<EQArchive> = Gd::default();
        obj.bind_mut().load(&filename);
        obj
    }
}
