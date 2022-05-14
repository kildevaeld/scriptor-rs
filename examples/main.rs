use std::error::Error;

use rquickjs::{Context, Func, Function, Promise, Runtime};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let rt = Runtime::new()?;
    let ctx = Context::full(&rt)?;

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
