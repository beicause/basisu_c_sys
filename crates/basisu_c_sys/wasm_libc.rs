use std::path::{Path, PathBuf};
use std::{env, fs};

/// Emscripten's pre-generated wasm32 arch headers.
const EMSCRIPTEN_ARCH: &str = "vendored/emscripten/system/lib/libc_musl_arch_emscripten";

/// Our replacements for the emscripten arch headers that pull in wasi/JS
/// syscall glue (`syscall_arch.h`, `pthread_arch.h`); `atomic_arch.h` is a
/// copy of emscripten's (self-contained, uses C11 atomics only).
const MUSL_ARCH_SHIM: &str = "src/wasm_ffi/c/musl_arch";

/// The musl C sources come from emscripten's fork of musl 1.2.6
/// (`3rdparty/emscripten/system/lib/libc/musl`, via the `vendored/musl`
/// symlink), compiled as a freestanding libc — same tree, same layout as
/// upstream musl 1.2.6 for everything this crate compiles.
const MUSL_ROOT: &str = "vendored/musl";

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

fn copy_tree(src: &Path, dest: &Path) {
    fs::create_dir_all(dest).ok();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let from = entry.path();
        let to = dest.join(entry.file_name());
        if from.is_dir() {
            copy_tree(&from, &to);
        } else {
            fs::copy(&from, &to).ok();
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
/// the musl `include` before musl `src/include` so `<stdio.h>` is the public
/// one (`extern FILE *stdout`) and the internal `__stdout_FILE` redirects in
/// `src/include/stdio.h` stay inactive.
///
/// Downstream compiles against the vendored musl tree directly — the two
/// files `stage_musl` patches (`src/internal/syscall.h`,
/// `src/string/memcmp.c`) are never included downstream. The one fork
/// divergence downstream does need, `<sys/timex.h>` (dropped by emscripten's
/// fork, still included by basisu_enc.cpp), is restored into the generated
/// include dir, which is first on the path.
pub fn includes() -> [PathBuf; 4] {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let generated_include = out_dir.join("include");
    [
        generated_include,
        "vendored/musl/include".into(),
        "vendored/musl/src/include".into(),
        // emscripten's musl stdio.h pulls in <wasi/api.h> when __EMSCRIPTEN__
        // is defined (build.rs defines it for the basisu C/zstd sources);
        // the header is vendored at src/wasm_ffi/c/wasi/api.h.
        "src/wasm_ffi/c".into(),
    ]
}

/// Include paths for compiling the musl sources themselves — mirrors musl's
/// own Makefile order (`src/include` before `include`, so `<features.h>`
/// provides `weak_alias` and internal redirects are active), plus our arch
/// shim ahead of the staged `bits/` headers. The musl dirs come from the
/// patched staging copy (`stage_musl`).
fn musl_includes(staged: &Path) -> Vec<PathBuf> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    vec![
        out_dir.join("include"),
        MUSL_ARCH_SHIM.into(),
        // empty fp_arch.h (soft-float default) used by internal/libm.h
        staged.join("arch/generic"),
        staged.join("src/include"),
        staged.join("src/internal"),
        staged.join("include"),
        EMSCRIPTEN_ARCH.into(),
        "src/wasm_ffi/c".into(),
    ]
}

/// Copy emscripten's whole musl tree into OUT_DIR and patch the two files
/// that do not parse in a bare (non-`__EMSCRIPTEN__`) compile, so the
/// vendored submodule stays pristine. The whole tree is copied (not just
/// `src/`) because the `src/include/*` wrappers include the public headers
/// with relative paths (`../../../include/...`) that must keep resolving.
///
/// - `src/internal/syscall.h` — emscripten moved the body of upstream's
///   `__alt_socketcall` into a bare `{...}` block in the `#else` branch
///   (syntax error unless `__EMSCRIPTEN__` is defined).
/// - `src/string/memcmp.c` — the emscripten word loop uses
///   `uint32_t`/`uintptr_t` but only includes `<stdint.h>` under
///   `#if __EMSCRIPTEN__`.
///
/// Everything else in the fork compiles fine without `__EMSCRIPTEN__`
/// (the remaining changes are `__builtin_*`/`__has_feature` tweaks or
/// `__EMSCRIPTEN__`-guarded sections that stay inactive).
fn stage_musl() -> PathBuf {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let staged = out_dir.join("musl");
    copy_tree(Path::new(MUSL_ROOT), &staged);

    let patch = |rel: &str, old: &str, new: &str| {
        let path = staged.join(rel);
        let text = fs::read_to_string(&path).unwrap();
        assert!(
            text.contains(old),
            "expected patch site in {rel} (emscripten musl fork changed?)"
        );
        fs::write(&path, text.replace(old, new)).unwrap();
    };

    // Restore upstream's __alt_socketcall (emscripten's fork left a stray `{`
    // in the #else branch). The socketcall macros are never expanded by the
    // sources this crate compiles, but the header must still parse.
    patch(
        "src/internal/syscall.h",
        r#"#ifdef __EMSCRIPTEN__
#define __socketcall(nm,a,b,c,d,e,f) __syscall(SYS_##nm, a, b, c, d, e, f)
#define __socketcall_cp(nm,a,b,c,d,e,f) __syscall_cp(SYS_##nm, a, b, c, d, e, f)
#else
{
	long r;
	if (cp) r = __syscall_cp(sys, a, b, c, d, e, f);
	else r = __syscall(sys, a, b, c, d, e, f);
	if (r != -ENOSYS) return r;
#ifdef SYS_socketcall
	if (cp) r = __syscall_cp(SYS_socketcall, sock, ((long[6]){a, b, c, d, e, f}));
	else r = __syscall(SYS_socketcall, sock, ((long[6]){a, b, c, d, e, f}));
#endif
	return r;
}
#define __socketcall(nm, a, b, c, d, e, f) __alt_socketcall(SYS_##nm, __SC_##nm, 0, \
	__scc(a), __scc(b), __scc(c), __scc(d), __scc(e), __scc(f))
#define __socketcall_cp(nm, a, b, c, d, e, f) __alt_socketcall(SYS_##nm, __SC_##nm, 1, \
	__scc(a), __scc(b), __scc(c), __scc(d), __scc(e), __scc(f))
#endif"#,
        r#"static inline long __alt_socketcall(int sys, int sock, int cp, syscall_arg_t a, syscall_arg_t b, syscall_arg_t c, syscall_arg_t d, syscall_arg_t e, syscall_arg_t f)
{
	long r;
	if (cp) r = __syscall_cp(sys, a, b, c, d, e, f);
	else r = __syscall(sys, a, b, c, d, e, f);
	if (r != -ENOSYS) return r;
#ifdef SYS_socketcall
	if (cp) r = __syscall_cp(SYS_socketcall, sock, ((long[6]){a, b, c, d, e, f}));
	else r = __syscall(SYS_socketcall, sock, ((long[6]){a, b, c, d, e, f}));
#endif
	return r;
}
#define __socketcall(nm, a, b, c, d, e, f) __alt_socketcall(SYS_##nm, __SC_##nm, 0, \
	__scc(a), __scc(b), __scc(c), __scc(d), __scc(e), __scc(f))
