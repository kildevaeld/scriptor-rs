#[macro_use]
mod macros;

mod builder;
mod file_desc;
pub mod global;
mod process;
mod stream;

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

pub(crate) use file_desc::*;
pub(crate) use stream::*;

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

    (resolver, loader)
}
