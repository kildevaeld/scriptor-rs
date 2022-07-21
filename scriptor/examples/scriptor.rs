use rquickjs::{Func, Promise};
use scriptor::{global, modules, wasm, ObjectExt, VmBuilder};
use tokio::task::LocalSet;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let source = tokio::fs::read_to_string("test.ts").await?;

    let local_set = LocalSet::default();

    local_set
        .run_until(async move {
            let mut builder = VmBuilder::default();

            builder
                .set_cwd(".")
                // .root("scriptor-root")
                .add_module(modules::FS)
                .add_module(modules::HTTP)
                .add_module(modules::OS)
                .add_module(modules::UTIL)
                .add_module(modules::TASKS)
                .add_module(modules::PIPE)
                .add_loader(wasm::WasmPluginLoader::new("./target/wasm32-wasi/release")?);

            let vm = builder.build()?;

            vm.with(|ctx| global::init(ctx))?;

            // let mut vm = Vm::new(".").await?;

            let key = vm.with(|ctx| {
                //

                ctx.globals().set(
                    "print",
                    Func::from(|s: String| {
                        //
                        println!("{}", s);
                    }),
                )?;

                let module = ctx.compile("main", source)?;

                // let ret: Promise<()> = module.call("main", (100,))?;

                rquickjs::Result::Ok(ctx.register(module.into()))
            })?;

            vm.idle().await;

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            anyhow::Ok(())
        })
        .await?;

    local_set.await;

    Ok(())
}
