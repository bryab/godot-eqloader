use godot::engine::{Node};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct EQArchiveLoader {

    #[base]
    base: Base<Node>,
}

#[godot_api]
impl GodotExt for EQArchiveLoader {
    fn init(base: Base<Node>) -> Self {
        EQArchiveLoader {
            base
        }
    }

    fn ready(&mut self) {
        godot_print!("EQArchiveLoader Ready!");
    }
}
