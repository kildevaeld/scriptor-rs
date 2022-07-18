use std::{path::Path, sync::Mutex};

use crate::{esm::EsmLoader, wasm::util::instantiate};

use self::loader::LoaderData;
use rquickjs::Module as JsModule;
use wasmtime::{Config, Engine, Linker, Store};

use super::util::Context;

wit_bindgen_wasmtime::import!("../loader.wit");
pub struct WasmLoaderPlugin {
    store: Mutex<Store<Context<(), LoaderData>>>,
    exports: loader::Loader<Context<(), LoaderData>>,
    exts: Vec<String>,
}

impl WasmLoaderPlugin {
    fn load<'js>(
        &self,
        ctx: rquickjs::Ctx<'js>,
        path: &Path,
        source: &[u8],
    ) -> rquickjs::Result<rquickjs::Module<'js, rquickjs::Loaded<()>>> {
        let mut store = self.store.lock().unwrap();
        let output = self
            .exports
            .transform(&mut *store, source)
            .map_err(throw!())?;

        let output = match output {
            loader::Compilation::Success(ret) => ret,
            loader::Compilation::Failure(err) => {
                return Err(rquickjs::Error::new_loading_message(
                    path.to_string_lossy().to_string(),
                    err,
                ))
            }
        };

        Ok(JsModule::new(ctx, path.to_string_lossy().to_string(), output)?.into_loaded())
    }
}

pub struct WasmPluginLoader {
    plugins: Vec<WasmLoaderPlugin>,
    extensions: Vec<String>,
}

pub fn open<P: AsRef<Path>>(engine: Engine, path: P) -> anyhow::Result<WasmLoaderPlugin> {
    use loader::*;
    let (exports, mut store) = instantiate(
        &engine,
        path,
        |_linker: &mut Linker<Context<(), _>>| Ok(()),
        |store, module, linker| Loader::instantiate(store, module, linker, |cx| &mut cx.exports),
    )?;

    let exts = exports.extension(&mut store)?;

    Ok(WasmLoaderPlugin {
        exports,
        store: Mutex::new(store),
        exts,
    })
}

impl WasmPluginLoader {
    pub fn new(path: impl AsRef<Path>) -> Result<WasmPluginLoader, anyhow::Error> {
        let mut config = Config::default();

        config.cache_config_load_default()?;

        // if let Some(cache) = cfg.cache {
        //     let path = ensure_cache_config(cache).await?;
        //     config.cache_config_load(path)?;
        // }

        let engine = Engine::new(&config)?;

        let stream = std::fs::read_dir(path)?;

        let mut loaders = Vec::default();

        for next in stream {
            let path = next?.path();
            let ext = path
                .extension()
                .map(|m| m.to_string_lossy())
                .unwrap_or_default();

            if &*ext != "wasm" {
                continue;
            }

            let engine = engine.clone();

            loaders.push(open(engine, path));
        }

        let plugins = loaders
            .into_iter()
            .map(|ret| match ret {
                Ok(ret) => Ok(ret),
                Err(err) => Err(err),
            })
            .collect::<Result<Vec<_>, _>>()?;

        let extensions = plugins.iter().flat_map(|m| m.exts.clone()).collect();

        Ok(WasmPluginLoader {
            plugins,
            extensions,
        })
    }
}

impl EsmLoader for WasmPluginLoader {
    fn extensions(&self) -> &[String] {
        &self.extensions
    }

    fn load<'js>(
        &self,
        ctx: rquickjs::Ctx<'js>,
        path: &std::path::Path,
        source: &[u8],
    ) -> rquickjs::Result<rquickjs::Module<'js, rquickjs::Loaded<()>>> {
        let mut last = None;
        for next in &self.plugins {
            match next.load(ctx, path, source) {
                Ok(ret) => return Ok(ret),
                Err(err) => {
                    last = Some(err);
                    continue;
                }
            };
        }

        Err(last
            .unwrap_or_else(|| rquickjs::Error::new_loading(path.to_string_lossy().to_string())))
    }
}
