//! see https://godot-rust.github.io/book/intro/hello-world.html
//!
//! When exporting your project, you need to use paths inside res://.
//! Outside paths like .. are not supported.

use godot::prelude::*;

mod adapter;

mod attr_impl;

/// see <https://godot-rust.github.io/book/intro/hello-world.html>
///
/// write `res://HelloWorld.gdextension` as follow
///
/// ```gdextension
/// [configuration]
/// entry_symbol = "gdext_rust_init"
/// compatibility_minimum = 4.1
/// reloadable = true
///
/// [libraries]
/// linux.debug.x86_64 =     "res://../rust/target/debug/lib{YourCrate}.so"
/// linux.release.x86_64 =   "res://../rust/target/release/lib{YourCrate}.so"
/// windows.debug.x86_64 =   "res://../rust/target/debug/{YourCrate}.dll"
/// windows.release.x86_64 = "res://../rust/target/release/{YourCrate}.dll"
/// macos.debug =            "res://../rust/target/debug/lib{YourCrate}.dylib"
/// macos.release =          "res://../rust/target/release/lib{YourCrate}.dylib"
/// macos.debug.arm64 =      "res://../rust/target/debug/lib{YourCrate}.dylib"
/// macos.release.arm64 =    "res://../rust/target/release/lib{YourCrate}.dylib"
/// ```
///
/// write `res://.godot/extension_list.cfg` as `res://HelloWorld.gdextension`
struct ExSystemComponent;

#[gdextension]
unsafe impl ExtensionLibrary for ExSystemComponent {}
