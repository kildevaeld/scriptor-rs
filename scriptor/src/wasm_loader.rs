use std::path::{Path, PathBuf};

// use anyhow::Result;
use rquickjs::{Loader, Module as JsModule};
use wasmtime::{Config, Engine, Instance, Linker, Module, Store};

use self::loader::LoaderData;

wit_bindgen_wasmtime::import!("../loader.wit");
pub struct WasmLoader {
    store: Store<Context<(), LoaderData>>,
    exports: loader::Loader<Context<(), LoaderData>>,
    exts: Vec<String>,
}

impl WasmLoader {
    fn load<'js>(
        &mut self,
        ctx: rquickjs::Ctx<'js>,
        path: &str,
    ) -> rquickjs::Result<rquickjs::Module<'js, rquickjs::Loaded<()>>> {
        let source = std::fs::read_to_string(path)?;
        let output = self
            .exports
            .transform(&mut self.store, &source)
            .map_err(throw!())?;

        let output = match output {
            loader::Compilation::Success(ret) => ret,
            loader::Compilation::Failure(err) => {
                return Err(rquickjs::Error::new_loading_message(path, err))
            }
        };

        Ok(JsModule::new(ctx, path, output)?.into_loaded())
    }
}

// fn default_config() -> Result<Config> {
//     // Create an engine with caching enabled to assist with iteration in this
//     // project.
//     let mut config = Config::new();
//     config.cache_config_load_default()?;

//     config.wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);
//     Ok(config)
// }

fn default_wasi() -> wasmtime_wasi::WasiCtx {
    wasmtime_wasi::sync::WasiCtxBuilder::new()
        .inherit_stdio()
        .build()
}

struct Context<I, E> {
    wasi: wasmtime_wasi::WasiCtx,
    imports: I,
    exports: E,
}

// fn instantiate<I: Default, E: Default, T>(
//     wasm: &str,
//     add_imports: impl FnOnce(&mut Linker<Context<I, E>>) -> Result<()>,
//     mk_exports: impl FnOnce(
//         &mut Store<Context<I, E>>,
//         &Module,
//         &mut Linker<Context<I, E>>,
//     ) -> Result<(T, Instance)>,
// ) -> Result<(T, Store<Context<I, E>>)> {
//     let engine = Engine::new(&default_config()?)?;
//     let module = Module::from_file(&engine, wasm)?;

//     let mut linker = Linker::new(&engine);
//     add_imports(&mut linker)?;
//     wasmtime_wasi::add_to_linker(&mut linker, |cx| &mut cx.wasi)?;

//     let mut store = Store::new(
//         &engine,
//         Context {
//             wasi: default_wasi(),
//             imports: I::default(),
//             exports: E::default(),
//         },
//     );
//     let (exports, _instance) = mk_exports(&mut store, &module, &mut linker)?;
//     Ok((exports, store))
// }

fn instantiate<I: Default, E: Default, T>(
    engine: &Engine,
    wasm: impl AsRef<Path>,
    add_imports: impl FnOnce(&mut Linker<Context<I, E>>) -> anyhow::Result<()>,
    mk_exports: impl FnOnce(
        &mut Store<Context<I, E>>,
        &Module,
        &mut Linker<Context<I, E>>,
    ) -> anyhow::Result<(T, Instance)>,
) -> anyhow::Result<(T, Store<Context<I, E>>)> {
    // let engine = Engine::new(&default_config()?)?;
    let module = Module::from_file(&engine, wasm)?;

    let mut linker = Linker::new(&engine);
    add_imports(&mut linker)?;
    wasmtime_wasi::add_to_linker(&mut linker, |cx| &mut cx.wasi)?;

    let mut store = Store::new(
        &engine,
        Context {
            wasi: default_wasi(),
            imports: I::default(),
            exports: E::default(),
        },
    );
    let (exports, _instance) = mk_exports(&mut store, &module, &mut linker)?;
    Ok((exports, store))
}

