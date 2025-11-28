use godot::prelude::*;

use crate::cores::unify_type::{FixedName, FixedString};

/// same as [`FixedName`]
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct FixedNameWrapper(pub StringName);

impl From<StringName> for FixedNameWrapper {
    fn from(value: StringName) -> Self {
        FixedNameWrapper(value)
    }
}

impl From<&FixedNameWrapper> for StringName {
    fn from(val: &FixedNameWrapper) -> Self {
        val.0.clone()
    }
}

impl FixedName for FixedNameWrapper {}

/// same as [`FixedString`]
#[derive(PartialEq, Eq, Hash, Clone, Debug, Default)]
pub struct FixedStringWrapper(pub GString);

impl From<GString> for FixedStringWrapper {
    fn from(value: GString) -> Self {
        FixedStringWrapper(value)
    }
}

impl From<&FixedStringWrapper> for GString {
    fn from(val: &FixedStringWrapper) -> Self {
        val.0.clone()
    }
}

impl FixedString for FixedStringWrapper {}
