use core::fmt;
use rquickjs::{Ctx, FromJs, IntoJs, Promise, RegisteryKey, Result};
use std::path::{Path, PathBuf};
use tokio::runtime::Builder;
use tokio::sync::{mpsc, oneshot};
use tokio::task::LocalSet;
use value_quickjs::value::Value;
use value_quickjs::Val;

use crate::{esm::EsmModulesBuilder, vm::VmBuilder};

// pub struct Val(Value);

// impl<'js> FromJs<'js> for Val {
//     fn from_js(ctx: rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
//         Ok(Val(value_quickjs::convert::from_js(ctx, value)?))
//     }
// }

// impl<'js> IntoJs<'js> for Val {
//     fn into_js(self, ctx: rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
//         Ok(value_quickjs::convert::into_js(ctx, self.0)?)
//     }
// }

pub enum QuickResult {
    Value(Val),
    RegistryKey(rquickjs::RegisteryKey),
}

impl QuickResult {
    pub fn into_key(self) -> RegisteryKey {
        match self {
            QuickResult::RegistryKey(key) => key,
            QuickResult::Value(_) => panic!("not key"),
        }
    }
}

impl From<Value> for QuickResult {
    fn from(value: Value) -> QuickResult {
        QuickResult::Value(value.into())
    }
}

impl From<RegisteryKey> for QuickResult {
    fn from(value: RegisteryKey) -> QuickResult {
        QuickResult::RegistryKey(value)
    }
}

enum Message {
    Kill,
    With {
        func: Box<dyn FnOnce(&Ctx) -> rquickjs::Result<QuickResult> + Send>,
        returns: oneshot::Sender<rquickjs::Result<QuickResult>>,
    },
    WithAsync {
        func: Box<dyn FnOnce(&Ctx) -> rquickjs::Result<Promise<Val>> + Send>,
        returns: oneshot::Sender<rquickjs::Result<Val>>,
    },
}

// #[derive(Clone)]
pub struct VmWorker {
    send: mpsc::UnboundedSender<Message>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl VmWorker {
    pub fn new<F: FnOnce(&mut EsmModulesBuilder) + 'static + Send>(init: F) -> Self {
        let (send, recv) = mpsc::unbounded_channel();

        let rt = Builder::new_current_thread().enable_all().build().unwrap();

        let handle = std::thread::spawn(move || {
            let local = LocalSet::new();

            local.spawn_local(run_vm(recv, init));

            // This will return once all senders are dropped and all
            // spawned tasks have returned.
            rt.block_on(local);
        });

        Self {
            send,
            handle: Some(handle),
        }
    }

    pub async fn with<F: FnOnce(&Ctx) -> rquickjs::Result<QuickResult> + 'static + Send>(
        &self,
        func: F,
    ) -> Result<QuickResult> {
        let (sx, rx) = oneshot::channel();

        self.send
            .send(Message::With {
                func: Box::new(func),
                returns: sx,
            })
            .ok();

        rx.await.unwrap()
    }

    pub async fn with_async<F: FnOnce(&Ctx) -> rquickjs::Result<Promise<Val>> + 'static + Send>(
        &self,
        func: F,
    ) -> Result<Value> {
        let (sx, rx) = oneshot::channel();

        self.send
            .send(Message::WithAsync {
                func: Box::new(func),
                returns: sx,
            })
            .ok();

        rx.await.unwrap().map(|m| m.into())
    }
}

async fn run_vm<F: FnOnce(&mut EsmModulesBuilder)>(
    mut recv: mpsc::UnboundedReceiver<Message>,
    init: F,
) -> Result<()> {
    let mut builder = VmBuilder::default();

    init(&mut builder);

    let vm = builder.build()?;

    while let Some(new_task) = recv.recv().await {
        match new_task {
            Message::Kill => break,
            Message::With { func, returns } => {
                vm.with(|ctx| {
                    let ret = match func(&ctx) {
                        Ok(ret) => Ok(ret),
                        Err(err) => Err(err),
                    };

                    returns.send(ret).ok();
                });
            }
            Message::WithAsync { func, returns } => {
                let ret = vm.with(|ctx| func(&ctx));

                let ret = match ret {
                    Ok(ret) => ret.await,
                    Err(err) => Err(err),
                };

                returns.send(ret).ok();
            }
        }
    }

    Result::Ok(())
}

// async fn run_scraper<P: Progress + Clone + 'static>(
//     vm: &Vm2,
//     ui: P,
//     scrapers: &ScraperCollection,
//     next: ScraperId,
//     actions: &async_channel::Sender<ScraperRequest>,
// ) {
//     let scraper = match scrapers.scraper(&next) {
//         Some(scraper) => scraper,
//         None => {
//             log::warn!("scraper with id: {:?} not found", next);
//             return;
//         }
//     };

//     ui.started(*scraper.id());

//     let ctx = RunCtx::new(next, &actions, &ui);

//     match vm.run_scraper(scraper, &ctx).await {
//         Ok(_) => {
//             ui.done(*scraper.id());
//         }
//         Err(err) => {
//             actions
//                 .send(ScraperRequest {
//                     scraper_id: next,
//                     task: Task::Error {
//                         path: RelativePathBuf::from(format!("{}.error", scraper.name())),
//                         data: format!("{}", err).as_bytes().to_vec(),
//                     },
//                 })
//                 .await
//                 .ok();

//             ui.error(*scraper.id(), err.into());
//         }
//     }
// }

impl Drop for VmWorker {
    fn drop(&mut self) {
        self.send.send(Message::Kill).ok();
        self.handle.take().unwrap().join().ok();
    }
}
