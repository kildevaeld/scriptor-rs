use std::collections::BTreeMap;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use rquickjs::{class_def, Ctx, Error, Func, Method, Object, ObjectDef};

#[derive(Clone, Default)]
pub struct Headers {
    headers: BTreeMap<String, String>,
}

impl Headers {
    pub fn set(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
        // self.headers.append(key, HeaderValue::from_static(&value));
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a String, &'a String)> {
        self.headers.iter()
    }

    pub fn create_http_headers(&self) -> Result<HeaderMap, Error> {
        let mut map = HeaderMap::new();

        for (key, val) in self.iter() {
            let hv: HeaderValue = val.parse().unwrap();
            let hk: HeaderName = key.try_into().map_err(|_| throw!("invalid header name"))?;
            map.insert(hk, hv);
        }

        Ok(map)
    }

    pub fn from_http_headers(header_map: &HeaderMap) -> Result<Headers, Error> {
        let mut headers = BTreeMap::default();

        for (n, v) in header_map.iter() {
            headers.insert(
                n.to_string(),
                v.to_owned().to_str().map_err(throw!())?.to_string(),
            );
        }

        Ok(Headers { headers })
    }
}

class_def! {
    Headers
    (proto) {
        proto.set("set", Func::from(Method(Headers::set)))?;
    }
    // @(ctor) {
    //     ctor.set(
    //         "Headers",
    //         rquickjs::Func::new(
    //             "Headers",

    //                 rquickjs::Class::<Headers>::constructor(|| Headers::default())


    //         ),
    //     )?;
    // }
}

impl ObjectDef for Headers {
    fn init<'js>(_ctx: Ctx<'js>, ctor: &Object<'js>) -> rquickjs::Result<()> {
        ctor.set(
            "Headers",
            rquickjs::Func::new(
                "Headers",
                rquickjs::Class::<Headers>::constructor(|| Headers::default()),
            ),
        )?;

        Ok(())
    }
}
