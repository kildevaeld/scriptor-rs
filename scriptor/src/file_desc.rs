use futures_core::future::BoxFuture;
use rquickjs::{Result, TypedArray};
use std::{pin::Pin, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncSeek, AsyncWrite, AsyncWriteExt},
    sync::RwLock,
};

use crate::stream::JsStream;

pub struct FileDesc<F> {
    file: Arc<RwLock<F>>,
}

impl<F> FileDesc<F> {
    pub fn new(file: F) -> FileDesc<F> {
        FileDesc {
            file: Arc::new(RwLock::new(file)),
        }
    }
}

impl<F> Clone for FileDesc<F> {
    fn clone(&self) -> Self {
        FileDesc {
            file: self.file.clone(),
        }
    }
}

impl<F: AsyncRead + std::marker::Unpin + Send + 'static + Sync> FileDesc<F> {
    pub fn read(&mut self) -> BoxFuture<'static, Result<Vec<u8>>> {
        let file = self.file.clone();
        Box::pin(async move {
            let mut file = file.write().await;
            // let mut buf = Vec::with_capacity(1024);
            let mut buf: [u8; 1024] = [0; 1024];
            let read = file.read(&mut buf[..]).await.map_err(throw!())?;
            Ok(buf[0..read].to_vec())
        })
    }

    pub fn lines(
        &self,
    ) -> JsStream<tokio_stream::wrappers::LinesStream<tokio::io::BufReader<FileDesc<F>>>> {
        let this = self.clone();
        let file = tokio::io::BufReader::new(this);

        let stream = tokio_stream::wrappers::LinesStream::new(file.lines());

        JsStream::new(stream)
    }
}

impl<F: AsyncRead> AsyncRead for FileDesc<F> {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let mut inner = futures_lite::future::block_on(self.file.write());

        let inner = unsafe { Pin::new_unchecked(&mut *inner) };

        inner.poll_read(cx, buf)
    }
}

impl<F: AsyncWrite + std::marker::Unpin + Send + 'static + Sync> FileDesc<F> {
    pub fn write(&mut self, data: TypedArray<'_, u8>) -> BoxFuture<'static, Result<()>> {
        let file = self.file.clone();
        let data: &[u8] = data.as_ref();
        let data = data.to_vec();
        Box::pin(async move {
            let mut file = file.write().await;
            file.write_all(&data).await.map_err(throw!())
        })
    }

    pub fn write_str(&mut self, data: String) -> BoxFuture<'static, Result<()>> {
        let file = self.file.clone();

        Box::pin(async move {
            let mut file = file.write().await;
            file.write_all(data.as_bytes()).await.map_err(throw!())
        })
    }

    pub fn flush(&mut self) -> BoxFuture<'static, Result<()>> {
        let file = self.file.clone();
        Box::pin(async move {
            let mut file = file.write().await;
            file.flush().await.map_err(throw!())
        })
    }
}

impl<F: AsyncWrite + AsyncSeek + std::marker::Unpin + Send + 'static + Sync> FileDesc<F> {}

pub trait Named {
    const NAME: &'static str;
}
