use super::{EsmLoader, EsmModule};
use relative_path::RelativePath;
use rquickjs::{
    ClassDef, ClassId, Ctx, Error, FromJs, IntoJs, Loaded, Loader, Module, Resolver, Result,
    Runtime, Value,
};
use std::fs::canonicalize;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct EsmModulesBuilder {
    modules: Vec<Box<dyn EsmModule>>,
    cwd: Option<PathBuf>,
    loaders: Vec<Box<dyn EsmLoader>>,
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

    pub fn add_loader<L: EsmLoader + 'static>(&mut self, loader: L) -> &mut Self {
        self.loaders.push(Box::new(loader));
        self
    }

    pub fn with_loader<L: EsmLoader + 'static>(mut self, loader: L) -> Self {
        self.add_loader(loader);
        self
    }

    pub fn build(self) -> rquickjs::Result<EsmModules> {
        let esm = EsmModules(Arc::new(EsmModuleState {
            modules: Mutex::new(self.modules),
            cwd: match self.cwd {
                Some(cwd) => canonicalize(cwd)?,
                None => std::env::current_dir()?,
            },
            loaders: self.loaders,
        }));

        Ok(esm)
    }

    pub fn register(self, runtime: &Runtime) -> rquickjs::Result<()> {
        Ok(self.build()?.register(runtime))
    }
}

struct EsmModuleState {
    modules: Mutex<Vec<Box<dyn EsmModule>>>,
    loaders: Vec<Box<dyn EsmLoader>>,
    cwd: PathBuf,
}

#[derive(Clone)]
pub struct EsmModules(Arc<EsmModuleState>);

impl EsmModules {
    pub fn register(self, runtime: &Runtime) {
        runtime.set_loader(self.clone(), self);
    }

    pub fn compile<'js>(
        &self,
        ctx: Ctx<'js>,
        name: impl AsRef<Path>,
    ) -> Result<Module<'js, Loaded>> {
        let mut path = name.as_ref().to_path_buf();

        if !path.is_absolute() {
            path = path.canonicalize()?;
        }

        let ext = match path.extension() {
            Some(ext) => ext.to_string_lossy().to_string(),
            None => {
                return Err(Error::Exception {
                    message: "invalid path".into(),
                    file: "".into(),
                    line: 0,
                    stack: "".into(),
                })
            }
        };

        let state = self.0.clone();

        let loaders = state
            .loaders
            .iter()
            .filter(|loader| loader.extensions().contains(&ext));

        let source: Vec<_> = std::fs::read(&path)?;
        let mut last = None;
        for loader in loaders {
            match loader.load(ctx, &path, &source) {
                Ok(ret) => return Ok(ret),
                Err(err) => {
                    last = Some(err);
                    continue;
                }
            };
        }

        Err(last.unwrap_or_else(|| {
            Error::new_loading_message(name.as_ref().to_string_lossy(), "no loader for file type")
        }))
    }
}

impl ClassDef for EsmModules {
    const CLASS_NAME: &'static str = "Modules";

    unsafe fn class_id() -> &'static mut rquickjs::ClassId {
        static mut CLASS_ID: ClassId = ClassId::new();
        &mut CLASS_ID
    }

    const HAS_PROTO: bool = false;

    const HAS_STATIC: bool = false;

    const HAS_REFS: bool = false;
}

impl<'js> IntoJs<'js> for EsmModules {
    fn into_js(self, ctx: Ctx<'js>) -> Result<Value<'js>> {
        self.into_js_obj(ctx)
    }
}

impl<'js> FromJs<'js> for &'js EsmModules {
    fn from_js(ctx: Ctx<'js>, value: Value<'js>) -> Result<Self> {
        EsmModules::from_js_ref(ctx, value)
    }
}

impl<'js> FromJs<'js> for &'js mut EsmModules {
    fn from_js(ctx: Ctx<'js>, value: Value<'js>) -> Result<Self> {
        EsmModules::from_js_mut(ctx, value)
    }
}

impl<'js> FromJs<'js> for EsmModules {
    fn from_js(ctx: Ctx<'js>, value: Value<'js>) -> Result<Self> {
        EsmModules::from_js_obj(ctx, value)
    }
}

impl Resolver for EsmModules {
    fn resolve<'js>(&mut self, ctx: Ctx<'js>, base: &str, name: &str) -> Result<String> {
        log::debug!("resolving: {} from parent: {}", name, base);
        let state = self.0.clone();
        let path = RelativePath::new(name);
        if name.starts_with(".") || path.extension().is_some() {
            log::debug!("resolving as file module: {}", name);
            let root = match Path::new(base).parent() {
                Some(root) if root.is_absolute() => root,
                _ => &state.cwd,
            };

            // let path = RelativePath::new(name);
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
            log::debug!("resolving as native module: {}", name);
            state.modules.lock().unwrap().resolve(ctx, base, name)
        }
    }
}

impl Loader for EsmModules {
    fn load<'js>(
        &mut self,
        ctx: Ctx<'js>,
        name: &str,
    ) -> Result<rquickjs::Module<'js, rquickjs::Loaded<()>>> {
        let state = self.0.clone();

        let mut modules = state.modules.lock().expect("lock");
        if let Ok(module) = modules.load(ctx, name) {
            return Ok(module);
        }

        drop(modules);

        let path = RelativePath::new(name);

        let ext = match path.extension() {
            Some(ext) => ext.to_string(),
            None => {
                return Err(Error::Exception {
                    message: "invalid path".into(),
                    file: "".into(),
                    line: 0,
                    stack: "".into(),
                })
            }
        };

        let loaders = state
            .loaders
            .iter()
            .filter(|loader| loader.extensions().contains(&ext));

        let source: Vec<_> = std::fs::read(path.as_str())?;
        let mut last = None;
        let path = Path::new(name);
        for loader in loaders {
            match loader.load(ctx, path, &source) {
                Ok(ret) => return Ok(ret),
                Err(err) => {
                    last = Some(err);
                    continue;
                }
            };
        }

        Err(last.unwrap_or_else(|| Error::new_loading_message(name, "no loader for file type")))
    }
}
