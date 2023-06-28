use godot::prelude::*;
struct EQLoader;

mod archive;
mod fragments;
mod loader;
mod util;
mod wld;
#[gdextension]
unsafe impl ExtensionLibrary for EQLoader {}
