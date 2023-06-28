# godot-eqloader

A Rust-based Godot 4 plugin, using [the gdextension crate](https://github.com/godot-rust/gdextension), for loading EverQuest assets at runtime, from the original EverQuest data files.

This plugin depends heavily on the [libeq crate](https://github.com/cjab/libeq), and is really a thin wrapper around it with a translation layer into Godot's objects and conventions.

The goal of this plugin is to provide a flexible, low-level API for using EverQuest data files in whatever way the user wishes, in GDScript. It is up to the end-user to create any kind of "engine" to serve the purposes of their game. I am just having fun with this - and contributions are welcome!

This project is in an experimental state and its API may change drastically. At the moment it only supports a tiny fraction of the EQ data, but I am working on supporting all of the fragments that I feel would be needed to create a rough fascimile of Everquest in Godot. As I go about this, I will update the example project. This plugin has only been tested in Godot `4.0.3`

Check out the example project to see how this data can be used to build Godot objects such as Meshes and Materials. You must provide the actual S3D files - copy them into the `eq_data` folder.

# Building

First, make sure to follow the setup instructions for [the gdextension crate](https://github.com/godot-rust/gdextension). At the time of this writing, that includes setting the `GODOT4_BIN` environment variable to point to your Godot4 binary.
Then...

`cargo build`

Or for a release build:

`cargo build --release`

# Installation

After building, copy `godot_eqloader.dll` (or `.dylib` or `.so`) from `./target/release/` into your project directory somewhere.

Create a file called `EQLoader.gdextension` next to it that looks something like this:

```
[configuration]
entry_symbol = "gdext_rust_init"

[libraries]
linux.64 = "res://./godot_eqloader.so"
macos.64 = "res://./godot_eqloader.dylib"
windows.64 = "res://./godot_eqloader.dll"
```
