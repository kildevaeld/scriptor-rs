use rquickjs::{Context, Runtime};
use scriptor::{esm, global, modules, wasm};

fn main() -> anyhow::Result<()> {
    let rt = Runtime::new()?;
    let ctx = Context::full(&rt)?;

    esm::EsmModulesBuilder::default()
        .with_cwd("./scriptor/examples")
        .with_module(modules::UTIL)
        .with_loader(esm::ScriptLoader::default())
        .with_loader(wasm::WasmPluginLoader::new("./target/wasm32-wasi/release")?)
        .register(&rt)?;

    ctx.with(|ctx| {
        ctx.globals().set(
            "print",
            rquickjs::Func::new("print", |v: String| {
                //
                println!("{}", v);
            }),
        )?;

        global::init(ctx)
    })?;

    ctx.with(|ctx| {
        let module = ctx.compile::<_, _>(
            "test",
            r#"
        import test from './quick.ts';
        test();

    "#,
        )?;

        rquickjs::Result::Ok(())
    })?;

    Ok(())
}
