use std::path::{Path, PathBuf};
use std::{env, fs};

/// Emscripten's pre-generated wasm32 arch headers.
const EMSCRIPTEN_ARCH: &str = "vendored/emscripten/system/lib/libc_musl_arch_emscripten";

/// Our replacements for the emscripten arch headers that pull in wasi/JS
/// syscall glue (`syscall_arch.h`, `pthread_arch.h`); `atomic_arch.h` is a
/// copy of emscripten's (self-contained, uses C11 atomics only).
const MUSL_ARCH_SHIM: &str = "src/wasm_ffi/c/musl_arch";

fn parse_dir<T: AsRef<Path>>(path: T, sources: &mut Vec<PathBuf>, ext: &str) {
    let Ok(dirs) = fs::read_dir(path) else {
        return;
    };

    for dir in dirs {
        let dir = dir.unwrap();
        let path = dir.path();

        if path.is_file()
            && let Some(extension) = path.extension()
            && extension == ext
        {
            sources.push(path);
        }
    }
}

/// Copy all .h files from `src_dir` into `dest_dir`.
fn copy_headers(src_dir: &str, dest_dir: &Path) {
    let Ok(entries) = fs::read_dir(src_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "h")
            && let Some(name) = path.file_name()
        {
            fs::copy(&path, dest_dir.join(name)).ok();
        }
    }
}

/// Include paths for downstream crates (libc++, basisu). Order matters:
/// `vendored/musl/include` before `vendored/musl/src/include` so `<stdio.h>`
/// is the public one (`extern FILE *stdout`) and the internal
/// `__stdout_FILE` redirects in `src/include/stdio.h` stay inactive.
pub fn includes() -> [PathBuf; 5] {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let generated_include = out_dir.join("include");
    [
        generated_include,
        "vendored/musl/include".into(),
        "vendored/musl/src/include".into(),
        "vendored/musl/src/internal".into(),
        "src/wasm_ffi/c".into(),
    ]
}

/// Include paths for compiling the musl sources themselves — mirrors musl's
/// own Makefile order (`src/include` before `include`, so `<features.h>`
/// provides `weak_alias` and internal redirects are active), plus our arch
/// shim ahead of the staged `bits/` headers.
fn musl_includes() -> Vec<PathBuf> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    vec![
        out_dir.join("include"),
        MUSL_ARCH_SHIM.into(),
        // empty fp_arch.h (soft-float default) used by internal/libm.h
        "vendored/musl/arch/generic".into(),
        "vendored/musl/src/include".into(),
        "vendored/musl/src/internal".into(),
        "vendored/musl/include".into(),
        EMSCRIPTEN_ARCH.into(),
        "src/wasm_ffi/c".into(),
    ]
}

/// Subdirectories of `vendored/musl/src` compiled wholesale (every .c file).
/// All are self-contained (no syscalls, no mmap, no threads):
///
/// - `string/` — strcmp/strcpy/strlen/strnlen/strcasecmp/strdup/strstr/
///   wcslen/wmemchr/... (this fixes most of the previously hand-written
///   string functions)
/// - `ctype/` — tolower/toupper/isalpha/... + `__ctype_get_mb_cur_max`
/// - `math/` — the full musl math library (nextafterf, lrintf, ...)
/// - `multibyte/` — mbrtowc/wcrtomb/mbsnrtowcs/... (UTF-8 via the
///   single-threaded `__get_tp` glue in wasm_libc_shim.c)
const MUSL_WHOLESALE: &[&str] = &["string", "ctype", "math", "multibyte"];

