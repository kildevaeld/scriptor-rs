use rquickjs::{Context, Ctx, Function, IntoJs, Promise, Result, Runtime};
use std::path::Path;

#[cfg(feature = "os")]
static MAIN: &'static str = include_str!("../lib/main.js");

use crate::esm::EsmModulesBuilder;

#[derive(Default)]
pub struct VmBuilder {
    modules: EsmModulesBuilder,
}

impl VmBuilder {
    pub fn build(self) -> rquickjs::Result<Vm> {
        let rt = Runtime::new()?;
        let ctx = Context::full(&rt)?;

        rt.spawn_executor(rquickjs::Tokio);

        self.modules.register(&rt)?;

        Ok(Vm { rt, ctx })
    }
}

impl std::ops::Deref for VmBuilder {
    type Target = EsmModulesBuilder;
    fn deref(&self) -> &Self::Target {
        &self.modules
    }
}

impl std::ops::DerefMut for VmBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.modules
    }
}

pub struct Vm {
    rt: Runtime,
    ctx: Context,
}

impl Vm {
    pub fn new() -> VmBuilder {
        VmBuilder::default()
    }

    pub fn with<F: FnOnce(Ctx) -> R, R>(&self, func: F) -> R {
        self.ctx.with(func)
    }

    pub async fn idle(&self) {
        self.rt.idle().await;
    }
}
