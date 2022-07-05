use rquickjs::{
    Async, ClassDef, ClassId, Ctx, FromJs, Func, IntoJs, Method, Object, Result, Value,
};
use tokio::fs::File as TokioFile;

use crate::file_desc::Named;

impl Named for TokioFile {
    const NAME: &'static str = "File";
}

readwriter!(TokioFile);
