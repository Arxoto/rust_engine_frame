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

impl Into<StringName> for &FixedNameWrapper {
    fn into(self) -> StringName {
        self.0.clone()
    }
}

impl FixedName for FixedNameWrapper {
    fn from_str(s: &str) -> Self {
        FixedNameWrapper(s.into())
    }
}

/// same as [`FixedString`]
#[derive(PartialEq, Eq, Hash, Clone, Debug, Default)]
pub struct FixedStringWrapper(pub GString);

impl From<GString> for FixedStringWrapper {
    fn from(value: GString) -> Self {
        FixedStringWrapper(value)
    }
}

impl Into<GString> for &FixedStringWrapper {
    fn into(self) -> GString {
        self.0.clone()
    }
}

impl FixedString for FixedStringWrapper {}
