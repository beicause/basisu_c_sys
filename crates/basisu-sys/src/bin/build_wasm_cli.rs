include!(concat!(env!("OUT_DIR"), "/build_wasm_emcc_args.rs"));

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

pub fn build_wasm_cmd() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    std::env::set_current_dir(manifest_dir).unwrap();

    let mut emcc_cmd = std::process::Command::new("em++");
    emcc_cmd.args(DEFAULT_EMCC_ARGS);

    let user_args = Args::parse();
    if let Some(flags) = user_args.emcc_flags {
        emcc_cmd.args(flags.split(" ").filter(|s| !s.is_empty()));
    }

    println!(
        "Building basisu vendor wasm: {:#?}",
        [emcc_cmd.get_program()]
            .iter()
            .map(|s| s.to_string_lossy().to_string())
            .chain(emcc_cmd.get_args().map(|s| s.to_string_lossy().to_string()))
            .collect::<Vec<_>>()
            .join(" ")
    );

    let exit_status = emcc_cmd.spawn().unwrap().wait().unwrap();
    if !exit_status.success() {
        panic!("emcc didn't exit with success status: {}", exit_status);
    } else {
        println!("Build basisu vendor wasm successfully");
    }

    if let Some(flags) = user_args.wasm_opt_flags {
        let mut wasm_opt_cmd = std::process::Command::new("wasm-opt");
        wasm_opt_cmd.args([
            "--enable-simd",
            "--enable-bulk-memory-opt",
            "--enable-nontrapping-float-to-int",
            "wasm/basisu_vendor.wasm",
            "-o",
            "wasm/basisu_vendor.wasm",
        ]);

        wasm_opt_cmd.args(flags.split(" ").filter(|s| !s.is_empty()));

        println!(
            "Optimizing basisu vendor wasm: {:#?}",
            [wasm_opt_cmd.get_program()]
                .iter()
                .map(|s| s.to_string_lossy().to_string())
                .chain(
                    wasm_opt_cmd
                        .get_args()
                        .map(|s| s.to_string_lossy().to_string())
                )
                .collect::<Vec<_>>()
                .join(" ")
        );

        let exit_status = wasm_opt_cmd.spawn().unwrap().wait().unwrap();
        if !exit_status.success() {
            panic!("wasm-opt didn't exit with success status: {}", exit_status);
        } else {
            println!("Optimize basisu vendor wasm successfully");
        }
    }
}

fn main() {
    build_wasm_cmd();
}
