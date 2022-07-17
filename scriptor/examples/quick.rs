use rquickjs::{Context, Loader, Resolver, Runtime, ScriptLoader};
use scriptor::{bundle, esm};

fn main() -> rquickjs::Result<()> {
    let rt = Runtime::new()?;
    let ctx = Context::full(&rt)?;

    esm::EsmModulesBuilder::default()
        .with_cwd("./scriptor/examples")
        .with_module(bundle::UTIL)
        .register(&rt)?;

    ctx.with(|ctx| {
        ctx.globals().set(
            "print",
            rquickjs::Func::new("print", |v: String| {
                //
                println!("{}", v);
            }),
        )
    })?;

    ctx.with(|ctx| {
        let module = ctx.compile::<_, _>(
            "test",
            r#"
        import test from './quick.js';
        test();

    "#,
        )?;

        rquickjs::Result::Ok(())
    })?;

    Ok(())
}
