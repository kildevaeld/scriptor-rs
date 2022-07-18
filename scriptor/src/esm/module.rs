use std::marker::PhantomData;

use rquickjs::{Bundle, Ctx, Loader, ModuleDef, Resolver, Result};

pub trait EsmModule {
    fn load<'js>(
        &mut self,
        ctx: Ctx<'js>,
        name: &str,
    ) -> rquickjs::Result<rquickjs::Module<'js, rquickjs::Loaded<()>>>;

    fn resolve<'js>(&mut self, ctx: Ctx<'js>, base: &str, name: &str) -> rquickjs::Result<String>;
}

impl EsmModule for Box<dyn EsmModule> {
    fn load<'js>(
        &mut self,
        ctx: Ctx<'js>,
        name: &str,
    ) -> Result<rquickjs::Module<'js, rquickjs::Loaded<()>>> {
        (&mut **self).load(ctx, name)
    }

    fn resolve<'js>(&mut self, ctx: Ctx<'js>, base: &str, name: &str) -> Result<String> {
        (&mut **self).resolve(ctx, base, name)
    }
}

impl<T> EsmModule for Bundle<T>
where
    Bundle<T>: Loader,
    Bundle<T>: Resolver,
{
    fn load<'js>(
        &mut self,
        ctx: Ctx<'js>,
        name: &str,
    ) -> Result<rquickjs::Module<'js, rquickjs::Loaded<()>>> {
        <Bundle<T> as Loader>::load(self, ctx, name)
    }

    fn resolve<'js>(&mut self, ctx: Ctx<'js>, base: &str, name: &str) -> Result<String> {
        <Bundle<T> as Resolver>::resolve(self, ctx, base, name)
    }
}

#[derive(Debug)]
pub struct NativeModule<M, S>(S, PhantomData<M>);

impl<M, S: Clone> Clone for NativeModule<M, S> {
    fn clone(&self) -> Self {
        NativeModule(self.0.clone(), PhantomData)
    }
}

impl<M, S: Copy> Copy for NativeModule<M, S> {}

impl<S> NativeModule<(), S> {
    pub const fn new<M>(name: S) -> NativeModule<M, S> {
        NativeModule(name, PhantomData)
    }
}

impl<M, S> EsmModule for NativeModule<M, S>
where
    S: AsRef<str>,
    M: ModuleDef,
{
    fn load<'js>(
        &mut self,
        ctx: Ctx<'js>,
        name: &str,
    ) -> Result<rquickjs::Module<'js, rquickjs::Loaded<()>>> {
        if name != self.0.as_ref() {
            return Err(rquickjs::Error::new_loading(name));
        }

        Ok(rquickjs::Module::new_def::<M, _>(ctx, name)?.into_loaded())
    }

    fn resolve<'js>(&mut self, _ctx: Ctx<'js>, base: &str, name: &str) -> Result<String> {
        if name != self.0.as_ref() {
            return Err(rquickjs::Error::new_resolving(base, name));
        }
        Ok(name.into())
    }
}

impl EsmModule for Vec<Box<dyn EsmModule>> {
    fn load<'js>(
        &mut self,
        ctx: Ctx<'js>,
        name: &str,
    ) -> Result<rquickjs::Module<'js, rquickjs::Loaded<()>>> {
        let mut last = None;
        for next in self {
            match next.load(ctx, name) {
                Ok(ret) => return Ok(ret),
                Err(err) => {
                    last = Some(err);
                    continue;
                }
            };
        }

        Err(last.unwrap_or_else(|| rquickjs::Error::new_loading(name)))
    }

    fn resolve<'js>(&mut self, ctx: Ctx<'js>, base: &str, name: &str) -> Result<String> {
        let mut last = None;
        for next in self {
            match next.resolve(ctx, base, name) {
                Ok(ret) => return Ok(ret),
                Err(err) => {
                    last = Some(err);
                    continue;
                }
            };
        }

        Err(last.unwrap_or_else(|| rquickjs::Error::new_resolving(base, name)))
    }
}
