# godot-eqloader

A Godot 4 plugin, using [![the Rust gdextension crate](https://github.com/godot-rust/gdextension), for loading EverQuest assets at runtime, from the original EverQuest data files.

This plugin depends heavily on the `[![libeq crate](https://github.com/cjab/libeq)].

The goal of this plugin is to provide a flexible, low-level API for using EverQuest data files in whatever way the user wishes.  It is up to the end-user to create any kind of "engine" to serve the purposes of their game.  I am just having fun with this - and contributions are welcome!