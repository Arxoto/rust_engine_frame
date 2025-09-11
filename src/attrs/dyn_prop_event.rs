pub trait OnCurrentMin: FnMut() {}
impl<T: FnMut()> OnCurrentMin for T {}

#[derive(Debug)]
pub struct DynPropEvent<F1>
where
    F1: OnCurrentMin,
{
    pub on_current_min: Option<F1>,
}

impl<F1> Default for DynPropEvent<F1>
where
    F1: OnCurrentMin,
{
    fn default() -> Self {
        Self {
            on_current_min: None,
        }
    }
}
