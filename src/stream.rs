use std::sync::Arc;

use futures_core::{future::BoxFuture, Stream};
use futures_lite::StreamExt;

use rquickjs::{embed, IntoJs};
use tokio::sync::Mutex;

#[derive(IntoJs)]
pub struct Next<T> {
    done: bool,
    value: Option<T>,
}

pub struct JsStream<S> {
    s: Arc<Mutex<S>>,
}

impl<S> Clone for JsStream<S> {
    fn clone(&self) -> Self {
        JsStream { s: self.s.clone() }
    }
}

impl<S> JsStream<S> {
    pub fn new(stream: S) -> JsStream<S> {
        JsStream {
            s: Arc::new(Mutex::new(stream)),
        }
    }
}

impl<S> JsStream<S>
where
    S: Stream + std::marker::Unpin + Send + 'static,
    for<'js> S::Item: IntoJs<'js>,
    S::Item: Send,
{
    pub fn next(&self) -> BoxFuture<'static, Next<S::Item>> {
        let stream = self.s.clone();
        Box::pin(async move {
            let mut stream = stream.lock().await;

            match stream.next().await {
                Some(next) => Next {
                    done: false,
                    value: Some(next),
                },
                None => Next {
                    done: true,
                    value: None,
                },
            }
        })
    }
}

#[embed(path = "src", public)]
pub mod pipe {}
