pub trait FixedName: Eq + std::hash::Hash + Clone {}

impl FixedName for String {}
impl FixedName for &str {}

pub struct FixedNameBox<S>
where
    S: FixedName,
{
    the_name: S,
}

impl<S: FixedName> FixedNameBox<S> {
    pub fn new<T: Into<S>>(w: T) -> Self {
        Self { the_name: w.into() }
    }

    pub fn get_name(&self) -> &S {
        &self.the_name
    }
}
