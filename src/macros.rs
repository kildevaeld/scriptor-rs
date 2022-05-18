#[allow(unused_macros)]
macro_rules! throw {
    ($error: expr) => {
        rquickjs::Error::Exception {
            message: $error.to_string(),
            file: String::default(),
            line: 0,
            stack: String::default(),
        }
    };
    () => {
        |err| throw!(err)
    };
}

#[cfg(any(feature = "fs", feature = "os"))]
macro_rules! stream {
    ($file: ty) => {
        impl rquickjs::ClassDef for $crate::JsStream<$file> {
            /// The name of a class
            const CLASS_NAME: &'static str = stringify!($file);

            /// The reference to class identifier
            ///
            /// # Safety
            /// This method should return reference to mutable static class id which should be initialized to zero.
            unsafe fn class_id() -> &'static mut rquickjs::ClassId {
                static mut CLASS_ID: rquickjs::ClassId = rquickjs::ClassId::new();
                &mut CLASS_ID
            }

            /// The class has prototype
            const HAS_PROTO: bool = true;

            /// The prototype initializer method
            fn init_proto<'js>(
                ctx: rquickjs::Ctx<'js>,
                proto: &rquickjs::Object<'js>,
            ) -> rquickjs::Result<()> {
                proto.set(
                    "next",
                    Func::from(rquickjs::Async(rquickjs::Method(
                        $crate::JsStream::<$file>::next,
                    ))),
                )?;

                let key: rquickjs::Symbol = ctx.eval("Symbol.asyncIterator")?;

                proto.set(
                    key,
                    Func::from(rquickjs::Method(|this: &$crate::JsStream<$file>| {
                        this.clone()
                    })),
                )?;

                Ok(())
            }

            /// The class has static data
            const HAS_STATIC: bool = false;
        }

        impl<'js> rquickjs::IntoJs<'js> for $crate::JsStream<$file> {
            fn into_js(self, ctx: rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
                use rquickjs::ClassDef;
                self.into_js_obj(ctx)
            }
        }

        impl<'js> rquickjs::FromJs<'js> for &'js $crate::JsStream<$file> {
            fn from_js(
                ctx: rquickjs::Ctx<'js>,
                value: rquickjs::Value<'js>,
            ) -> rquickjs::Result<Self> {
                use rquickjs::ClassDef;

                $crate::JsStream::<$file>::from_js_ref(ctx, value)
            }
        }

        impl<'js> rquickjs::FromJs<'js> for &'js mut $crate::JsStream<$file> {
            fn from_js(
                ctx: rquickjs::Ctx<'js>,
                value: rquickjs::Value<'js>,
            ) -> rquickjs::Result<Self> {
                use rquickjs::ClassDef;

                $crate::JsStream::<$file>::from_js_mut(ctx, value)
            }
        }

        impl<'js> rquickjs::FromJs<'js> for $crate::JsStream<$file> {
            fn from_js(
                ctx: rquickjs::Ctx<'js>,
                value: rquickjs::Value<'js>,
            ) -> rquickjs::Result<Self> {
                use rquickjs::ClassDef;

                $crate::JsStream::<$file>::from_js_obj(ctx, value)
            }
        }
    };
}

#[cfg(any(feature = "fs", feature = "os"))]
macro_rules! readwriter {
    ($file: ident) => {
        stream!(tokio_stream::wrappers::LinesStream<tokio::io::BufReader<$crate::FileDesc<$file>>>);
        impl ClassDef for $crate::FileDesc<$file> {
            /// The name of a class
            const CLASS_NAME: &'static str = <$file as $crate::Named>::NAME;

            /// The reference to class identifier
            ///
            /// # Safety
            /// This method should return reference to mutable static class id which should be initialized to zero.
            unsafe fn class_id() -> &'static mut ClassId {
                static mut CLASS_ID: ClassId = ClassId::new();
                &mut CLASS_ID
            }

            /// The class has prototype
            const HAS_PROTO: bool = true;

            /// The prototype initializer method
            fn init_proto<'js>(_ctx: Ctx<'js>, proto: &Object<'js>) -> Result<()> {
                proto.set(
                    "read",
                    Func::from(Async(Method($crate::FileDesc::<$file>::read))),
                )?;
                proto.set(
                    "lines",
                    Func::from(Method($crate::FileDesc::<$file>::lines)),
                )?;
                proto.set(
                    "write",
                    Func::from(Async(Method($crate::FileDesc::<$file>::write))),
                )?;
                proto.set(
                    "flush",
                    Func::from(Async(Method($crate::FileDesc::<$file>::flush))),
                )?;
                Ok(())
            }

            /// The class has static data
            const HAS_STATIC: bool = false;
        }

        impl<'js> IntoJs<'js> for $crate::FileDesc<$file> {
            fn into_js(self, ctx: Ctx<'js>) -> Result<Value<'js>> {
                self.into_js_obj(ctx)
            }
        }

        impl<'js> FromJs<'js> for &'js $crate::FileDesc<$file> {
            fn from_js(ctx: Ctx<'js>, value: Value<'js>) -> Result<Self> {
                $crate::FileDesc::<$file>::from_js_ref(ctx, value)
            }
        }

        impl<'js> FromJs<'js> for &'js mut $crate::FileDesc<$file> {
            fn from_js(ctx: Ctx<'js>, value: Value<'js>) -> Result<Self> {
                $crate::FileDesc::<$file>::from_js_mut(ctx, value)
            }
        }

        impl<'js> FromJs<'js> for $crate::FileDesc<$file> {
            fn from_js(ctx: Ctx<'js>, value: Value<'js>) -> Result<Self> {
                $crate::FileDesc::<$file>::from_js_obj(ctx, value)
            }
        }
    };
}

