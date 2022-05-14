use super::{
    request::Request,
    response::{Response, ResponseState},
};
use reqwest::Client as HttpClient;
use rquickjs::{class_def, Async, Class, Ctx, Error, Func, Method};

#[derive(Default, Clone)]
pub struct Client {
    client: HttpClient,
}

class_def! {
    Client
    (proto) {
        proto.set("send", Func::from(Async(Method(|this: &Client, ctx: Ctx, req: Class<'_, Request>| {

            let req: &Request = req.as_ref();
            let req = req.create_http_request(ctx);

            let client = this.client.clone();

            async move {
                let req = match req {
                    Err(err) => return Err(err),
                    Ok(req) => req
                };

                let ret = client.execute(req).await.map_err(throw!())?;

                Result::<_, Error>::Ok(Response {
                    state: ResponseState::Uninit(ret)
                })
            }
        }))))?;
    }
}
