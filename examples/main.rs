use scriptor::Vm;
use std::error::Error;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    tokio::task::LocalSet::default()
        .run_until(async move {
            let vm = Vm::new(".")?;

            vm.run_main("test.ts", "Hello").await
        })
        .await?;

    Ok(())
}
