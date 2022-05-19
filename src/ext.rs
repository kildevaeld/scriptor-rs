use rquickjs::{AsArguments, FromJs, Function, Module, Object, Result};

pub trait ObjectExt<'js> {
    fn call<R, A>(&self, name: &str, args: A) -> Result<R>
    where
        A: AsArguments<'js>,
        R: FromJs<'js>;
}

impl<'js> ObjectExt<'js> for Module<'js> {
    fn call<R, A>(&self, name: &str, args: A) -> Result<R>
    where
        A: AsArguments<'js>,
        R: FromJs<'js>,
    {
        let func = self.get::<_, Function<'_>>(name)?;

        func.call(args)
    }
}

impl<'js> ObjectExt<'js> for Object<'js> {
    fn call<R, A>(&self, name: &str, args: A) -> Result<R>
    where
        A: AsArguments<'js>,
        R: FromJs<'js>,
    {
        let func = self.get::<_, Function<'_>>(name)?;

        func.call(args)
    }
}
