use rquickjs::{
    BuiltinResolver, Bundle, Context, Ctx, FileResolver, Function, IntoJs, Loader, ModuleDef,
    ModuleLoader, Promise, Resolver, Result, Runtime, Script,
};
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
    time::Duration,
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

impl<M, S> IntoUserModule for UserModuleImpl<M, S>
where
    M: ModuleDef,
    S: AsRef<str>,
{
    type UserModule = Self;
    fn into_module(self) -> Self::UserModule {
        self
    }
}

impl UserModule for Box<dyn UserModule> {
    fn register(&mut self, resolver: &mut BuiltinResolver, loader: &mut ModuleLoader) {
        (**self).register(resolver, loader)
    }
}

//

pub trait BundleModule {
    fn load<'js>(
        &mut self,
        ctx: Ctx<'js>,
        name: &str,
    ) -> Result<rquickjs::Module<'js, rquickjs::Loaded<()>>>;

    fn resolve<'js>(&mut self, ctx: Ctx<'js>, base: &str, name: &str) -> Result<String>;
}

impl BundleModule for Box<dyn BundleModule> {
    fn load<'js>(
        &mut self,
        ctx: Ctx<'js>,
        name: &str,
    ) -> Result<rquickjs::Module<'js, rquickjs::Loaded<()>>> {
        (&mut **self).load(ctx, name)
    }

    fn resolve<'js>(&mut self, ctx: Ctx<'js>, base: &str, name: &str) -> Result<String> {
        (&mut **self).resolve(ctx, base, name)
    }
}

// impl Loader<Script> for Box<dyn BundleModule> {
//     fn load<'js>(
//         &mut self,
//         ctx: Ctx<'js>,
//         name: &str,
//     ) -> Result<rquickjs::Module<'js, rquickjs::Loaded<Script>>> {
//         <Self as BundleModule>::load(self, ctx, name)
//     }
// }

// impl Resolver for Box<dyn BundleModule> {
//     fn resolve<'js>(&mut self, ctx: Ctx<'js>, base: &str, name: &str) -> Result<String> {
//         <Self as BundleModule>::resolve(self, ctx, base, name)
//     }
// }

struct BundleModuleImpl<T>(T);

impl<T> BundleModule for BundleModuleImpl<T>
where
    T: Loader + Resolver,
{
    fn load<'js>(
        &mut self,
        ctx: Ctx<'js>,
        name: &str,
    ) -> Result<rquickjs::Module<'js, rquickjs::Loaded<()>>> {
        self.0.load(ctx, name)
    }

    fn resolve<'js>(&mut self, ctx: Ctx<'js>, base: &str, name: &str) -> Result<String> {
        self.0.resolve(ctx, base, name)
    }
}

#[derive(Clone)]
struct BundleModuleCol(Rc<RefCell<Vec<Box<dyn BundleModule>>>>);

impl Loader for BundleModuleCol {
    fn load<'js>(
        &mut self,
        ctx: Ctx<'js>,
        name: &str,
    ) -> Result<rquickjs::Module<'js, rquickjs::Loaded<()>>> {
        let mut last = None;
        for next in self.0.borrow_mut().iter_mut() {
            last = match next.load(ctx, name) {
                Ok(ret) => return Ok(ret),
                Err(err) => Some(err),
            };
        }

        Err(last.unwrap_or(rquickjs::Error::new_loading(name)))
    }
}

impl Resolver for BundleModuleCol {
    fn resolve<'js>(&mut self, ctx: Ctx<'js>, base: &str, name: &str) -> Result<String> {
        let mut last = None;
        for next in self.0.borrow_mut().iter_mut() {
            last = match next.resolve(ctx, base, name) {
                Ok(ret) => return Ok(ret),
                Err(err) => Some(err),
            };
        }

        Err(last.unwrap_or(rquickjs::Error::new_resolving(base, name)))
    }
}
//

#[derive(Default)]
pub struct VmBuilder {
    modules: Vec<Box<dyn UserModule>>,
    bundles: Vec<Box<dyn BundleModule>>,
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

        log::debug!("using cwd: {:?}", cwd);

        let mut script_resolver = FileResolver::default()
            .with_path(&cwd.as_os_str().to_string_lossy())
            .with_path("./")
            .with_native();

        #[cfg(feature = "typescript")]
        script_resolver.add_pattern("{}.ts");

        #[cfg(not(feature = "typescript"))]
        let script_loader = ScriptLoader::default().add;

        #[cfg(feature = "typescript")]
        let script_loader = crate::TypescriptFileLoader::default();

        let rt = Runtime::new()?;
        let ctx = Context::full(&rt)?;

        rt.spawn_executor(rquickjs::Tokio);

        let resolver = (resolver, UTIL, PIPE, TASKS, script_resolver);
        let loader = (loader, UTIL, PIPE, TASKS, script_loader);

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
