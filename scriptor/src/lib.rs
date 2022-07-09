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
mod ext;
mod utils;

mod bundle_module;
mod user_module;

// #[cfg(feature = "typescript")]
// mod typescript_loader;

use rquickjs::{BuiltinResolver, Loader, ModuleLoader, Resolver};

// #[cfg(feature = "typescript")]
// pub use typescript_loader::*;

#[cfg(feature = "http")]
pub mod http;

#[cfg(feature = "fs")]
pub mod fs;

#[cfg(feature = "os")]
pub mod os;

// mod vm_worker;

pub use bundle::{PIPE, TASKS, UTIL};

#[cfg(feature = "vm")]
pub use vm::{Vm, VmBuilder};

pub use rquickjs::{Error, Result};

pub use user_module::{IntoUserModule, UserModule, UserModuleImpl};

#[cfg(any(feature = "fs", feature = "os"))]
pub(crate) use file_desc::*;
#[cfg(any(feature = "fs", feature = "os"))]
pub(crate) use stream::*;

pub use ext::*;

#[cfg(feature = "wasm")]
pub mod wasm_loader;

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
