use godot::prelude::*;
struct EQLoader;

mod loader;

#[gdextension]
unsafe impl ExtensionLibrary for EQLoader {}
