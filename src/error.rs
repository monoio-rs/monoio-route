#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InsertError {
    _priv: (),
}

impl InsertError {
    pub(crate) const fn new() -> Self {
        Self { _priv: () }
    }
}

impl std::fmt::Display for InsertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("insert route failed")
    }
}
impl std::error::Error for InsertError {}