#define __socketcall_cp(nm, a, b, c, d, e, f) __alt_socketcall(SYS_##nm, __SC_##nm, 1, \
	__scc(a), __scc(b), __scc(c), __scc(d), __scc(e), __scc(f))"#,
    );

    // Make the <stdint.h> include unconditional (uint32_t/uintptr_t are used
    // unconditionally by the optimized word loop below it).
    patch(
        "src/string/memcmp.c",
        "#if __EMSCRIPTEN__\n#include <stdint.h>\n#endif\n#include <string.h>",
        "#include <stdint.h>\n#include <string.h>",
    );

    staged
}

/// Subdirectories of `musl/src` compiled wholesale (every .c file).
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

    // Emscripten's fork dropped include/sys/timex.h (basisu_enc.cpp still
    // includes it on the __GNUC__ timing path); restore it from our vendored
    // copy of upstream musl 1.2.6 into the generated include dir, which is
    // first on the downstream include paths (see `includes`).
    fs::create_dir_all(out_dir.join("include").join("sys")).unwrap();
    fs::copy(
        "src/wasm_ffi/c/sys/timex.h",
        out_dir.join("include").join("sys").join("timex.h"),
    )
    .unwrap();

    // Patch a copy of emscripten's musl src/ (see stage_musl_src) and compile
    // from that. The actual libc comes from these musl C sources plus a small
    // set of Rust files (src/wasm_ffi/rust/*.rs: malloc/itoa/atexit/signal)
    // and C shims (errno, nanoprintf, stdio_shim, wasm_libc_shim).
    //
    // The wholesale dirs + explicit files mirror the "compile musl sources
    // directly" approach of sqlite-wasm-rs instead of hand-writing every
    // string/math function in Rust.
    let staged = stage_musl();
    let mut sources: Vec<PathBuf> = vec![];
    for dir in MUSL_WHOLESALE {
        parse_dir(staged.join("src").join(dir), &mut sources, "c");
    }
    for f in MUSL_FILES {
        sources.push(staged.join("src").join(f));
    }

    let mut build = cc::Build::new();
    build
        .includes(musl_includes(&staged))
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
}
