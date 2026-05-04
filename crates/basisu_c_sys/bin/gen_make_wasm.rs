//! This is a command line tool to generate Makefile for building basisu wasm using Emscripten.

include!(concat!(env!("OUT_DIR"), "/build_encoder_emcc_args.rs"));
include!(concat!(env!("OUT_DIR"), "/build_transcoder_emcc_args.rs"));
include!(concat!(env!("OUT_DIR"), "/build_encoder_sources.rs"));
include!(concat!(env!("OUT_DIR"), "/build_transcoder_sources.rs"));

use std::path::Path;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Extra flags to pass to emcc.
    #[arg(long)]
    emcc_flags: Option<String>,
}

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    std::env::set_current_dir(Path::new(&manifest_dir).join("build_args")).unwrap();

    let (mut encoder_args, mut transcoder_args) = (
        DEFAULT_ENCODER_EMCC_ARGS.to_vec(),
        DEFAULT_TRANSCODER_EMCC_ARGS.to_vec(),
    );
    let user_args = Args::parse();
    if let Some(flags) = &user_args.emcc_flags {
        encoder_args.extend(flags.split(" ").filter(|s| !s.is_empty()));
        transcoder_args.extend(flags.split(" ").filter(|s| !s.is_empty()));
    }

    for (name, sources) in [
        ("encoder_srcs.txt", ENCODER_SOURCES),
        ("transcoder_srcs.txt", TRANSCODER_SOURCES),
    ] {
        std::fs::write(name, sources.join("\n")).unwrap();
    }

    for (name, sources) in [
        ("encoder_link_flags.txt", &encoder_args),
        ("transcoder_link_flags.txt", &transcoder_args),
    ] {
        std::fs::write(name, sources.join("\n")).unwrap();
    }

    for (name, sources) in [
        ("encoder_compile_flags.txt", encoder_args),
        ("transcoder_compile_flags.txt", transcoder_args),
    ] {
        std::fs::write(
            name,
            sources
                .into_iter()
                .filter(|f| !f.starts_with("-s") || *f == "-sSTRICT")
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .unwrap();
    }
}
