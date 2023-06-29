# godot-eqloader

A Rust-based Godot 4 extension, using [the gdextension crate](https://github.com/godot-rust/gdextension), for loading EverQuest assets at runtime from the original data files. It depends heavily on the [libeq crate](https://github.com/cjab/libeq), and is really a thin wrapper around it with a translation layer into Godot's objects and conventions.

The goal of this plugin is to provide a flexible, low-level API for using EverQuest data files in whatever way the user wishes, in GDScript. It is up to the end-user to create any kind of "engine" to serve the purposes of their game.

# Status

This plugin has only been tested in Godot `4.0.3`, with EQ Phase 4 Beta and Final (original release) files.

This project is in an experimental state and its API may change drastically. At the moment it only supports a tiny fraction of the EQ data, but I am working on supporting all of the fragments that I feel would be needed to create a rough fascimile of Everquest in Godot. Any contributions or critiques are welcome!

# Example Project

The example project in this repository attempts to show, in the most concise way possible, how to build Godot assets with this plugin. For a more advanced example, I will be setting up a separate repository. To run the example project, you must provide `.s3d` files yourself, and either put them in the `eq_data` folder, or create an environment variable called `EQDATA` that points to a folder containing `.s3d` files.

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
