use std::path::Path;

use rquickjs::{AsArguments, FromJs, Function, Module, Object, Promise, Result, Script};

#[cfg(features = "wasm")]
use crate::wasm::WasmPluginLoader;
use crate::{
    esm::{EsmModulesBuilder, ScriptLoader},
    global,
    wasm::WasmPluginLoader,
    Vm, VmBuilder,
};

pub struct ScriptorBuilder {
    modules: VmBuilder,
}

impl ScriptorBuilder {
    pub fn new(path: impl AsRef<Path>) -> anyhow::Result<ScriptorBuilder> {
        let mut modules = VmBuilder::default();

        modules
            .add_module(crate::modules::UTIL)
            .add_module(crate::modules::TASKS)
            .add_module(crate::modules::PIPE);

        modules.set_cwd(path);

        modules.add_loader(ScriptLoader::default());

        #[cfg(feature = "wasm")]
        modules.add_loader(WasmPluginLoader::new_default()?);

        #[cfg(feature = "fs")]
        modules.add_module(crate::modules::FS);

        #[cfg(feature = "http")]
        modules.add_module(crate::modules::HTTP);

        #[cfg(feature = "os")]
        modules.add_module(crate::modules::OS);

        Ok(ScriptorBuilder { modules })
    }

    pub fn build(self) -> anyhow::Result<Scriptor> {
        let vm = self.modules.build()?;

        vm.with(|ctx| global::init(ctx))?;

        Ok(Scriptor { vm })
    }
}

impl std::ops::Deref for ScriptorBuilder {
    type Target = VmBuilder;
    fn deref(&self) -> &Self::Target {
        &self.modules
    }
}

impl std::ops::DerefMut for ScriptorBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.modules
    }
}

pub struct Scriptor {
    vm: Vm,
}

impl Scriptor {
    pub async fn eval_path<V, A>(&self, path: impl AsRef<Path>, args: A) -> Result<V>
    where
        for<'js> V: FromJs<'js> + 'static,
        for<'js> A: AsArguments<'js>,
    {
        let ret = self
            .vm
            .with(|ctx| {
                let module = self.vm.modules().compile(ctx, path)?;

                let module = module.eval()?;

                module.meta::<Object>()?.set("main", true)?;

                let main: Function = module.get("main")?;

                let ret: Promise<V> = main.call(args)?;

                Result::Ok(ret)
            })?
            .await?;

        Ok(ret)
    }
}

impl std::ops::Deref for Scriptor {
    type Target = Vm;
    fn deref(&self) -> &Self::Target {
        &self.vm
    }
}