pub fn open<P: AsRef<Path>>(engine: Engine, path: P) -> anyhow::Result<WasmLoader> {
    use loader::*;
    let (exports, mut store) = instantiate(
        &engine,
        path,
        |_linker: &mut Linker<Context<(), _>>| Ok(()),
        |store, module, linker| Loader::instantiate(store, module, linker, |cx| &mut cx.exports),
    )?;

    let exts = exports.extension(&mut store)?;

    Ok(WasmLoader {
        exports,
        store,
        exts,
    })
}

pub struct WasmConfig<'a> {
    pub loaders: &'a Path,
    pub config: &'a Path,
    pub cache: Option<&'a Path>,
}

pub async fn ensure_cache_config(root: impl AsRef<Path>) -> anyhow::Result<PathBuf> {
    let mut root_path = root.as_ref().to_path_buf();
    if !root_path.is_absolute() {
        root_path = tokio::fs::canonicalize(root_path).await?;
    }

    let cache_config_path = root_path.join("wasm-config.toml");
    let cache_path = root_path.join("wasm-cache");
    if !cache_path.exists() {
        tokio::fs::create_dir_all(&cache_path).await?;
    }

    if !cache_config_path.exists() {
        let cfg = format!(
            r#"
[cache]
enabled = true
directory = {:?}
cleanup-interval = "1d"
files-total-size-soft-limit = "1Gi"
"#,
            cache_path
        );

        tokio::fs::write(&cache_config_path, cfg).await?;
    }

    Ok(cache_config_path)
}

pub async fn open_path(cfg: WasmConfig<'_>) -> anyhow::Result<WasmLoaders> {
    use futures_lite::StreamExt;

    let mut config = Config::default();

    if let Some(cache) = cfg.cache {
        let path = ensure_cache_config(cache).await?;
        config.cache_config_load(path)?;
    }

    let engine = Engine::new(&config)?;

    let mut stream = tokio::fs::read_dir(cfg.loaders).await?;

    let mut loaders = Vec::default();

    while let Some(next) = stream.next_entry().await? {
        let path = next.path();
        let ext = path
            .extension()
            .map(|m| m.to_string_lossy())
            .unwrap_or_default();

        if &*ext != "wasm" {
            continue;
        }

        let engine = engine.clone();

        loaders.push(tokio::task::spawn_blocking(move || open(engine, path)));
    }

    let loaders = futures_lite::stream::iter(loaders)
        .then(|m| async move { m.await })
        .map(|ret| match ret {
            Ok(Ok(ret)) => Ok(ret),
            Ok(Err(err)) => Err(err),
            Err(err) => Err(anyhow::Error::new(err)),
        })
        .try_collect::<_, _, Vec<_>>()
        .await?;

    Ok(WasmLoaders { loaders })
}

pub struct WasmLoaders {
    loaders: Vec<WasmLoader>,
}

impl WasmLoaders {
    pub fn extensions<'a>(&'a self) -> impl Iterator<Item = &'a str> {
        self.loaders
            .iter()
            .flat_map(|m| m.exts.iter().map(|m| m.as_str()))
    }
}

impl Loader for WasmLoaders {
    fn load<'js>(
        &mut self,
        ctx: rquickjs::Ctx<'js>,
        p: &str,
    ) -> rquickjs::Result<rquickjs::Module<'js, rquickjs::Loaded<()>>> {
        let path = Path::new(p);
        let ext = path
            .extension()
            .map(|m| m.to_string_lossy())
            .unwrap_or_default()
            .to_string();

        for loader in self
            .loaders
            .iter_mut()
            .filter(|loader| loader.exts.contains(&ext))
        {
            match loader.load(ctx, p) {
                Ok(ret) => return Ok(ret),
                Err(err) => return Err(err),
            }
        }

        Err(rquickjs::Error::new_loading(p))
    }
}
