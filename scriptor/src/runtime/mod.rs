#[cfg(any(feature = "fs", feature = "os"))]
mod file_desc;

pub mod global;
#[cfg(any(feature = "fs", feature = "os"))]
mod stream;

#[cfg(any(feature = "fs", feature = "os"))]
pub use self::stream::*;

#[cfg(any(feature = "fs", feature = "os"))]
pub use self::file_desc::*;
