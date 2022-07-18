wit_bindgen_rust::export!("../loader.wit");

struct Loader;

impl loader::Loader for Loader {
    fn extension() -> Vec<String> {
        vec![String::from("ts"), String::from("tsx")]
    }

    fn transform(input: Vec<u8>) -> loader::Compilation {
        let source = match String::from_utf8(input) {
            Ok(source) => source,
            Err(err) => return loader::Compilation::Failure(err.to_string()),
        };

        match compile("module.ts", source) {
            Ok(ret) => loader::Compilation::Success(ret),
            Err(err) => loader::Compilation::Failure(err.to_string()),
        }
    }
}

use std::sync::Arc;
use swc::{
    common::{
        errors::{ColorConfig, Handler},
        FileName, SourceMap,
    },
    config::{JscConfig, Options},
    ecmascript::ast::EsVersion,
    // sourcemap::SourceMap,
};

fn compile(name: &str, source: impl ToString) -> Result<String, anyhow::Error> {
    let cm = Arc::new(SourceMap::default());

    let fm = cm.new_source_file(FileName::Custom(name.into()), source.to_string());

    let handler = Arc::new(Handler::with_tty_emitter(
        ColorConfig::Auto,
        true,
        false,
        Some(cm.clone()),
    ));

    let c = swc::Compiler::new(cm.clone());

    let output = c.process_js_file(
        fm,
        &handler,
        &Options {
            config: swc::config::Config {
                jsc: JscConfig {
                    target: EsVersion::Es2017.into(),
                    external_helpers: false.into(),
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
    )?;

    Ok(output.code)
}

fn main() {}
