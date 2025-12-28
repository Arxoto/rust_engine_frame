use godot::prelude::*;

use crate::cores::unify_type::{FixedName, FixedString};

/// same as [`FixedName`]
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct FixedNameWrapper(pub StringName);

impl From<&str> for FixedNameWrapper {
    fn from(value: &str) -> Self {
        FixedNameWrapper(StringName::from(value))
    }
}

impl From<StringName> for FixedNameWrapper {
    fn from(value: StringName) -> Self {
        FixedNameWrapper(value)
    }
}

impl From<FixedNameWrapper> for StringName {
    fn from(value: FixedNameWrapper) -> Self {
        value.0
    }
}

impl FixedName for FixedNameWrapper {}

/// same as [`FixedString`]
#[derive(PartialEq, Eq, Hash, Clone, Debug, Default)]
pub struct FixedStringWrapper(pub GString);

impl From<&str> for FixedStringWrapper {
    fn from(value: &str) -> Self {
        FixedStringWrapper(GString::from(value))
    }
}

impl From<GString> for FixedStringWrapper {
    fn from(value: GString) -> Self {
        FixedStringWrapper(value)
    }
}

impl From<FixedStringWrapper> for GString {
    fn from(value: FixedStringWrapper) -> Self {
        value.0
    }
}

impl FixedString for FixedStringWrapper {}
