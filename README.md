# godot-eqloader

A Rust-based Godot 4 extension, using [the gdextension crate](https://github.com/godot-rust/gdextension), for loading EverQuest assets at runtime from the original data files. It depends heavily on the [libeq crate](https://github.com/cjab/libeq), and is really a thin wrapper around it with a translation layer into Godot's objects and conventions.

# Example Project

The example project in this repository attempts to show, in the most concise way possible, how to build Godot assets with this plugin. For a more advanced example, I will be setting up a separate repository. To run the example project, you must provide `.s3d` files yourself, and either put them in the `eq_data` folder, or create an environment variable called `EQDATA` that points to a folder containing `.s3d` files.

To run the example, first build the GDExtension:
`cargo build`
Then you can either open the example project in the Editor, or run it in the command line:
`godot4 --path ./example/EQLoaderExample`

There are some command line options. The default action is to load a random zone and character file.

Load a specific zone, including its characters:
`godot4 --path ./example/EQLoaderExample -- --zone misty`

Load a character file only:
`godot4 --path ./example/EQLoaderExample -- --chr global_chr`

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
