use rquickjs::{
    BuiltinLoader, BuiltinResolver, Bundle, Context, Ctx, FileResolver, Function, IntoJs, Loader,
    ModuleDef, ModuleLoader, Promise, Resolver, Result, Runtime, Script, ScriptLoader,
};
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
    time::Duration,
};

use futures_lite::StreamExt;

#[cfg(feature = "os")]
static MAIN: &'static str = include_str!("../lib/main.js");

use crate::{
    bundle_module::{BundleModule, BundleModuleCol, BundleModuleImpl},
    user_module::{IntoUserModule, UserModule},
    utils::Either,
};

#[cfg(feature = "wasm")]
use crate::wasm_loader::{open_path, WasmConfig, WasmLoaders};

use super::bundle::*;

#[derive(Default)]
pub struct VmBuilder {
    modules: Vec<Box<dyn UserModule>>,
    bundles: Vec<Box<dyn BundleModule>>,
    cwd: Option<PathBuf>,
    root: Option<PathBuf>,
}

impl VmBuilder {
    pub fn add_module<M: IntoUserModule>(&mut self, module: M) -> &mut Self
    where
        M::UserModule: 'static,
    {
        self.modules.push(Box::new(module.into_module()));
        self
    }

    pub fn add_bundle<T>(&mut self, bundle: T) -> &mut Self
    where
        T: Loader + Resolver + 'static,
    {
        self.bundles.push(Box::new(BundleModuleImpl(bundle)));
        self
    }

    pub fn cwd(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.cwd = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn root(&mut self, root: impl Into<PathBuf>) -> &mut Self {
        self.root = Some(root.into());
        self
    }

    pub async fn build(self) -> Result<Vm> {
        self.build_with(|_| Ok(())).await
    }

    async fn ensure_root(path: &Path) -> std::io::Result<()> {
        if !path.exists() {
            tokio::fs::create_dir_all(path).await?;
        }

        let loaders = path.join("loaders");
        if !loaders.exists() {
            tokio::fs::create_dir_all(loaders).await?;
        }

        Ok(())
    }

    #[cfg(feature = "wasm")]
    async fn get_wasm_loader(root: Option<PathBuf>) -> anyhow::Result<Option<WasmLoaders>> {
        let root = match &root {
            Some(root) => root,
            None => return Ok(None),
        };

        VmBuilder::ensure_root(root).await?;

        let loaders = root.join("loaders");
        let cache = root.join("cache");

        let loader = open_path(WasmConfig {
            loaders: &loaders,
            cache: Some(&cache),
        })
        .await?;

        Ok(Some(loader))
    }

    #[cfg(not(feature = "wasm"))]
    async fn get_wasm_loader(
        root: Option<PathBuf>,
    ) -> std::result::Result<Option<BuiltinLoader>, std::convert::Infallible> {
        Ok(None)
    }

    pub async fn build_with<F: FnMut(Ctx<'_>) -> Result<()> + 'static>(
        self,
        config: F,
    ) -> Result<Vm> {
        let cwd = match self.cwd {
            Some(cwd) => cwd,
            None => std::env::current_dir()?,
        };

        let mut resolver = BuiltinResolver::default();
        let mut loader = ModuleLoader::default();

        for mut module in self.modules {
            module.register(&mut resolver, &mut loader);
        }

        log::debug!("using cwd: {:?}", cwd);

        #[allow(unused_mut)]
        let mut script_resolver = FileResolver::default()
            .with_path(&cwd.as_os_str().to_string_lossy())
            .with_path("./")
            .with_native();

        let script_loader = ScriptLoader::default();

        let wasm_loader = VmBuilder::get_wasm_loader(self.root)
            .await
            .map_err(throw!())?;

        #[cfg(feature = "wasm")]
        if let Some(loader) = &wasm_loader {
            for ext in loader.extensions() {
                let mut string = String::from("{}.");
                string.push_str(ext);
                script_resolver.add_pattern(string);
            }
        }
        // #[cfg(feature = "typescript")]
        // let script_loader = crate::TypescriptFileLoader::default();
        let rt = Runtime::new()?;
        let ctx = Context::full(&rt)?;

        rt.spawn_executor(rquickjs::Tokio);

        let resolver = (resolver, UTIL, PIPE, TASKS, script_resolver);
        let loader = match wasm_loader {
            Some(wasm) => Either::Left((loader, UTIL, PIPE, TASKS, wasm, script_loader)),
            None => Either::Right((loader, UTIL, PIPE, TASKS, script_loader)),
        };

        if !self.bundles.is_empty() {
            let col = BundleModuleCol(Rc::new(RefCell::new(self.bundles)));
            rt.set_loader((resolver, col.clone()), (col.clone(), loader));
        } else {
            rt.set_loader(resolver, loader);
        }

        ctx.with(|ctx| crate::global::init(ctx))?;

        ctx.with(config)?;

        Ok(Vm { rt, ctx })
    }
}

pub struct Vm {
    rt: Runtime,
    ctx: Context,
}

impl Vm {
    pub fn build() -> VmBuilder {
        VmBuilder::default()
    }

    pub async fn new(work_path: impl AsRef<Path>) -> Result<Vm> {
        let mut builder = VmBuilder::default();

        #[cfg(feature = "os")]
        builder.add_module(crate::os::Module);

        #[cfg(feature = "fs")]
        builder.add_module(crate::fs::Module);

        #[cfg(feature = "http")]
        builder.add_module(crate::http::Module);

        builder.cwd(work_path);
        builder.build().await
    }

    pub fn with<F: FnOnce(Ctx) -> R, R>(&self, func: F) -> R {
        self.ctx.with(func)
    }

    pub async fn run_main<A>(&mut self, path: impl AsRef<Path>, args: A) -> Result<()>
    where
        for<'js> A: IntoJs<'js>,
    {
        let idle = self.rt.idle();

        #[cfg(not(feature = "os"))]
        let source = tokio::fs::read_to_string(path).await?;

        #[cfg(all(feature = "typescript", not(feature = "os")))]
        let source = crate::compile("main", source).map_err(throw!())?;

        self.ctx
            .with(|ctx| {
                cfg_if::cfg_if! {
                    if #[cfg(not(feature = "os"))] {
                        let module = ctx.compile("main", source)?;
                        let main: Function = module.get("main")?;
                        main.call::<_, Promise<()>>((args,))
                    } else {
                        let module = ctx.compile("main", MAIN)?;
                        let main: Function = module.get("main")?;
                        let path = path.as_ref().to_string_lossy().to_string();
                        main.call::<_, Promise<()>>((path, args))
                    }
                }
            })?
            .await?;

        if self.rt.is_job_pending() {
            while self.rt.is_job_pending() {
                self.rt.execute_pending_job()?;
                tokio::task::yield_now().await;
            }
        }

        idle.await;

        Ok(())
    }

    pub async fn idle(&self) {
        self.rt.idle().await;
    }
}
