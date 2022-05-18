#[macro_use]
mod macros;

#[cfg(feature = "vm")]
mod vm;

// mod builder;
#[cfg(any(feature = "fs", feature = "os"))]
mod file_desc;
pub mod global;
mod process;
#[cfg(any(feature = "fs", feature = "os"))]
mod stream;

mod bundle;

#[cfg(feature = "typescript")]
mod typescript_loader;

use rquickjs::{BuiltinResolver, Loader, ModuleLoader, Resolver};
#[cfg(feature = "typescript")]
pub use typescript_loader::*;

#[cfg(feature = "http")]
pub mod http;

#[cfg(feature = "fs")]
pub mod fs;

#[cfg(feature = "os")]
pub mod os;

pub use bundle::{PIPE, TASKS, UTIL};

#[cfg(feature = "vm")]
pub use vm::*;

pub use rquickjs::{Error, Result};

#[cfg(any(feature = "fs", feature = "os"))]
pub(crate) use file_desc::*;
#[cfg(any(feature = "fs", feature = "os"))]
pub(crate) use stream::*;

#[allow(unused_mut)]
pub fn create() -> (impl Resolver, impl Loader) {
    let mut resolver = BuiltinResolver::default();
    let mut loader = ModuleLoader::default();
    #[cfg(feature = "http")]
    {
        resolver.add_module("http");
        loader.add_module("http", http::Module);
    }

    #[cfg(feature = "fs")]
    {
        resolver.add_module("fs");
        loader.add_module("fs", fs::Module);
    }

    #[cfg(feature = "os")]
    {
        resolver.add_module("os");
        loader.add_module("os", os::Module);
    }

    ((resolver, PIPE, UTIL, TASKS), (loader, PIPE, UTIL, TASKS))
}
