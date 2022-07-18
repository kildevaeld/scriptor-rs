mod client;
mod headers;
mod request;
mod response;

use rquickjs::{Class, Error, Module as QuickModule, ModuleDef};

use client::Client;
use headers::Headers;
use request::Request;
use response::Response;

pub use request::Url;

use crate::esm::NativeModule;

pub struct Module;

impl ModuleDef for Module {
    fn load<'js>(
        _ctx: rquickjs::Ctx<'js>,
        module: &QuickModule<'js, rquickjs::Created>,
    ) -> rquickjs::Result<()> {
        module.add("Url")?;
        module.add("Headers")?;
        module.add("Request")?;
        module.add("Response")?;
        module.add("Client")?;

        Ok(())
    }

    fn eval<'js>(
        ctx: rquickjs::Ctx<'js>,
        module: &QuickModule<'js, rquickjs::Loaded<rquickjs::Native>>,
    ) -> rquickjs::Result<()> {
        Class::<Headers>::register(ctx)?;
        Class::<Request>::register(ctx)?;
        Class::<Response>::register(ctx)?;
        Class::<Url>::register(ctx)?;
        Class::<Client>::register(ctx)?;

        module.set(
            "Url",
            rquickjs::Func::new("Url", rquickjs::Class::<Url>::constructor(Url::new)),
        )?;

        module.set(
            "Headers",
            rquickjs::Func::new(
                "Headers",
                rquickjs::Class::<Headers>::constructor(Headers::default),
            ),
        )?;

        module.set(
            "Request",
            rquickjs::Func::new(
                "Request",
                rquickjs::Class::<Request>::constructor(Request::new),
            ),
        )?;

        module.set(
            "Response",
            rquickjs::Func::new(
                "Response",
                rquickjs::Class::<Response>::constructor(|| {
                    Result::<Response, Error>::Err(Error::Unknown)
                }),
            ),
        )?;

        module.set(
            "Client",
            rquickjs::Func::new(
                "Client",
                rquickjs::Class::<Client>::constructor(Client::default),
            ),
        )?;

        Ok(())
    }
}

pub const HTTP: NativeModule<Module, &'static str> = NativeModule::new("http");