/// Individual musl sources (subset of `stdlib/`, `misc/`, `locale/`,
/// `stdio/`, `internal/`). `stdio/__toread.c`+`__uflow.c`+`vfscanf.c`+
/// `vsscanf.c`+`sscanf.c` and `internal/` scan helpers give a real `sscanf`
/// (used by libc++'s locale machinery).
const MUSL_FILES: &[&str] = &[
    // stdlib — numeric conversion (malloc stays Rust-side dlmalloc)
    "stdlib/atof.c",
    "stdlib/atoi.c",
    "stdlib/strtod.c",
    "stdlib/strtol.c",
    "stdlib/wcstod.c",
    "stdlib/wcstol.c",
    // exit — __cxa_atexit/atexit (replaces the hand-written Rust registry)
    "exit/atexit.c", // misc — basename/dirname (used by basisu)
    "misc/basename.c",
    "misc/dirname.c",
    // locale — the _l variants simply forward to the plain functions
    "locale/strtod_l.c",
    // fenv — dummy implementations for archs without an FP environment
    // (wasm has no runtime rounding-mode register, so fegetround returns
    // FE_TONEAREST). Referenced by math/fmaf.c and math/fmal.c, which are
    // compiled wholesale from the math/ directory above.
    "fenv/fenv.c",
    "fenv/fesetround.c",
    // stdio — the scanf family (needs the internal scan helpers below)
    "stdio/sscanf.c",
    "stdio/vsscanf.c",
    "stdio/vfscanf.c",
    "stdio/__uflow.c",
    "stdio/__toread.c",
    // internal — scanf/strto* scan helpers
    "internal/intscan.c",
    "internal/floatscan.c",
    "internal/shgetc.c",
];

pub fn main() {
    // Stage arch-specific bits/ headers into OUT_DIR/include/bits/.
    // Uses emscripten's pre-generated wasm32 headers (matching how emscripten
    // builds musl), with generic fallbacks underneath.
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bits_dir = out_dir.join("include").join("bits");
    fs::create_dir_all(&bits_dir).expect("failed to create bits dir");

    // Layer bits/ headers: our musl generic as base, then emscripten's wasm32
    // arch headers on top. Only errno.h is skipped (references wasi/api.h).
    copy_headers("vendored/musl/arch/generic/bits", &bits_dir);
    let emsc_bits = format!("{EMSCRIPTEN_ARCH}/bits");
    if let Ok(entries) = fs::read_dir(&emsc_bits) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().unwrap().to_str().unwrap_or_default();
            // errno.h references wasi/api.h — keep our musl generic version
            if name == "errno.h" {
                continue;
            }
            if path.extension().is_some_and(|ext| ext == "h") {
                fs::copy(&path, bits_dir.join(name)).ok();
            }
        }
    }

    let generated_include = out_dir.join("include");

    // The actual libc comes from musl C sources (below) plus a small set of
    // Rust files (src/wasm_ffi/rust/*.rs: malloc/itoa/atexit/signal) and C
    // shims (errno, nanoprintf, stdio_shim, wasm_libc_shim).
    //
    // The wholesale dirs + explicit files mirror the "compile musl sources
    // directly" approach of sqlite-wasm-rs instead of hand-writing every
    // string/math function in Rust.
    let mut sources: Vec<PathBuf> = vec![];
    for dir in MUSL_WHOLESALE {
        parse_dir(format!("vendored/musl/src/{dir}"), &mut sources, "c");
    }
    for f in MUSL_FILES {
        sources.push(format!("vendored/musl/src/{f}").into());
    }

    let mut build = cc::Build::new();
    build
        .includes(musl_includes())
        // musl's own build flags
        .flag("-D_XOPEN_SOURCE=700")
        .flag("-D_GNU_SOURCE")
        .flag_if_supported("-Wno-macro-redefined")
        .flag_if_supported("-w")
        .std("c17")
        // our own C: errno + shims (nanoprintf/stdio_shim provide the
        // printf family; wasm_libc_shim provides pthread/time/FILE/locale/
        // setjmp stubs + the single-threaded __get_tp glue)
        .file("src/wasm_ffi/c/errno.c")
        .file("src/wasm_ffi/c/nanoprintf.c")
        .file("src/wasm_ffi/c/stdio_shim.c")
        .file("src/wasm_ffi/c/wasm_libc_shim.c")
        .files(sources)
        .compile("wasm32-libc");

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    // Export both include paths separated by ':' for downstream crates
    println!(
        "cargo::metadata=include={}:{}/musl/include",
        generated_include.display(),
        manifest_dir,
    );
}
