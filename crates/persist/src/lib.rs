mod error;
mod global;
mod novel;
mod options;
mod persist;

pub use error::PersistError;
pub use global::Global;
pub use novel::{CoverLoc, PersistNovel, SavedNovel};
pub use options::PersistOptions;
pub use persist::Persist;
