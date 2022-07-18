use rquickjs::{Bundle, Loader, ModuleDef, Resolver};

use super::{EsmModule, NativeModule};

pub trait IntoEsmModule {
    type Module: EsmModule;
    fn into_module(self) -> Self::Module;
}

impl<T> IntoEsmModule for Bundle<T>
where
    Bundle<T>: Loader,
    Bundle<T>: Resolver,
{
    type Module = Bundle<T>;

    fn into_module(self) -> Self::Module {
        self
    }
}

impl<S, M> IntoEsmModule for NativeModule<M, S>
where
    S: AsRef<str>,
    M: ModuleDef,
{
    type Module = NativeModule<M, S>;

    fn into_module(self) -> Self::Module {
        self
    }
}

impl<S, M> IntoEsmModule for (S, M)
where
    S: AsRef<str>,
    M: ModuleDef,
{
    type Module = NativeModule<M, S>;

    fn into_module(self) -> Self::Module {
        NativeModule::new(self.0)
    }
}
