const FLAGS: &[&str] = &[
    "/wd4189",
    "-Wno-unused-variable",
    "-Wno-type-limits",
    "-Wno-unused-but-set-variable",
    "-Wno-unused-function",
    "-Wno-misleading-indentation",
    "-fno-exceptions",
    // Fix gcc optimization issue.
    // See vendor/basis_universal/transcoder/basisu.h
    // See https://github.com/godotengine/godot/pull/114839
    "-fno-strict-aliasing",
];

// Disable PVRTC1/2, ATC, FXT1 as wgpu does not support them.
const DEFINES: &[(&str, &str)] = &[
    // ("BASISU_FORCE_DEVEL_MESSAGES", "1"), // Enable debug message.
    // ("BASISD_SUPPORT_KTX2", "1"),
    // ("BASISD_SUPPORT_KTX2_ZSTD", "1"),
    // ("BASISD_SUPPORT_UASTC", "1"),
    ("BASISD_SUPPORT_DXT1", "0"), //(BC1)
    // ("BASISD_SUPPORT_DXT5A", "1"), //(BC3 / 4 / 5)
    // ("BASISD_SUPPORT_BC7", "1"),
    // ("BASISD_SUPPORT_BC7_MODE5", "1"),
    ("BASISD_SUPPORT_PVRTC1", "0"),
    // ("BASISD_SUPPORT_ETC2_EAC_A8", "1"),
    // ("BASISD_SUPPORT_ASTC", "1"),
    // ("BASISD_SUPPORT_XUASTC", "1"),
    ("BASISD_SUPPORT_ATC", "0"),
    // ("BASISD_SUPPORT_ETC2_EAC_RG11", "1"),
    // ("BASISD_SUPPORT_ASTC_HIGHER_OPAQUE_QUALITY", "1"),
    ("BASISD_SUPPORT_FXT1", "0"),
    ("BASISD_SUPPORT_PVRTC2", "0"),
    // ("BASISD_SUPPORT_UASTC_HDR", "1"),
];

const ENCODER_SRCS: &[&str] = &[
    "vendor/basis_universal/encoder/basisu_astc_hdr_6x6_enc.cpp",
    "vendor/basis_universal/encoder/basisu_astc_hdr_common.cpp",
    "vendor/basis_universal/encoder/basisu_astc_ldr_common.cpp",
    "vendor/basis_universal/encoder/basisu_astc_ldr_encode.cpp",
    "vendor/basis_universal/encoder/basisu_backend.cpp",
    "vendor/basis_universal/encoder/basisu_basis_file.cpp",
    "vendor/basis_universal/encoder/basisu_bc7enc.cpp",
    "vendor/basis_universal/encoder/basisu_comp.cpp",
    "vendor/basis_universal/encoder/basisu_enc.cpp",
    "vendor/basis_universal/encoder/basisu_etc.cpp",
    "vendor/basis_universal/encoder/basisu_frontend.cpp",
    "vendor/basis_universal/encoder/basisu_gpu_texture.cpp",
    "vendor/basis_universal/encoder/basisu_kernels_sse.cpp",
    "vendor/basis_universal/encoder/basisu_opencl.cpp",
    "vendor/basis_universal/encoder/basisu_pvrtc1_4.cpp",
    "vendor/basis_universal/encoder/basisu_resample_filters.cpp",
    "vendor/basis_universal/encoder/basisu_resampler.cpp",
    "vendor/basis_universal/encoder/basisu_ssim.cpp",
    "vendor/basis_universal/encoder/basisu_uastc_enc.cpp",
    "vendor/basis_universal/encoder/basisu_uastc_hdr_4x4_enc.cpp",
    "vendor/basis_universal/encoder/basisu_wasm_api.cpp",
    "vendor/basis_universal/encoder/basisu_wasm_transcoder_api.cpp",
    "vendor/basis_universal/encoder/jpgd.cpp",
    "vendor/basis_universal/encoder/pvpngreader.cpp",
    "vendor/basis_universal/encoder/3rdparty/android_astc_decomp.cpp",
    "vendor/basis_universal/encoder/3rdparty/tinyexr.cpp",
    "vendor/basis_universal/transcoder/basisu_transcoder.cpp",
    "vendor/basis_universal/zstd/zstd.c",
];

