use std::task::Poll;

use futures_core::{ready, Stream};
use rquickjs::{class_def, Accessor, Func, Method};

pub struct DirEntry {
    entry: tokio::fs::DirEntry,
}

class_def! {
    DirEntry
    (proto) {

        proto.prop("path", Accessor::from(Method(|this: &DirEntry| {
            this.entry.path().as_os_str().to_string_lossy().to_string()
        })))?;

    }
}

pin_project_lite::pin_project! {
    pub struct ReadDir {
        #[pin]
        pub dir: tokio::fs::ReadDir,
    }
}

impl Stream for ReadDir {
    type Item = Result<DirEntry, rquickjs::Error>;
    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        match ready!(this.dir.poll_next_entry(cx)) {
            Ok(Some(entry)) => Poll::Ready(Some(Ok(DirEntry { entry }))),
            Ok(None) => Poll::Ready(None),
            Err(err) => Poll::Ready(Some(Err(throw!(err)))),
        }
    }
}

stream!(ReadDir);
