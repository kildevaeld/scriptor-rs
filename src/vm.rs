use std::path::{Path, PathBuf};

use rquickjs::{
    Context, FileResolver, Function, Loader, Promise, Resolver, Result, Runtime, ScriptLoader,
};

pub struct Vm {
    rt: Runtime,
    ctx: Context,
}

impl Vm {
    pub fn new(work_path: impl AsRef<Path>) -> Result<Vm> {
        let rt = Runtime::new()?;
        let ctx = Context::full(&rt)?;

        let (resolver, loader) = Vm::create_loaders(work_path.as_ref());

        rt.set_loader(resolver, loader);

        ctx.with(|ctx| crate::global::init(ctx))?;

        Ok(Vm { rt, ctx })
    }

    fn create_loaders(cwd: &Path) -> (impl Resolver, impl Loader) {
        let (resolver, loader) = crate::create();

        let mut script_resolver =
            FileResolver::default().with_path(&cwd.as_os_str().to_string_lossy());

        #[cfg(feature = "typescript")]
        script_resolver.add_pattern("{}.ts");

        #[cfg(not(feature = "typescript"))]
        let script_loader = ScriptLoader::default();

        #[cfg(feature = "typescript")]
        let script_loader = crate::TypescriptFileLoader::default();

        ((resolver, script_resolver), (loader, script_loader))
    }

    pub async fn run_main(self, path: impl AsRef<Path>) -> Result<()> {
        let source = tokio::fs::read_to_string(path).await?;

        self.rt.spawn_executor(rquickjs::Tokio);

        #[cfg(feature = "typescript")]
        let source = crate::compile("main", source).map_err(throw!())?;

        self.ctx
            .with(|ctx| {
                let module = ctx.compile("main", source)?;
                let main: Function = module.get("main")?;

                main.call::<_, Promise<()>>(())
            })?
            .await?;

        self.rt.idle().await;

        if self.rt.is_job_pending() {
            println!("pending");
        }

        // handle.await.map_err(throw!())?;

        Ok(())
    }
}
