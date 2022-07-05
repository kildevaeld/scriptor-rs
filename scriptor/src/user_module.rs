use rquickjs::{BuiltinResolver, ModuleDef, ModuleLoader};

pub trait UserModule {
    fn register(&mut self, resolver: &mut BuiltinResolver, loader: &mut ModuleLoader);
}

pub trait IntoUserModule {
    type UserModule: UserModule;
    fn into_module(self) -> Self::UserModule;
}

impl<S, M> IntoUserModule for (S, M)
where
    S: AsRef<str>,
    M: ModuleDef,
{
    type UserModule = UserModuleImpl<M, S>;
    fn into_module(self) -> Self::UserModule {
        UserModuleImpl::new(self.0, self.1)
    }
}

pub struct UserModuleImpl<M, S> {
    module: Option<M>,
    name: S,
}

impl<M, S> UserModuleImpl<M, S> {
    pub const fn new(name: S, module: M) -> UserModuleImpl<M, S> {
        UserModuleImpl {
            module: Some(module),
            name,
        }
    }
}

impl<M, S> UserModule for UserModuleImpl<M, S>
where
    M: ModuleDef,
    S: AsRef<str>,
{
    fn register(&mut self, resolver: &mut BuiltinResolver, loader: &mut ModuleLoader) {
        resolver.add_module(self.name.as_ref());
        loader.add_module(self.name.as_ref(), self.module.take().unwrap());
    }
}

impl<M, S> IntoUserModule for UserModuleImpl<M, S>
where
    M: ModuleDef,
    S: AsRef<str>,
{
    type UserModule = Self;
    fn into_module(self) -> Self::UserModule {
        self
    }
}

impl UserModule for Box<dyn UserModule> {
    fn register(&mut self, resolver: &mut BuiltinResolver, loader: &mut ModuleLoader) {
        (**self).register(resolver, loader)
    }
}
