use rquickjs::Func;
use scriptor::Vm;
use std::error::Error;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    tokio::task::LocalSet::default()
        .run_until(async move {
            let mut vm = Vm::new(".")?;

            vm.with(|ctx| {
                ctx.globals().set(
                    "print",
                    Func::from(|arg: String| {
                        //
                        println!("{}", arg);
                    }),
                )
            })?;

            vm.run_main("test.ts", "Hello").await
        })
        .await?;

    Ok(())
}