const TRANSCODER_SRCS: &[&str] = &[
    "vendor/basis_universal/encoder/basisu_wasm_transcoder_api.cpp",
    "vendor/basis_universal/transcoder/basisu_transcoder.cpp",
    "vendor/basis_universal/zstd/zstddeclib.c",
];

fn main() {
    bindgen();
    wasm_bindgen();
    #[cfg(feature = "__gen_make_wasm")]
    make_wasm_build_cmd();
    let target = std::env::var("TARGET").unwrap();
    if std::env::var("DOCS_RS").is_err() && target != "wasm32-unknown-unknown" {
        compile_basisu_static();
    }
    println!("cargo::rerun-if-changed=vendor/");
}

#[derive(Debug)]
struct MacroTypeCallbacks;

#[derive(Debug)]
struct WasmBoolRenameCallbacks;

impl bindgen::callbacks::ParseCallbacks for MacroTypeCallbacks {
    fn int_macro(&self, name: &str, _value: i64) -> Option<bindgen::callbacks::IntKind> {
        if name.starts_with("BU_COMP_FLAGS_") {
            Some(bindgen::callbacks::IntKind::U64)
        } else if name.starts_with("BU_EFFORT_") || name.starts_with("BU_QUALITY_") {
            Some(bindgen::callbacks::IntKind::I32)
        } else if name.starts_with("BTF_")
            || name.starts_with("TF_")
            || name.starts_with("DECODE_FLAGS_")
        {
            Some(bindgen::callbacks::IntKind::U32)
        } else {
            None
        }
    }
}

impl bindgen::callbacks::ParseCallbacks for WasmBoolRenameCallbacks {
    fn item_name(&self, item_info: bindgen::callbacks::ItemInfo) -> Option<String> {
        if item_info.name == "wasm_bool_t" {
            Some("Bool32".to_string())
        } else {
            None
        }
    }
}

fn bindgen() {
    let binding_file =
        std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("basisu_api_common.rs");
    bindgen::Builder::default()
        .clang_args(&["-fvisibility=default"])
        .header("vendor/basis_universal/encoder/basisu_wasm_api_common.h")
        .use_core()
        .allowlist_var("^(BU_QUALITY_.*)$")
        .allowlist_var("^(BU_EFFORT_.*)$")
        .allowlist_var("^(BU_COMP_FLAGS_.*)$")
        .allowlist_var("^(BTF_.*)$")
        .allowlist_var("^(TF_.*)$")
        .allowlist_var("^(DECODE_FLAGS_.*)$")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .parse_callbacks(Box::new(MacroTypeCallbacks))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(binding_file)
        .expect("Couldn't write bindings!");

    let binding_file =
        std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("basisu_c_api.rs");
    bindgen::Builder::default()
        .clang_args(&["-fvisibility=default"])
        .header("vendor/basis_universal/encoder/basisu_wasm_api.h")
        .use_core()
        .must_use_type("wasm_bool_t")
        .new_type_alias("Bool32")
        .allowlist_function("^(bu_.*)$")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .parse_callbacks(Box::new(WasmBoolRenameCallbacks))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(binding_file)
        .expect("Couldn't write bindings!");

    let binding_file =
        std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("basisu_c_transcoder_api.rs");
    bindgen::Builder::default()
        .clang_args(&["-fvisibility=default"])
        .header("vendor/basis_universal/encoder/basisu_wasm_transcoder_api.h")
        .use_core()
        .must_use_type("wasm_bool_t")
        .new_type_alias("Bool32")
        .allowlist_function("^(bt_.*)$")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .parse_callbacks(Box::new(WasmBoolRenameCallbacks))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(binding_file)
        .expect("Couldn't write bindings!");
}

