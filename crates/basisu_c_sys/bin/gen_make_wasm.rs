//! This is a command line tool to generate Makefile for building basisu wasm using Emscripten.

include!(concat!(env!("OUT_DIR"), "/build_encoder_emcc_args.rs"));
include!(concat!(env!("OUT_DIR"), "/build_transcoder_emcc_args.rs"));
include!(concat!(env!("OUT_DIR"), "/build_encoder_sources.rs"));
include!(concat!(env!("OUT_DIR"), "/build_transcoder_sources.rs"));

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Extra flags to pass to emcc.
    #[arg(long)]
    emcc_flags: Option<String>,
    /// Enable wasm-opt and pass extra flags to it.
    #[arg(long)]
    wasm_opt_flags: Option<String>,
}

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    std::env::set_current_dir(manifest_dir).unwrap();

    let (mut encoder_args, mut transcoder_args) = (
        DEFAULT_ENCODER_EMCC_ARGS.to_vec(),
        DEFAULT_TRANSCODER_EMCC_ARGS.to_vec(),
    );
    let user_args = Args::parse();
    if let Some(flags) = &user_args.emcc_flags {
        encoder_args.extend(flags.split(" ").filter(|s| !s.is_empty()));
        transcoder_args.extend(flags.split(" ").filter(|s| !s.is_empty()));
    }

    let mut makefile = format!(
        r#"
CXX = em++
CXXFLAGS = {}

ENCODER_LINKFLAGS = {}
TRANSCODER_LINKFLAGS = {}


ENCODER_SOURCES = {}
TRANSCODER_SOURCES = {}

ENCODER_BUILD_DIR = build/encoder
ENCODER_OBJECT_FILENAMES = $(patsubst %.c, %.o, $(patsubst %.cpp, %.o, $(notdir $(ENCODER_SOURCES))))
ENCODER_OBJECTS = $(addprefix $(ENCODER_BUILD_DIR)/, $(ENCODER_OBJECT_FILENAMES))

TRANSCODER_BUILD_DIR = build/transcoder
TRANSCODER_OBJECT_FILENAMES = $(patsubst %.c, %.o, $(patsubst %.cpp, %.o, $(notdir $(TRANSCODER_SOURCES))))
TRANSCODER_OBJECTS = $(addprefix $(TRANSCODER_BUILD_DIR)/, $(TRANSCODER_OBJECT_FILENAMES))

all: wasm/basisu_encoder.wasm wasm/basisu_encoder.js wasm/basisu_transcoder.wasm wasm/basisu_transcoder.js

.PHONY: clean
clean:
	rm -rf build

build_dir:
	mkdir -p build/encoder
	mkdir -p build/transcoder

build/basisu_encoder.wasm: $(ENCODER_OBJECTS) | build_dir
	$(CXX) $(ENCODER_LINKFLAGS) $^ -o $(patsubst %.wasm, %.js, $@)

build/basisu_transcoder.wasm: $(TRANSCODER_OBJECTS) | build_dir
	$(CXX) $(TRANSCODER_LINKFLAGS) $^ -o $(patsubst %.wasm, %.js, $@)
"#,
        encoder_args
            .clone()
            .into_iter()
            .filter(|f| ["-std=c++17", "-sSTRICT"].contains(f) || !f.starts_with("-s"))
            .collect::<Vec<&str>>()
            .join(" "),
        encoder_args
            .clone()
            .into_iter()
            .filter(|f| *f != ("-xc++"))
            .collect::<Vec<&str>>()
            .join(" "),
        transcoder_args
            .clone()
            .into_iter()
            .filter(|f| *f != ("-xc++"))
            .collect::<Vec<&str>>()
            .join(" "),
        ENCODER_SOURCES.join(" "),
        TRANSCODER_SOURCES.join(" "),
    );

    for (name, sources) in [
        ("encoder", ENCODER_SOURCES),
        ("transcoder", TRANSCODER_SOURCES),
    ] {
        for src in sources.iter() {
            let src_path = std::path::Path::new(src);
            makefile.push_str(&format!(
                r#"
{}: {} | build_dir
	$(CXX) $(CXXFLAGS) -c $< -o $@
"#,
                "build/".to_string()
                    + name
                    + "/"
                    + src_path
                        .with_extension("o")
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap(),
                src,
            ));
        }
    }

    if let Some(flags) = user_args.wasm_opt_flags {
        let wasm_opt_args = flags
            .split(" ")
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        makefile.push_str(&format!(
            r#"
WASM_OPT_FLAGS = {}

wasm/basisu_encoder.wasm: build/basisu_encoder.wasm
	wasm-opt $(WASM_OPT_FLAGS) $^ -o $@

wasm/basisu_transcoder.wasm: build/basisu_transcoder.wasm
	wasm-opt $(WASM_OPT_FLAGS) $^ -o $@
        "#,
            wasm_opt_args.join(" ")
        ));
    } else {
        makefile.push_str(
            r#"
wasm/basisu_encoder.wasm: build/basisu_encoder.wasm
	cp $^ $@

wasm/basisu_transcoder.wasm: build/basisu_transcoder.wasm
	cp $^ $@
    "#,
        );
    }
    makefile.push_str(
        r#"
wasm/basisu_encoder.js: build/basisu_encoder.wasm
	cp $(patsubst %.wasm, %.js, $^) $@

wasm/basisu_transcoder.js: build/basisu_transcoder.wasm
	cp $(patsubst %.wasm, %.js, $^) $@
"#,
    );
    std::fs::write("Makefile", makefile).unwrap();
}
