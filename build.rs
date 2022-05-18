fn main() {
    println!("cargo:rerun-if-changed=lib/pipe.ts");
    println!("cargo:rerun-if-changed=lib/util.ts");
    println!("cargo:rerun-if-changed=lib/main.ts");
    println!("cargo:rerun-if-changed=lib/tasks.ts");

    std::fs::write("lib/util.js", compile_jsx("lib/util.ts")).expect("write file");
    std::fs::write("lib/pipe.js", compile_jsx("lib/pipe.ts")).expect("write file");
    std::fs::write("lib/main.js", compile_jsx("lib/main.ts")).expect("write file");
    std::fs::write("lib/tasks.js", compile_jsx("lib/tasks.ts")).expect("write file");
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

pub fn compile_jsx(path: &str) -> String {
    let source = std::fs::read_to_string(path).expect("");

    let cm = Arc::new(SourceMap::default());

    let fm = cm.new_source_file(FileName::Custom("temp_file_name".into()), source);

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
                        target: Some(EsVersion::Es2020),
                        syntax: Some(swc_ecma_parser::Syntax::Typescript(
                            swc_ecma_parser::TsConfig {
                                tsx: true,

                                ..Default::default()
                            },
                        )),
                        external_helpers: false,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .expect("failed to process file");

    output.code
}