fn wasm_bindgen() {
    let encoder_api_file =
        std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("basisu_c_api.rs");
    let transcoder_api_file =
        std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("basisu_c_transcoder_api.rs");
    let encoder_api_file = std::fs::read_to_string(encoder_api_file).unwrap();
    let transcoder_api_file = std::fs::read_to_string(transcoder_api_file).unwrap();

    let encoder_ast = syn::parse_file(&encoder_api_file).unwrap();
    let transcoder_ast = syn::parse_file(&transcoder_api_file).unwrap();

    fn gen_binding_funcs(file_ast: &syn::File) -> Vec<syn::ForeignItem> {
        let ty_bool32: syn::Type = syn::parse_quote!(Bool32);
        let ty_u32: syn::Type = syn::parse_quote!(u32);

        file_ast
            .items
            .iter()
            .filter_map(|item| {
                if let syn::Item::ForeignMod(foreign) = item {
                    assert!(foreign.items.len() == 1);
                    let syn::ForeignItem::Fn(mut func) = foreign.items[0].clone() else {
                        return None;
                    };
                    let func_name = "_".to_string() + &func.sig.ident.to_string();
                    func.attrs = syn::parse_quote!(#[wasm_bindgen(method,js_name=#func_name)]);
                    func.sig.inputs.insert(0, syn::parse_quote!(this: &Basisu));

                    if let syn::ReturnType::Type(_, ty) = &mut func.sig.output
                        && ty.as_ref() == &ty_bool32
                    {
                        *ty = ty_u32.clone().into();
                    }
                    Some(syn::ForeignItem::Fn(func))
                } else {
                    None
                }
            })
            .collect()
    }

    fn gen_public_funcs(file_ast: &syn::File) -> Vec<syn::Item> {
        let ty_bool32: syn::Type = syn::parse_quote!(Bool32);

        file_ast
            .items
            .iter()
            .filter_map(|item| {
                if let syn::Item::ForeignMod(foreign) = item {
                    assert!(foreign.items.len() == 1);
                    let syn::ForeignItem::Fn(func) = foreign.items[0].clone() else {
                        return None;
                    };
                    let func_name = func.sig.ident.clone();
                    let func_inputs = func.sig.inputs.clone();
                    let func_args = func_inputs
                        .iter()
                        .map(|arg| {
                            let syn::FnArg::Typed(pat_type) = arg else {
                                unreachable!()
                            };
                            let syn::Pat::Ident(ident) = &*pat_type.pat else {
                                unreachable!()
                            };
                            ident.ident.clone()
                        })
                        .collect::<Vec<syn::Ident>>();
                    let block: syn::Block = if let syn::ReturnType::Type(_, ty) = &func.sig.output
                        && ty.as_ref() == &ty_bool32
                    {
                        syn::parse_quote! (
                            {
                                BASISU_INSTANCE.with(|inst| {
                                    let inst = inst.get().unwrap();
                                    Bool32(inst.#func_name(#(#func_args),*))
                                })
                            }
                        )
                    } else {
                        syn::parse_quote! (
                            {
                                BASISU_INSTANCE.with(|inst| {
                                    let inst = inst.get().unwrap();
                                    inst.#func_name(#(#func_args),*)
                                })
                            }
                        )
                    };
                    let mut func = syn::ItemFn {
                        attrs: func.attrs,
                        vis: syn::Visibility::Public(Default::default()),
                        sig: func.sig,
                        block: Box::new(block),
                    };
                    func.sig.unsafety = Some(Default::default());
                    Some(syn::Item::Fn(func))
                } else {
                    None
                }
            })
            .collect()
    }

    let encoder_binding_apis = gen_binding_funcs(&encoder_ast);
    let transcoder_binding_apis = gen_binding_funcs(&transcoder_ast);

    std::fs::write(
        std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("wasm_encoder_binding.rs"),
        prettyplease::unparse(&syn::parse_quote!(
            #[wasm_bindgen]
            extern "C" {
                #[derive(Debug)]
                pub type Basisu;

                #[wasm_bindgen(method,getter,js_name=HEAPU8)]
                pub(crate) fn wasm_heap_memory(this: &Basisu) -> Uint8Array;

                #(#encoder_binding_apis)*
                #(#transcoder_binding_apis)*
            }
        )),
    )
    .unwrap();

    std::fs::write(
        std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("wasm_transcoder_binding.rs"),
        prettyplease::unparse(&syn::parse_quote!(
            #[wasm_bindgen]
            extern "C" {
                #[derive(Debug)]
                pub type Basisu;

                #[wasm_bindgen(method,getter,js_name=HEAPU8)]
                pub(crate) fn wasm_heap_memory(this: &Basisu) -> Uint8Array;

                #(#transcoder_binding_apis)*
            }
        )),
    )
    .unwrap();

    let encoder_pub_funcs = gen_public_funcs(&encoder_ast);
    let transcoder_pub_funcs = gen_public_funcs(&transcoder_ast);

    std::fs::write(
        std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("wasm_encoder_pub_funcs.rs"),
        prettyplease::unparse(&syn::parse_quote!(
            #(#encoder_pub_funcs)*
        )),
    )
    .unwrap();

    std::fs::write(
        std::path::Path::new(&std::env::var("OUT_DIR").unwrap())
            .join("wasm_transcoder_pub_funcs.rs"),
        prettyplease::unparse(&syn::parse_quote!(
            #(#transcoder_pub_funcs)*
        )),
    )
    .unwrap();
}

fn compile_basisu_static() {
    for is_cpp in [true, false] {
        let mut build = cc::Build::new();

        // Use c++_static for Android.
        let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
        if is_cpp && target_os == "android" {
            build.cpp_link_stdlib("c++_static").flag("-U_GNU_SOURCE");
        }

        build.cpp(is_cpp).std(if is_cpp { "c++17" } else { "c17" });
        for f in FLAGS {
            build.flag_if_supported(f);
        }
        for (define, value) in DEFINES {
            build.define(define, *value);
        }
        build.files(
            if cfg!(feature = "encoder") {
                ENCODER_SRCS
            } else {
                TRANSCODER_SRCS
            }
            .iter()
            .filter(|src| src.ends_with(if is_cpp { ".cpp" } else { ".c" })),
        );
        build.compile(&format!(
            "basisu_c_sys_{}",
            if is_cpp { "cpp" } else { "c" }
        ));
    }
}

#[cfg(feature = "__gen_make_wasm")]
fn make_wasm_build_cmd() {
    let encoder_api_file =
        std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("basisu_c_api.rs");
    let transcoder_api_file =
        std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("basisu_c_transcoder_api.rs");
    let mut encoder_apis = Vec::new();
    let mut transcoder_apis = Vec::new();

    fn extract_func_name(line: &str) -> Option<&str> {
        let line = line.trim_start();
        if let Some(start) = line.find("pub fn ")
            && let Some(end) = line.rfind("(")
        {
            Some(&line[(start + "pub fn ".len())..end])
        } else {
            None
        }
    }

    for (vec, file) in [
        (&mut encoder_apis, encoder_api_file),
        (&mut transcoder_apis, transcoder_api_file),
    ] {
        for line in std::fs::read_to_string(file).unwrap().lines() {
            if let Some(func) = extract_func_name(line) {
                vec.push("_".to_string() + func);
            }
        }
    }
    encoder_apis.extend(transcoder_apis.iter().cloned());
    for (name, apis, srcs) in [
        ("encoder", encoder_apis, ENCODER_SRCS),
        ("transcoder", transcoder_apis, TRANSCODER_SRCS),
    ] {
        let emcc_args = [
            "-sSTRICT".to_string(),
            "-sEXPORT_ES6".to_string(),
            "-sINCOMING_MODULE_JS_API=wasmBinary".to_string(),
            "-sALLOW_MEMORY_GROWTH".to_string(),
            "-sEXPORTED_RUNTIME_METHODS=HEAPU8".to_string(),
            "-sEXPORTED_FUNCTIONS=".to_string() + &apis.join(","),
        ];
        let mut default_emcc_args = Vec::new();
        default_emcc_args.extend(
            FLAGS
                .iter()
                .filter(|f| !f.starts_with("/wd"))
                .map(ToString::to_string),
        );
        default_emcc_args.extend(
            DEFINES
                .iter()
                .map(|(define, value)| format!("-D{define}={value}")),
        );
        default_emcc_args.extend(emcc_args);

        std::fs::write(
            std::path::Path::new(&std::env::var("OUT_DIR").unwrap())
                .join(format!("build_{name}_emcc_args.rs")),
            format!(
                "const DEFAULT_{}_EMCC_ARGS: &[&str] = &{:?};",
                name.to_uppercase(),
                default_emcc_args
            ),
        )
        .unwrap();

        std::fs::write(
            std::path::Path::new(&std::env::var("OUT_DIR").unwrap())
                .join(format!("build_{name}_sources.rs")),
            format!(
                "const {}_SOURCES: &[&str] = &{:?};",
                name.to_uppercase(),
                srcs
            ),
        )
        .unwrap();
    }
}
