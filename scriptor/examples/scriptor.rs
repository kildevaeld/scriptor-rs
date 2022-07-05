use rquickjs::Func;
use scriptor::{Vm, VmBuilder};
use std::error::Error;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    tokio::task::LocalSet::default()
        .run_until(async move {
            let mut builder = VmBuilder::default();

            builder
                .cwd(".")
                .root("scriptor-root")
                .add_module(scriptor::fs::Module)
                .add_module(scriptor::os::Module)
                .add_module(scriptor::http::Module);

            let mut vm = builder.build().await?;
            // let mut vm = Vm::new(".").await?;

            // vm.with(|ctx| {
            //     ctx.globals().set(
            //         "print",
            //         Func::from(|arg: String| {
            //             //
            //             println!("{}", arg);
            //         }),
            //     )
            // })?;

            vm.run_main("test.ts", "Hello").await
        })
        .await?;

    Ok(())
}
