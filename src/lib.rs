pub mod base_lib;

pub mod cores;

// Ability System Component
pub mod attrs;
pub mod effects;

// Combat System Component
#[cfg(feature = "commonimpl")]
pub mod combats;

#[cfg(feature = "godotext")]
pub mod godot_ext_impl;
