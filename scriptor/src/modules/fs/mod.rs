mod file;
mod read_dir;

use crate::{esm::NativeModule, runtime::JsStream, FileDesc};
use rquickjs::{Async, Class, Func, ModuleDef, Result};

use self::read_dir::{DirEntry, ReadDir};

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

        module.add("readDir")?;

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

        Class::<JsStream<ReadDir>>::register(ctx)?;
        Class::<DirEntry>::register(ctx)?;

        module.set(
            "open",
            Func::new(
                "open",
                (
                    Async(|path: String| async move {
                        let file = tokio::fs::File::open(path).await.map_err(throw!())?;
                        Result::<_>::Ok(FileDesc::new(file))
                    }),
                    Async(|path: String, mode: String| async move {
                        let mut opts = tokio::fs::OpenOptions::new();

                        for ch in mode.chars() {
                            match ch {
                                'r' => opts.read(true),
                                'w' => opts.write(true),
                                'a' => opts.append(true),
                                't' => opts.truncate(true),
                                'c' => opts.create(true),
                                _ => &mut opts,
                            };
                        }

                        let file = opts.open(path).await.map_err(throw!())?;
                        Result::<_>::Ok(FileDesc::new(file))
                    }),
                ),
            ),
        )?;

        module.set("writeFile", Func::from((Async(write), Async(write_str))))?;

        module.set("readFile", Func::from(Async(read)))?;

        module.set(
            "readDir",
            Func::from(Async(|path: String| {
                //
                async move {
                    Result::<_>::Ok(JsStream::new(ReadDir {
                        dir: tokio::fs::read_dir(path).await.map_err(throw!())?,
                    }))
                }
            })),
        )?;

        Ok(())
    }
}

pub const FS: NativeModule<Module, &'static str> = NativeModule::new("fs");
