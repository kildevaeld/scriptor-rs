use std::path::Path;

use wasmtime::{Engine, Instance, Linker, Module, Store};

pub struct Context<I, E> {
    pub wasi: wasmtime_wasi::WasiCtx,
    pub imports: I,
    pub exports: E,
}

fn default_wasi() -> wasmtime_wasi::WasiCtx {
    wasmtime_wasi::sync::WasiCtxBuilder::new()
        .inherit_stdio()
        .build()
}

pub fn instantiate<I: Default, E: Default, T>(
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
