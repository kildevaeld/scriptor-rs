use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use rquickjs::{
    Context, FileResolver, Func, Function, Loader, Promise, Resolver, Result, Runtime, ScriptLoader,
};

#[cfg(feature = "os")]
static MAIN: &'static str = include_str!("../lib/main.js");
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
        let handle = self.rt.spawn_executor(rquickjs::Tokio);

        let idle = self.rt.idle();

        self.ctx.with(|ctx| {
            ctx.globals().set(
                "print",
                Func::from(|arg: String| {
                    println!("print: {}", arg);
                }),
            )
        })?;

        #[cfg(not(feature = "os"))]
        let source = tokio::fs::read_to_string(path).await?;

        #[cfg(all(feature = "typescript", not(feature = "os")))]
        let source = crate::compile("main", source).map_err(throw!())?;

        self.ctx
            .with(|ctx| {
                cfg_if::cfg_if! {
                    if #[cfg(not(feature = "os"))] {
                        let module = ctx.compile("main", source)?;
                        let main: Function = module.get("main")?;
                        main.call::<_, Promise<()>>(())
                    } else {
                        let module = ctx.compile("main", MAIN)?;
                        let main: Function = module.get("main")?;
                        let path = path.as_ref().to_string_lossy().to_string();
                        main.call::<_, Promise<()>>((path,))
                    }
                }
            })?
            .await?;

        tokio::time::sleep(Duration::from_millis(1)).await;

        if self.rt.is_job_pending() {
            while self.rt.is_job_pending() {
                self.rt.execute_pending_job()?;
                tokio::task::yield_now().await;
            }
        }

        idle.await;
        // handle.await.map_err(throw!())?;

        drop(handle);

        Ok(())
    }
}
