use std::{
    fs::canonicalize,
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use relative_path::RelativePath;
use rquickjs::{
    Bundle, Ctx, Error, Loader, Module, ModuleDef, Resolver, Result, Runtime, ScriptLoader,
};

use crate::{bundle_module::BundleModule, IntoUserModule, UserModule};

pub trait IntoEsmModule {
    type Module: EsmModule;
    fn into_module(self) -> Self::Module;
}

pub trait EsmModule {
    fn load<'js>(
        &mut self,
        ctx: Ctx<'js>,
        name: &str,
    ) -> Result<rquickjs::Module<'js, rquickjs::Loaded<()>>>;

    fn resolve<'js>(&mut self, ctx: Ctx<'js>, base: &str, name: &str) -> Result<String>;
}

impl EsmModule for Box<dyn EsmModule> {
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

impl<T> EsmModule for Bundle<T>
where
    Bundle<T>: Loader,
    Bundle<T>: Resolver,
{
    fn load<'js>(
        &mut self,
        ctx: Ctx<'js>,
        name: &str,
    ) -> Result<rquickjs::Module<'js, rquickjs::Loaded<()>>> {
        <Bundle<T> as Loader>::load(self, ctx, name)
    }

    fn resolve<'js>(&mut self, ctx: Ctx<'js>, base: &str, name: &str) -> Result<String> {
        <Bundle<T> as Resolver>::resolve(self, ctx, base, name)
    }
}

#[derive(Debug)]
pub struct NativeModule<M, S>(S, PhantomData<M>);

impl<M, S: Clone> Clone for NativeModule<M, S> {
    fn clone(&self) -> Self {
        NativeModule(self.0.clone(), PhantomData)
    }
}

impl<M, S: Copy> Copy for NativeModule<M, S> {}

impl<S> NativeModule<(), S> {
    pub const fn new<M>(name: S) -> NativeModule<M, S> {
        NativeModule(name, PhantomData)
    }
}

impl<M, S> EsmModule for NativeModule<M, S>
where
    S: AsRef<str>,
    M: ModuleDef,
{
    fn load<'js>(
        &mut self,
        ctx: Ctx<'js>,
        name: &str,
    ) -> Result<rquickjs::Module<'js, rquickjs::Loaded<()>>> {
        if name != self.0.as_ref() {
            return Err(rquickjs::Error::new_loading(name));
        }

        Ok(rquickjs::Module::new_def::<M, _>(ctx, name)?.into_loaded())
    }

    fn resolve<'js>(&mut self, _ctx: Ctx<'js>, base: &str, name: &str) -> Result<String> {
        if name != self.0.as_ref() {
            return Err(rquickjs::Error::new_resolving(base, name));
        }
        Ok(name.into())
    }
}

impl<T> IntoEsmModule for Bundle<T>
where
    Bundle<T>: Loader,
    Bundle<T>: Resolver,
{
    type Module = Bundle<T>;

    fn into_module(self) -> Self::Module {
        self
    }
}

impl<S, M> IntoEsmModule for NativeModule<M, S>
where
    S: AsRef<str>,
    M: ModuleDef,
{
    type Module = NativeModule<M, S>;

    fn into_module(self) -> Self::Module {
        self
    }
}

impl<S, M> IntoEsmModule for (S, M)
where
    S: AsRef<str>,
    M: ModuleDef,
{
    type Module = NativeModule<M, S>;

    fn into_module(self) -> Self::Module {
        NativeModule::new(self.0)
    }
}

impl EsmModule for Vec<Box<dyn EsmModule>> {
    fn load<'js>(
        &mut self,
        ctx: Ctx<'js>,
        name: &str,
    ) -> Result<rquickjs::Module<'js, rquickjs::Loaded<()>>> {
        let mut last = None;
        for next in self {
            match next.load(ctx, name) {
                Ok(ret) => return Ok(ret),
                Err(err) => {
                    last = Some(err);
                    continue;
                }
            };
        }

        Err(last.unwrap_or_else(|| rquickjs::Error::new_loading(name)))
    }

    fn resolve<'js>(&mut self, ctx: Ctx<'js>, base: &str, name: &str) -> Result<String> {
        let mut last = None;
        for next in self {
            match next.resolve(ctx, base, name) {
                Ok(ret) => return Ok(ret),
                Err(err) => {
                    last = Some(err);
                    continue;
                }
            };
        }

        Err(last.unwrap_or_else(|| rquickjs::Error::new_resolving(base, name)))
    }
}

#[derive(Default)]
pub struct EsmModulesBuilder {
    modules: Vec<Box<dyn EsmModule>>,
    cwd: Option<PathBuf>,
    loaders: Vec<Box<dyn Loader>>,
    extensions: Vec<String>,
}

impl EsmModulesBuilder {
    pub fn add_module<M: EsmModule + 'static>(&mut self, module: M) -> &mut Self {
        self.modules.push(Box::new(module));
        self
    }

    pub fn with_module<M: EsmModule + 'static>(mut self, module: M) -> Self {
        self.add_module(module);
        self
    }

    pub fn set_cwd(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.cwd = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn with_cwd(mut self, path: impl AsRef<Path>) -> Self {
        self.set_cwd(path);
        self
    }

    pub fn register(self, runtime: &Runtime) -> rquickjs::Result<()> {
        let esm = EsmModules(Arc::new(Mutex::new(EsmModuleState {
            modules: self.modules,
            cwd: match self.cwd {
                Some(cwd) => canonicalize(cwd)?,
                None => std::env::current_dir()?,
            },
        })));

        runtime.set_loader(esm.clone(), esm);

        Ok(())
    }
}

pub trait FileLoader {
    fn extentions(&self) -> &[String];
    fn load<'js>(
        &mut self,
        ctx: Ctx<'js>,
        name: &str,
    ) -> Result<rquickjs::Module<'js, rquickjs::Loaded<()>>>;
}

struct EsmModuleState {
    modules: Vec<Box<dyn EsmModule>>,
    cwd: PathBuf,
}

#[derive(Clone)]
pub struct EsmModules(Arc<Mutex<EsmModuleState>>);

impl Resolver for EsmModules {
    fn resolve<'js>(&mut self, ctx: Ctx<'js>, base: &str, name: &str) -> Result<String> {
        let mut state = self.0.lock().unwrap();
        if name.starts_with(".") {
            let root = match Path::new(base).parent() {
                Some(root) if root.is_absolute() => root,
                _ => &state.cwd,
            };

            let path = RelativePath::new(name);
            let fp = path.to_logical_path(&root);
            if fp.exists() {
                match fp.to_str() {
                    Some(s) => Ok(s.to_string()),
                    None => Err(rquickjs::Error::new_resolving_message(
                        base,
                        name,
                        "invalid path format",
                    )),
                }
            } else {
                Err(rquickjs::Error::new_resolving_message(
                    base,
                    name,
                    "path does not exists",
                ))
            }
        } else {
            state.modules.resolve(ctx, base, name)
        }
    }
}

impl Loader for EsmModules {
    fn load<'js>(
        &mut self,
        ctx: Ctx<'js>,
        name: &str,
    ) -> Result<rquickjs::Module<'js, rquickjs::Loaded<()>>> {
        let mut state = self.0.lock().unwrap();

        if let Ok(module) = state.modules.load(ctx, name) {
            return Ok(module);
        }

        drop(state);

        let path = RelativePath::new(name);

        if let Some("js") = path.extension() {
            let source: Vec<_> = std::fs::read(path.as_str())?;

            Module::new(ctx, name, source).map(|m| m.into_loaded())
        } else {
            Err(Error::new_loading(name))
        }
    }
}
