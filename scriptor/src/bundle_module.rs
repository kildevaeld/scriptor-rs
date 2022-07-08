use std::{cell::RefCell, rc::Rc};

use rquickjs::{Ctx, Loader, Resolver, Result};

pub trait BundleModule {
    fn load<'js>(
        &mut self,
        ctx: Ctx<'js>,
        name: &str,
    ) -> Result<rquickjs::Module<'js, rquickjs::Loaded<()>>>;

    fn resolve<'js>(&mut self, ctx: Ctx<'js>, base: &str, name: &str) -> Result<String>;
}

impl BundleModule for Box<dyn BundleModule> {
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

// impl Loader<Script> for Box<dyn BundleModule> {
//     fn load<'js>(
//         &mut self,
//         ctx: Ctx<'js>,
//         name: &str,
//     ) -> Result<rquickjs::Module<'js, rquickjs::Loaded<Script>>> {
//         <Self as BundleModule>::load(self, ctx, name)
//     }
// }

// impl Resolver for Box<dyn BundleModule> {
//     fn resolve<'js>(&mut self, ctx: Ctx<'js>, base: &str, name: &str) -> Result<String> {
//         <Self as BundleModule>::resolve(self, ctx, base, name)
//     }
// }

pub(crate) struct BundleModuleImpl<T>(pub(crate) T);

impl<T> BundleModule for BundleModuleImpl<T>
where
    T: Loader + Resolver,
{
    fn load<'js>(
        &mut self,
        ctx: Ctx<'js>,
        name: &str,
    ) -> Result<rquickjs::Module<'js, rquickjs::Loaded<()>>> {
        self.0.load(ctx, name)
    }

    fn resolve<'js>(&mut self, ctx: Ctx<'js>, base: &str, name: &str) -> Result<String> {
        self.0.resolve(ctx, base, name)
    }
}

#[derive(Clone)]
pub(crate) struct BundleModuleCol(pub(crate) Rc<RefCell<Vec<Box<dyn BundleModule + Send>>>>);

impl Loader for BundleModuleCol {
    fn load<'js>(
        &mut self,
        ctx: Ctx<'js>,
        name: &str,
    ) -> Result<rquickjs::Module<'js, rquickjs::Loaded<()>>> {
        let mut last = None;
        for next in self.0.borrow_mut().iter_mut() {
            last = match next.load(ctx, name) {
                Ok(ret) => return Ok(ret),
                Err(err) => Some(err),
            };
        }

        Err(last.unwrap_or(rquickjs::Error::new_loading(name)))
    }
}

impl Resolver for BundleModuleCol {
    fn resolve<'js>(&mut self, ctx: Ctx<'js>, base: &str, name: &str) -> Result<String> {
        let mut last = None;
        for next in self.0.borrow_mut().iter_mut() {
            last = match next.resolve(ctx, base, name) {
                Ok(ret) => return Ok(ret),
                Err(err) => Some(err),
            };
        }

        Err(last.unwrap_or(rquickjs::Error::new_resolving(base, name)))
    }
}
//
