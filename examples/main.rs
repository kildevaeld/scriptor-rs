use std::error::Error;

use rquickjs::{BuiltinResolver, Context, Func, Function, ModuleLoader, Promise, Runtime};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let rt = Runtime::new()?;
    let ctx = Context::full(&rt)?;

    // let resolver = BuiltinResolver::default()
    //     .with_module("http")
    //     .with_module("fs")
    //     .with_module("os");

    // let loader = ModuleLoader::default()
    //     .with_module("http", scriptor::http::Module)
    //     .with_module("fs", scriptor::fs::Module)
    //     .with_module("os", scriptor::os::Module);

    let (resolver, loader) = scriptor::create();

    rt.set_loader(resolver, loader);

    ctx.with(|ctx| scriptor::global::init(ctx))?;

    let source = tokio::fs::read_to_string("test.js").await?;

    tokio::task::LocalSet::default()
        .run_until(async move {
            rt.spawn_executor(rquickjs::Tokio);

            let wait = ctx.with(|ctx| {
                //

                ctx.globals().set(
                    "print",
                    Func::from(|arg: String| {
                        println!("{}", arg);
                    }),
                )?;

                let module = ctx.compile("test.js", source)?;

                let func: Function = module.get("main")?;

                func.call::<_, Promise<()>>(())
            })?;

            wait.await?;

            rt.idle().await;

            rquickjs::Result::<_>::Ok(())
        })
        .await?;

    Ok(())
}
