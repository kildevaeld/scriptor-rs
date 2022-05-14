use std::time::Duration;

use rquickjs::{class_def, Async, Class, Ctx, Func, Method, Result, TypedArray, Value as JsValue};

pub struct TextEncoder;

impl TextEncoder {
    pub fn encode<'js>(&self, ctx: Ctx<'js>, string: String) -> Result<JsValue<'js>> {
        TypedArray::new_copy(ctx, string.as_bytes()).map(|m| m.into_value())
    }
}

class_def! {
    TextEncoder
    (proto) {
        proto.set("encode", Func::from(Method(TextEncoder::encode)))?;
    }
}

pub struct TextDecoder;

impl TextDecoder {
    pub fn decode<'js>(&self, ctx: Ctx<'js>, string: Vec<u8>) -> Result<String> {
        let s = String::from_utf8(string)?;
        Ok(s)
    }
}

class_def! {
    TextDecoder
    (proto) {
        proto.set("decode", Func::from(Method(TextDecoder::decode)))?;
    }
}

pub fn init(ctx: Ctx<'_>) -> Result<()> {
    Class::<TextEncoder>::register(ctx)?;
    Class::<TextDecoder>::register(ctx)?;

    ctx.globals().set(
        "TextEncoder",
        Func::new(
            "TextEncoder",
            Class::<TextEncoder>::constructor(|| TextEncoder),
        ),
    )?;

    ctx.globals().set(
        "TextDecoder",
        Func::new(
            "TextDecoder",
            Class::<TextDecoder>::constructor(|| TextDecoder),
        ),
    )?;

    ctx.globals().set(
        "delay",
        Func::new(
            "delay",
            Async(|number: u64| async move {
                tokio::time::sleep(Duration::from_millis(number)).await;
            }),
        ),
    )?;

    Ok(())
}
