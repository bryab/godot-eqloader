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

# Current State

The following features are currently implemented:

WLD fragment access

- **S3DWld** - Provides methods for getting all the fragments described below
- **S3DMesh** - A wrapper around `DMSPRITEDEF` and `DMSPRITEDEF2`, which represent all meshes
- **S3DMaterial** - A wrapper around `MATERIALDEF` and its `SIMPLESPRITEDEF` and `BMINFO` references, which represent materials and their texture properties
- **S3DActorDef** - A wrapper around `ACTORDEF`, which represents actors in the world such as placeable objects and characters
- **S3DActorInstance** - A wrapper around `ACTOR`, which represents instances of `ACTORDEFS` in a zone
- **S3DHierSprite** - A wrapper around `DMHIERARCHICALSPRITE`, which represents skeleton-based objects such as characters, and their animations (`TRACK` and `TRACKDEF` fragments)
- **S3DUnknownFragment** - A wrapper around unsupported fragments, for analysis.  To actually look at the fragment data, see the "Extra Features" section below.

Archive access

- Loading `.wld` files as `S3DWld` objects (described above)
- Loading `.bmp` files as Godot `Images` and `ImageTextures`
- Loading `.wav` file as Godot `AudioStreamWAV`

The following features may be supported in the future, and any help is welcome:

- Regions (For the sole purpose of detecting whether the player is in a particular special region such as water, zoneline, etc)
- Lights
- Blitsprites

 # Extra Features

 This library can be compiled with a `serde` feature, which adds a new method to all fragments: `as_dict`.  This returns a serde-serialized representation of the underlying raw fragment data as a Godot `Dictionary`, for analysis.  For fragments that do not have a wrapper, you can get them and look at their data with `wld.at(fragment_index).as_dict()`.