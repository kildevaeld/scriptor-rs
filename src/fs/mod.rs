mod file;

use rquickjs::{bind, Async, Class, Func, ModuleDef, Result};

use crate::{stream::JsStream, FileDesc};

pub struct Module;

async fn read(path: String) -> Result<Vec<u8>> {
    let output = tokio::fs::read(path).await.map_err(throw!())?;
    Ok(output)
}

async fn write(path: String, data: Vec<u8>) -> Result<()> {
    tokio::fs::write(path, &data).await.map_err(throw!())?;
    Ok(())
}

async fn write_str(path: String, data: String) -> Result<()> {
    tokio::fs::write(path, &data).await.map_err(throw!())?;
    Ok(())
}

impl ModuleDef for Module {
    fn load<'js>(
        _ctx: rquickjs::Ctx<'js>,
        module: &rquickjs::Module<'js, rquickjs::Created>,
    ) -> rquickjs::Result<()> {
        module.add("File")?;

        module.add("open")?;
        module.add("readFile")?;
        module.add("writeFile")?;

        Ok(())
    }

    fn eval<'js>(
        ctx: rquickjs::Ctx<'js>,
        module: &rquickjs::Module<'js, rquickjs::Loaded<rquickjs::Native>>,
    ) -> rquickjs::Result<()> {
        Class::<FileDesc<tokio::fs::File>>::register(ctx)?;
        Class::<
            JsStream<
                tokio_stream::wrappers::LinesStream<
                    tokio::io::BufReader<FileDesc<tokio::fs::File>>,
                >,
            >,
        >::register(ctx)?;

        module.set(
            "open",
            Func::new(
                "open",
                Async(|path: String| async move {
                    let file = tokio::fs::File::open(path).await.map_err(throw!())?;
                    Result::<_>::Ok(FileDesc::new(file))
                }),
            ),
        )?;

        module.set("writeFile", Func::from((Async(write), Async(write_str))))?;

        module.set("readFile", Func::from(Async(read)))?;

        Ok(())
    }
}
