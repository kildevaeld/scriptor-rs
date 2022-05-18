use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use rquickjs::{
    AsArguments, BuiltinResolver, Bundle, Context, Ctx, FileResolver, Func, Function, IntoJs,
    Loader, ModuleDef, ModuleLoader, Promise, Resolver, Result, Runtime, ScriptLoader,
};

#[cfg(feature = "os")]
static MAIN: &'static str = include_str!("../lib/main.js");

use super::bundle::*;

pub trait UserModule {
    fn register(&mut self, resolver: &mut BuiltinResolver, loader: &mut ModuleLoader);
}

pub trait IntoUserModule {
    type UserModule: UserModule;
    fn into_module(self) -> Self::UserModule;
}

impl<S, M> IntoUserModule for (S, M)
where
    S: AsRef<str>,
    M: ModuleDef,
{
    type UserModule = UserModuleImpl<M, S>;
    fn into_module(self) -> Self::UserModule {
        UserModuleImpl::new(self.0, self.1)
    }
}

pub struct UserModuleImpl<M, S> {
    module: Option<M>,
    name: S,
}

impl<M, S> UserModuleImpl<M, S> {
    pub const fn new(name: S, module: M) -> UserModuleImpl<M, S> {
        UserModuleImpl {
            module: Some(module),
            name,
        }
    }
}

impl<M, S> UserModule for UserModuleImpl<M, S>
where
    M: ModuleDef,
    S: AsRef<str>,
{
    fn register(&mut self, resolver: &mut BuiltinResolver, loader: &mut ModuleLoader) {
        resolver.add_module(self.name.as_ref());
        loader.add_module(self.name.as_ref(), self.module.take().unwrap());
    }
}

impl UserModule for Box<dyn UserModule> {
    fn register(&mut self, resolver: &mut BuiltinResolver, loader: &mut ModuleLoader) {
        (**self).register(resolver, loader)
    }
}

#[derive(Default)]
pub struct VmBuilder {
    modules: Vec<Box<dyn UserModule>>,
    cwd: Option<PathBuf>,
}

impl VmBuilder {
    pub fn add_module<M: IntoUserModule>(&mut self, module: M) -> &mut Self
    where
        M::UserModule: 'static,
    {
        self.modules.push(Box::new(module.into_module()));
        self
    }

    pub fn cwd(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.cwd = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn build(self) -> Result<Vm> {
        self.build_with(|_| Ok(()))
    }

    pub fn build_with<F: FnMut(Ctx<'_>) -> Result<()> + 'static>(self, config: F) -> Result<Vm> {
        let cwd = match self.cwd {
            Some(cwd) => cwd,
            None => std::env::current_dir()?,
        };

        let mut resolver = BuiltinResolver::default();
        let mut loader = ModuleLoader::default();

        for mut module in self.modules {
            module.register(&mut resolver, &mut loader);
        }

        let mut script_resolver =
            FileResolver::default().with_path(&cwd.as_os_str().to_string_lossy());

        #[cfg(feature = "typescript")]
        script_resolver.add_pattern("{}.ts");

        #[cfg(not(feature = "typescript"))]
        let script_loader = ScriptLoader::default();

        #[cfg(feature = "typescript")]
        let script_loader = crate::TypescriptFileLoader::default();

        let rt = Runtime::new()?;
        let ctx = Context::full(&rt)?;

        rt.set_loader(
            (resolver, UTIL, PIPE, TASKS, script_resolver),
            (loader, UTIL, PIPE, TASKS, script_loader),
        );

        ctx.with(|ctx| crate::global::init(ctx))?;

        Ok(Vm {
            rt,
            ctx,
            cfg: Some(Box::new(config)),
        })
    }
}

pub struct Vm {
    rt: Runtime,
    ctx: Context,
    cfg: Option<Box<dyn FnMut(Ctx<'_>) -> Result<()>>>,
}

impl Vm {
    pub fn build() -> VmBuilder {
        VmBuilder::default()
    }

    pub fn new(work_path: impl AsRef<Path>) -> Result<Vm> {
        let mut builder = VmBuilder::default();

        #[cfg(feature = "os")]
        builder.add_module(crate::os::Module);

        #[cfg(feature = "fs")]
        builder.add_module(crate::fs::Module);

        #[cfg(feature = "http")]
        builder.add_module(crate::http::Module);

        builder.cwd(work_path);
        builder.build()
    }
    // pub fn new(work_path: impl AsRef<Path>) -> Result<Vm> {
    //     let rt = Runtime::new()?;
    //     let ctx = Context::full(&rt)?;

    //     let (resolver, loader) = Vm::create_loaders(work_path.as_ref());

    //     rt.set_loader(resolver, loader);

    //     ctx.with(|ctx| crate::global::init(ctx))?;

    //     Ok(Vm { rt, ctx })
    // }

    // fn create_loaders(cwd: &Path) -> (impl Resolver, impl Loader) {
    //     let (resolver, loader) = crate::create();

    //     let mut script_resolver =
    //         FileResolver::default().with_path(&cwd.as_os_str().to_string_lossy());

    //     #[cfg(feature = "typescript")]
    //     script_resolver.add_pattern("{}.ts");

    //     #[cfg(not(feature = "typescript"))]
    //     let script_loader = ScriptLoader::default();

    //     #[cfg(feature = "typescript")]
    //     let script_loader = crate::TypescriptFileLoader::default();

    //     ((resolver, script_resolver), (loader, script_loader))
    // }

    pub async fn run_main<A>(mut self, path: impl AsRef<Path>, args: A) -> Result<()>
    where
        for<'js> A: IntoJs<'js>,
    {
        let handle = self.rt.spawn_executor(rquickjs::Tokio);

        let idle = self.rt.idle();

        self.ctx.with(|ctx| {
            ctx.globals().set(
                "print",
                Func::from(|arg: String| {
                    println!("print: {}", arg);
                }),
            )
        })?;

        if let Some(cfg) = self.cfg.take() {
            self.ctx.with(cfg)?;
        }

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

        tokio::time::sleep(Duration::from_millis(1)).await;

        if self.rt.is_job_pending() {
            while self.rt.is_job_pending() {
                self.rt.execute_pending_job()?;
                tokio::task::yield_now().await;
            }
        }

        idle.await;
        // handle.await.map_err(throw!())?;

        drop(handle);

        Ok(())
    }
}
