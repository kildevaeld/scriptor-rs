use super::Headers;
use rquickjs::{class_def, Accessor, Class, Ctx, Error, Func, HasRefs, Method, Persistent};
#[derive(Clone, Debug)]
pub struct Url {
    url: reqwest::Url,
}

impl Url {
    pub fn new(url: String) -> Result<Url, Error> {
        let url = reqwest::Url::parse(&url).map_err(throw!())?;

        Ok(Url { url })
    }
}

impl ToString for Url {
    fn to_string(&self) -> String {
        self.url.to_string()
    }
}

class_def! {
    Url
    (prop) {
        prop.set("toString", Func::from(Method(Url::to_string)))?;
    }
}

// #[bind(object)]
#[derive(Clone, Debug)]
pub struct Request {
    pub url: Persistent<Class<'static, Url>>,
    pub headers: Persistent<Class<'static, Headers>>,
    pub method: reqwest::Method,
}

impl Request {
    pub fn new<'js>(ctx: Ctx<'js>, url: Class<'js, Url>) -> Request {
        Request {
            url: Persistent::save(ctx, url),
            headers: Persistent::save(ctx, Class::instance(ctx, Headers::default()).unwrap()),
            method: reqwest::Method::GET,
        }
    }

    pub fn get_headers(&self) -> Persistent<Class<'static, Headers>> {
        self.headers.clone()
    }

    pub fn set_headers<'js>(&mut self, ctx: Ctx<'js>, headers: Class<'js, Headers>) {
        self.headers = Persistent::save(ctx, headers);
    }

    pub fn set_method(&mut self, method: String) {
        use reqwest::Method;

        self.method = match method.as_str() {
            "GET" | "get" => Method::GET,
            "POST" | "post" => Method::POST,
            "PUT" | "put" => Method::PUT,
            _ => Method::from_bytes(method.as_bytes()).unwrap(),
        };
    }

    pub fn get_method(&self) -> String {
        self.method.to_string()
    }

    pub fn create_http_request(&self, ctx: Ctx) -> Result<reqwest::Request, Error> {
        let url = self.url.clone().restore(ctx)?;
        let url: &Url = url.as_ref();
        let mut req = reqwest::Request::new(self.method.clone(), url.url.clone());

        let headers = self.headers.clone().restore(ctx)?;
        let headers: &Headers = headers.as_ref();

        let headers = headers.create_http_headers()?;

        *req.headers_mut() = headers;

        Ok(req)
    }
}

class_def! {
    Request
    (proto) {
        proto.prop("headers", Accessor::new(Method(Request::get_headers), Method(Request::set_headers)))?;
        proto.prop("method", Accessor::new(
            Method(Request::get_method),
            Method(Request::set_method)
        ))?;
    }

    ~(this, marker) {
        this.headers.mark_refs(marker);
        this.url.mark_refs(marker);
        // this.headers.mark_refs(marker);
        // mark internal refs if exists
    }
}
