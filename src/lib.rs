mod error;
mod parser;
mod tree;

pub type SmallVec<T> = smallvec::SmallVec<[T; 4]>;
pub type Params<'k, 'v> = SmallVec<(&'k [u8], &'v [u8])>;

pub use error::InsertError;
pub use tree::Tree;
