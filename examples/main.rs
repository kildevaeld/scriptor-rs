use scriptor::Vm;
use std::error::Error;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let vm = Vm::new(".")?;

    tokio::task::LocalSet::default()
        .run_until(async move { vm.run_main("test.ts", ()).await })
        .await?;

    Ok(())
}
