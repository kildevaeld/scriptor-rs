use rquickjs::{Ctx, Module, Result};
use std::{os::unix::prelude::OsStrExt, path::Path};

pub trait EsmLoader {
    fn extensions(&self) -> &[String];
    fn load<'js>(
        &self,
        ctx: Ctx<'js>,
        name: &Path,
        source: &[u8],
    ) -> Result<rquickjs::Module<'js, rquickjs::Loaded<()>>>;
}

impl EsmLoader for Box<dyn EsmLoader> {
    fn extensions(&self) -> &[String] {
        (&**self).extensions()
    }

    fn load<'js>(
        &self,
        ctx: Ctx<'js>,
        name: &Path,
        source: &[u8],
    ) -> Result<rquickjs::Module<'js, rquickjs::Loaded<()>>> {
        (&**self).load(ctx, name, source)
    }
}

pub struct ScriptLoader {
    extensions: Vec<String>,
}

impl Default for ScriptLoader {
    fn default() -> Self {
        ScriptLoader {
            extensions: vec!["js".into()],
        }
    }
}

impl EsmLoader for ScriptLoader {
    fn extensions(&self) -> &[String] {
        &self.extensions
    }

    fn load<'js>(
        &self,
        ctx: Ctx<'js>,
        name: &Path,
        source: &[u8],
    ) -> Result<rquickjs::Module<'js, rquickjs::Loaded<()>>> {
        Ok(Module::new(ctx, name.as_os_str().as_bytes(), source)?.into_loaded())
    }
}
