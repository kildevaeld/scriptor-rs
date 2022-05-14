use rquickjs::{
    Accessor, Async, Class, ClassDef, ClassId, Ctx, FromJs, Func, IntoJs, Method, Object,
    Persistent, RefsMarker, Result, Value,
};

use super::headers::Headers;

#[derive(Debug)]

pub enum ResponseState {
    Uninit(reqwest::Response),
    Init {
        status: reqwest::StatusCode,
        headers: Persistent<Class<'static, Headers>>,
        resp: Option<reqwest::Response>,
    },
}

// #[bind(object)]
#[derive(Debug)]
pub struct Response {
    // headers: Persistent<Class<'static, Headers>>,
    // method: reqwest::Method,
    pub state: ResponseState,
}

impl ResponseState {
    pub fn take(&mut self) -> Result<reqwest::Response> {
        match self {
            ResponseState::Init { resp, .. } => match resp.take() {
                Some(resp) => Ok(resp),
                None => Err(throw!("body already taken")),
            },
            _ => Err(throw!("not initialized")),
        }
    }

    fn status(&self) -> Result<u64> {
        match self {
            ResponseState::Init { status, .. } => Ok(status.as_u16() as u64),
            _ => Err(throw!("invalid state")),
        }
    }

    fn headers(&self) -> Result<Persistent<Class<'static, Headers>>> {
        match self {
            ResponseState::Init { headers, .. } => Ok(headers.clone()),
            _ => Err(throw!("invalid state")),
        }
    }
}

impl Response {
    // pub fn new(ctx: Ctx) -> Response {
    //     Response {
    //         // headers: Persistent::save(ctx, Class::instance(ctx, Headers::default()).unwrap()),
    //         // method: reqwest::Method::GET,
    //         resp: None,
    //     }
    // }

    // pub async fn text(&mut self) -> Result<String, Error> {
    //     self.resp.take().unwrap().text().await.map_err(throw!())
    // }

    pub fn status(&self) -> Result<u64> {
        self.state.status()
    }

    fn headers(&self) -> Result<Persistent<Class<'static, Headers>>> {
        self.state.headers()
    }

    pub fn take(&mut self) -> Result<reqwest::Response> {
        self.state.take()
    }

    fn init(self, ctx: Ctx) -> Result<Response> {
        let resp = match self.state {
            ResponseState::Uninit(resp) => resp,
            _ => panic!("already intiaillized"),
        };

        let headers = Headers::from_http_headers(resp.headers())?;

        let headers = Class::instance(ctx, headers)?;

        Ok(Response {
            state: ResponseState::Init {
                status: resp.status(),
                resp: Some(resp),
                headers: Persistent::save(ctx, headers),
            },
        })
    }

    // pub fn headers(&self) -> Persistent<Class<'static, Headers>> {
    //     self.headers.clone()
    // }
}

// class_def! {
//     Response
//     (proto) {
//         proto.set("text", Func::from(Async(Method(|this: &mut Response, ctx: Ctx| {

//             //
//             let resp = match this.take() {
//                 Some(resp) => resp,
//                 None => panic!("")
//             };

//             async move {
//                 resp.text().await.map_err(throw!())
//             }

//         }))))?;
//     }

// ~(this, marker) {
//     // this.headers.mark_refs(marker);
//     // mark internal refs if exists
// }
// }

impl ClassDef for Response {
    const CLASS_NAME: &'static str = "Response";

    unsafe fn class_id() -> &'static mut ClassId {
        static mut CLASS_ID: ClassId = ClassId::new();
        &mut CLASS_ID
    }

    // With prototype
    const HAS_PROTO: bool = true;
    fn init_proto<'js>(_ctx: Ctx<'js>, proto: &Object<'js>) -> Result<()> {
        proto.prop("status", Accessor::from(Method(Response::status)))?;

        proto.prop("headers", Accessor::from(Method(Response::headers)))?;

        proto.set(
            "text",
            Func::from(Async(Method(|this: &mut Response, _ctx: Ctx| {
                //
                let resp = this.take();

                async move {
                    let resp = resp?;
                    resp.text().await.map_err(throw!())
                }
            }))),
        )?;
        Ok(())
    }

    // With statics
    const HAS_STATIC: bool = false;
    fn init_static<'js>(_ctx: Ctx<'js>, _ctor: &Object<'js>) -> Result<()> {
        Ok(())
    }

    // With internal references
    const HAS_REFS: bool = true;
    fn mark_refs(&self, _marker: &RefsMarker) {
        // marker.mark(&self.some_persistent_value);
    }

    fn into_js_obj<'js>(mut self, ctx: Ctx<'js>) -> Result<Value<'js>>
    where
        Self: Sized,
    {
        self = self.init(ctx)?;
        Class::<Self>::instance(ctx, self).map(|val| val.into_value())
    }
}

impl<'js> IntoJs<'js> for Response {
    fn into_js(self, ctx: Ctx<'js>) -> Result<Value<'js>> {
        self.into_js_obj(ctx)
    }
}

impl<'js> FromJs<'js> for &'js Response {
    fn from_js(ctx: Ctx<'js>, value: Value<'js>) -> Result<Self> {
        Response::from_js_ref(ctx, value)
    }
}

impl<'js> FromJs<'js> for &'js mut Response {
    fn from_js(ctx: Ctx<'js>, value: Value<'js>) -> Result<Self> {
        Response::from_js_mut(ctx, value)
    }
}

// impl<'js> FromJs<'js> for Response {
//     fn from_js(ctx: Ctx<'js>, value: Value<'js>) -> Result<Self> {
//         Response::from_js_obj(ctx, value)
//     }
// }
