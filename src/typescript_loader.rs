use std::sync::Arc;

use rquickjs::{Error, Loader, Module};

use swc::{
    common::{
        errors::{ColorConfig, Handler},
        FileName, SourceMap,
    },
    config::{JscConfig, Options},
    ecmascript::ast::EsVersion,
    // sourcemap::SourceMap,
};

pub struct TypescriptFileLoader {
    sm: Arc<SourceMap>,
    handler: Arc<Handler>,
}

// impl fmt::Debug for TypescriptFileLoader {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         Ok(())
//     }
// }

impl Default for TypescriptFileLoader {
    fn default() -> TypescriptFileLoader {
        let sm = Arc::new(SourceMap::default());

        let handler = Arc::new(Handler::with_tty_emitter(
            ColorConfig::Auto,
            true,
            false,
            Some(sm.clone()),
        ));

        TypescriptFileLoader { sm, handler }
    }
}

impl Loader for TypescriptFileLoader {
    fn load<'js>(
        &mut self,
        ctx: rquickjs::Ctx<'js>,
        path: &str,
    ) -> rquickjs::Result<rquickjs::Module<'js, rquickjs::Loaded>> {
        let source = std::fs::read_to_string(&path)?;

        let c = swc::Compiler::new(self.sm.clone());

        let fm = self
            .sm
            .new_source_file(FileName::Custom(path.into()), source);

        let output = c
            .process_js_file(
                fm,
                &self.handler,
                &Options {
                    config: swc::config::Config {
                        jsc: JscConfig {
                            target: EsVersion::Es2017.into(),
                            external_helpers: false,
                            syntax: Some(swc_ecma_parser::Syntax::Typescript(
                                swc_ecma_parser::TsConfig {
                                    tsx: true,

                                    ..Default::default()
                                },
                            )),

                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .map_err(|err| Error::Loading {
                name: path.to_string(),
                message: Some(err.to_string()),
            })?;

        Ok(Module::new(ctx, path, output.code)?.into_loaded())
    }
}

pub fn compile(name: &str, source: impl ToString) -> Result<String, Error> {
    let cm = Arc::new(SourceMap::default());

    let fm = cm.new_source_file(FileName::Custom(name.into()), source.to_string());

    let handler = Arc::new(Handler::with_tty_emitter(
        ColorConfig::Auto,
        true,
        false,
        Some(cm.clone()),
    ));

    let c = swc::Compiler::new(cm.clone());

    let output = c
        .process_js_file(
            fm,
            &handler,
            &Options {
                config: swc::config::Config {
                    minify: true,
                    jsc: JscConfig {
                        syntax: Some(swc_ecma_parser::Syntax::Typescript(
                            swc_ecma_parser::TsConfig {
                                tsx: true,
                                ..Default::default()
                            },
                        )),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .map_err(|err| Error::Loading {
            name: name.to_string(),
            message: Some(err.to_string()),
        })?;

    Ok(output.code)
}
