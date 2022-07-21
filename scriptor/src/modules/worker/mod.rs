mod worker;

use rquickjs::ModuleDef;

pub struct Module {}

impl ModuleDef for Module {
    fn load<'js>(
        _ctx: rquickjs::Ctx<'js>,
        _module: &rquickjs::Module<'js, rquickjs::Created>,
    ) -> rquickjs::Result<()> {
        Ok(())
    }

    fn eval<'js>(
        _ctx: rquickjs::Ctx<'js>,
        _module: &rquickjs::Module<'js, rquickjs::Loaded<rquickjs::Native>>,
    ) -> rquickjs::Result<()> {
        Ok(())

        
    }
}