#[cfg(any(feature = "fs", feature = "os"))]
macro_rules! writer {
    ($file: ty) => {
        impl rquickjs::ClassDef for $crate::FileDesc<$file> {
            /// The name of a class
            const CLASS_NAME: &'static str = <$file as $crate::Named>::NAME;

            /// The reference to class identifier
            ///
            /// # Safety
            /// This method should return reference to mutable static class id which should be initialized to zero.
            unsafe fn class_id() -> &'static mut rquickjs::ClassId {
                static mut CLASS_ID: rquickjs::ClassId = rquickjs::ClassId::new();
                &mut CLASS_ID
            }

            /// The class has prototype
            const HAS_PROTO: bool = true;

            /// The prototype initializer method
            fn init_proto<'js>(
                _ctx: rquickjs::Ctx<'js>,
                proto: &rquickjs::Object<'js>,
            ) -> rquickjs::Result<()> {
                proto.set(
                    "write",
                    Func::from(rquickjs::Async(rquickjs::Method(
                        $crate::FileDesc::<$file>::write,
                    ))),
                )?;
                proto.set(
                    "flush",
                    Func::from(rquickjs::Async(rquickjs::Method(
                        $crate::FileDesc::<$file>::flush,
                    ))),
                )?;
                Ok(())
            }

            /// The class has static data
            const HAS_STATIC: bool = false;
        }

        impl<'js> rquickjs::IntoJs<'js> for $crate::FileDesc<$file> {
            fn into_js(self, ctx: rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
                use rquickjs::ClassDef;
                self.into_js_obj(ctx)
            }
        }

        impl<'js> rquickjs::FromJs<'js> for &'js $crate::FileDesc<$file> {
            fn from_js(
                ctx: rquickjs::Ctx<'js>,
                value: rquickjs::Value<'js>,
            ) -> rquickjs::Result<Self> {
                use rquickjs::ClassDef;
                $crate::FileDesc::<$file>::from_js_ref(ctx, value)
            }
        }

        impl<'js> rquickjs::FromJs<'js> for &'js mut $crate::FileDesc<$file> {
            fn from_js(
                ctx: rquickjs::Ctx<'js>,
                value: rquickjs::Value<'js>,
            ) -> rquickjs::Result<Self> {
                use rquickjs::ClassDef;
                $crate::FileDesc::<$file>::from_js_mut(ctx, value)
            }
        }

        impl<'js> rquickjs::FromJs<'js> for $crate::FileDesc<$file> {
            fn from_js(
                ctx: rquickjs::Ctx<'js>,
                value: rquickjs::Value<'js>,
            ) -> rquickjs::Result<Self> {
                use rquickjs::ClassDef;
                $crate::FileDesc::<$file>::from_js_obj(ctx, value)
            }
        }
    };
}

#[cfg(any(feature = "fs", feature = "os"))]
macro_rules! reader {
    ($file: ty) => {
        impl rquickjs::ClassDef for $crate::FileDesc<$file> {
            /// The name of a class
            const CLASS_NAME: &'static str = <$file as $crate::Named>::NAME;

            /// The reference to class identifier
            ///
            /// # Safety
            /// This method should return reference to mutable static class id which should be initialized to zero.
            unsafe fn class_id() -> &'static mut rquickjs::ClassId {
                static mut CLASS_ID: rquickjs::ClassId = rquickjs::ClassId::new();
                &mut CLASS_ID
            }

            /// The class has prototype
            const HAS_PROTO: bool = true;

            /// The prototype initializer method
            fn init_proto<'js>(
                _ctx: rquickjs::Ctx<'js>,
                proto: &rquickjs::Object<'js>,
            ) -> rquickjs::Result<()> {
                proto.set(
                    "read",
                    Func::from(rquickjs::Async(rquickjs::Method(
                        $crate::FileDesc::<$file>::read,
                    ))),
                )?;

                Ok(())
            }

            /// The class has static data
            const HAS_STATIC: bool = false;
        }

        impl<'js> rquickjs::IntoJs<'js> for $crate::FileDesc<$file> {
            fn into_js(self, ctx: rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
                use rquickjs::ClassDef;
                self.into_js_obj(ctx)
            }
        }

        impl<'js> rquickjs::FromJs<'js> for &'js $crate::FileDesc<$file> {
            fn from_js(
                ctx: rquickjs::Ctx<'js>,
                value: rquickjs::Value<'js>,
            ) -> rquickjs::Result<Self> {
                use rquickjs::ClassDef;

                $crate::FileDesc::<$file>::from_js_ref(ctx, value)
            }
        }

        impl<'js> rquickjs::FromJs<'js> for &'js mut $crate::FileDesc<$file> {
            fn from_js(
                ctx: rquickjs::Ctx<'js>,
                value: rquickjs::Value<'js>,
            ) -> rquickjs::Result<Self> {
                use rquickjs::ClassDef;

                $crate::FileDesc::<$file>::from_js_mut(ctx, value)
            }
        }

        impl<'js> rquickjs::FromJs<'js> for $crate::FileDesc<$file> {
            fn from_js(
                ctx: rquickjs::Ctx<'js>,
                value: rquickjs::Value<'js>,
            ) -> rquickjs::Result<Self> {
                use rquickjs::ClassDef;

                $crate::FileDesc::<$file>::from_js_obj(ctx, value)
            }
        }
    };
}

macro_rules! module_def {
    ($name: ident, $ident: ident) => {
        #[cfg(feature = "vm")]
        impl $crate::vm::IntoUserModule for $ident {
            type UserModule = $crate::vm::UserModuleImpl<$ident, &'static str>;
            fn into_module(self) -> Self::UserModule {
                $crate::vm::UserModuleImpl::new(stringify!($name), self)
            }
        }
    };
}
