use super::{EsmLoader, EsmModule};
use relative_path::RelativePath;
use rquickjs::{Ctx, Error, Loader, Resolver, Result, Runtime};
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

    pub fn register(self, runtime: &Runtime) -> rquickjs::Result<()> {
        let esm = EsmModules(Arc::new(EsmModuleState {
            modules: Mutex::new(self.modules),
            cwd: match self.cwd {
                Some(cwd) => canonicalize(cwd)?,
                None => std::env::current_dir()?,
            },
            loaders: self.loaders,
        }));

        runtime.set_loader(esm.clone(), esm);

        Ok(())
    }
}

struct EsmModuleState {
    modules: Mutex<Vec<Box<dyn EsmModule>>>,
    loaders: Vec<Box<dyn EsmLoader>>,
    cwd: PathBuf,
}

#[derive(Clone)]
pub struct EsmModules(Arc<EsmModuleState>);

impl Resolver for EsmModules {
    fn resolve<'js>(&mut self, ctx: Ctx<'js>, base: &str, name: &str) -> Result<String> {
        let state = self.0.clone();
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
