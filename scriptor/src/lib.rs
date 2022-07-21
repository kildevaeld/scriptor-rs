#[macro_use]
mod macros;

pub mod esm;

pub mod modules;

#[cfg(feature = "vm")]
mod vm;

#[cfg(feature = "worker")]
mod vm_worker;

#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(feature = "runtime")]
mod scriptor;

mod runtime;
pub use self::runtime::*;

mod ext;

#[cfg(feature = "vm")]
pub use vm::{Vm, VmBuilder};

pub use rquickjs::{Error, Result};

pub use ext::*;

#[cfg(feature = "worker")]
pub use vm_worker::VmWorker;


#[cfg(feature = "runtime")]
pub use self::scriptor::*;