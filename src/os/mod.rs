use rquickjs::{Accessor, Class, Func, ModuleDef};

use crate::file_desc::{FileDesc, Named};

impl Named for tokio::io::Stdout {
    const NAME: &'static str = "Stdout";
}

writer!(tokio::io::Stdout);

impl Named for tokio::io::Stderr {
    const NAME: &'static str = "Stderr";
}

writer!(tokio::io::Stderr);

impl Named for tokio::io::Stdin {
    const NAME: &'static str = "Stdin";
}

reader!(tokio::io::Stdin);

pub struct Module;

impl ModuleDef for Module {
    fn load<'js>(
        _ctx: rquickjs::Ctx<'js>,
        module: &rquickjs::Module<'js, rquickjs::Created>,
    ) -> rquickjs::Result<()> {
        module.add("stdout")?;
        module.add("stderr")?;
        module.add("stdin")?;
        Ok(())
    }

    fn eval<'js>(
        ctx: rquickjs::Ctx<'js>,
        module: &rquickjs::Module<'js, rquickjs::Loaded<rquickjs::Native>>,
    ) -> rquickjs::Result<()> {
        Class::<FileDesc<tokio::io::Stdout>>::register(ctx)?;
        Class::<FileDesc<tokio::io::Stderr>>::register(ctx)?;
        Class::<FileDesc<tokio::io::Stdin>>::register(ctx)?;

        module.set("stdout", FileDesc::new(tokio::io::stdout()))?;

        module.set(
            "stderr",
            Func::new("stderr", || FileDesc::new(tokio::io::stderr())),
        )?;

        module.set(
            "stdin",
            Func::new("stdin", || FileDesc::new(tokio::io::stdin())),
        )?;

        Ok(())
    }
}
