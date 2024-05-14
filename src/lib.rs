mod error;
mod parser;
mod tree;
mod util;

pub type SmallVec<T> = smallvec::SmallVec<[T; 4]>;
pub type Params<'k, 'v> = SmallVec<(&'k [u8], &'v [u8])>;
pub type ParamsStr<'k, 'v> = SmallVec<(&'k str, &'v str)>;
pub type ParamsOwned = SmallVec<(Vec<u8>, Vec<u8>)>;
pub type ParamsStrOwned = SmallVec<(String, String)>;

pub use error::InsertError;
pub use tree::Tree;
pub use util::{ParamsConvertOwned, ParamsConvertStr, ParamsGet, ParamsGetOwned};
