const FLAGS: &[&str] = &[
    "/wd4189",
    "-Wno-unused-variable",
    "-Wno-type-limits",
    "-Wno-unused-but-set-variable",
    "-Wno-unused-function",
    "-Wno-misleading-indentation",
    "-Wno-stringop-overflow",
    "-Wno-array-bounds",
    "-Wno-unused-parameter",
    "-fno-exceptions",
    // Fix gcc optimization issue.
    // See vendor/basis_universal/transcoder/basisu.h
    // See https://github.com/godotengine/godot/pull/114839
    "-fno-strict-aliasing",
];

// Disable PVRTC1/2, ATC, FXT1 as wgpu does not support them.
const DEFINES: &[(&str, &str)] = &[
    // ("BASISU_FORCE_DEVEL_MESSAGES", "1"), // Enable debug message.
    // ("BASISD_SUPPORT_KTX2", "0"),
    // ("BASISD_SUPPORT_KTX2_ZSTD", "0"),
    #[cfg(not(feature = "transcode_uastc"))]
    ("BASISD_SUPPORT_UASTC", "0"),
    #[cfg(not(feature = "transcode_etc1s_bc1"))]
    ("BASISD_SUPPORT_DXT1", "0"), //(BC1)
    #[cfg(not(feature = "transcode_etc1s_bc4_5"))]
    ("BASISD_SUPPORT_DXT5A", "0"), //(BC3 / 4 / 5)
    #[cfg(not(feature = "transcode_etc1s_bc7"))]
    ("BASISD_SUPPORT_BC7", "0"),
    ("BASISD_SUPPORT_PVRTC1", "0"),
    #[cfg(not(feature = "transcode_etc1s_etc2"))]
    ("BASISD_SUPPORT_ETC2_EAC_A8", "0"),
    #[cfg(not(feature = "transcode_astc"))]
    ("BASISD_SUPPORT_ASTC", "0"),
    #[cfg(not(feature = "transcode_xuastc"))]
    ("BASISD_SUPPORT_XUASTC", "0"),
    ("BASISD_SUPPORT_ATC", "0"),
    #[cfg(not(feature = "transcode_etc1s_etc2"))]
    ("BASISD_SUPPORT_ETC2_EAC_RG11", "0"),
    // ("BASISD_SUPPORT_ASTC_HIGHER_OPAQUE_QUALITY", "0"),
    ("BASISD_SUPPORT_FXT1", "0"),
    ("BASISD_SUPPORT_PVRTC2", "0"),
    #[cfg(not(feature = "transcode_uastc_hdr"))]
    ("BASISD_SUPPORT_UASTC_HDR", "0"),
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

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/write_cmake_args.rs"));

fn main() {
    bindgen();

    let target = std::env::var("TARGET").unwrap();

    if target == "wasm32-unknown-unknown" {
        wasm_bindgen();
        let (mut default_encoder_emcc_args, mut default_transcoder_emcc_args) =
            (Vec::<String>::new(), Vec::<String>::new());
        get_wasm_build_args(
            &mut default_encoder_emcc_args,
            &mut default_transcoder_emcc_args,
        );
        let target_feature = std::env::var("CARGO_CFG_TARGET_FEATURE").unwrap();
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let args_dir = std::path::PathBuf::from_iter([&out_dir, "build_args"]);
        let _ = std::fs::create_dir(&args_dir);
        write_cmake_args(
            &default_encoder_emcc_args,
            &default_transcoder_emcc_args,
            ENCODER_SRCS,
            TRANSCODER_SRCS,
            target_feature.contains("simd128").then_some("-msimd128"),
            &args_dir,
        );
        let mut cmake = cmake::Config::new(".");
        cmake
            .profile("")
            .target("wasm32-unknown-emscripten")
            .define("BUILD_ARGS_DIR", &args_dir)
            .build_target("transcoder");
        if std::env::var("PROFILE").unwrap() != "debug" {
            cmake.cflag("-flto=full").cxxflag("-flto=full");
        }
        let opt_flag = "-O".to_string() + &std::env::var("OPT_LEVEL").unwrap();
        cmake.cflag(&opt_flag).cxxflag(&opt_flag);
        #[cfg(feature = "encoder")]
        cmake.build_target("all");
        let dst = cmake.build();
        for name in ["encoder", "transcoder"] {
            #[cfg(not(feature = "encoder"))]
            if name == "encoder" {
                continue;
            }
            let path = dst.join(format!("build/basisu_{name}.js"));
            std::fs::write(
                std::path::PathBuf::from_iter([&out_dir, &format!("wasm_{name}_inline_js.rs")]),
                format!(
                    r##"#[wasm_bindgen(inline_js = r#"{}"#)]
                        extern "C" {{
                            #[wasm_bindgen(js_name = "default")]
                            pub async fn new_instance(args: &Object) -> Basisu;
                            }}"##,
                    std::fs::read_to_string(path).unwrap()
                ),
            )
            .unwrap();
            println!("cargo::rerun-if-changed=CMakeLists.txt");
        }
    }

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
        std::path::PathBuf::from_iter([&std::env::var("OUT_DIR").unwrap(), "basisu_api_common.rs"]);
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
        std::path::PathBuf::from_iter([&std::env::var("OUT_DIR").unwrap(), "basisu_c_api.rs"]);
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

    let binding_file = std::path::PathBuf::from_iter([
        &std::env::var("OUT_DIR").unwrap(),
        "basisu_c_transcoder_api.rs",
    ]);
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
        std::path::PathBuf::from_iter([&std::env::var("OUT_DIR").unwrap(), "basisu_c_api.rs"]);
    let transcoder_api_file = std::path::PathBuf::from_iter([
        &std::env::var("OUT_DIR").unwrap(),
        "basisu_c_transcoder_api.rs",
    ]);
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
        std::path::PathBuf::from_iter([
            &std::env::var("OUT_DIR").unwrap(),
            "wasm_encoder_binding.rs",
        ]),
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
        std::path::PathBuf::from_iter([
            &std::env::var("OUT_DIR").unwrap(),
            "wasm_transcoder_binding.rs",
        ]),
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
        std::path::PathBuf::from_iter([
            &std::env::var("OUT_DIR").unwrap(),
            "wasm_encoder_pub_funcs.rs",
        ]),
        prettyplease::unparse(&syn::parse_quote!(
            #(#encoder_pub_funcs)*
        )),
    )
    .unwrap();

    std::fs::write(
        std::path::PathBuf::from_iter([
            &std::env::var("OUT_DIR").unwrap(),
            "wasm_transcoder_pub_funcs.rs",
        ]),
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
        if target_os == "android" {
            build.flag("-U_GNU_SOURCE");
            build.cpp_link_stdlib("c++_static");
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

fn get_wasm_build_args(
    default_encoder_emcc_args: &mut Vec<String>,
    default_transcoder_emcc_args: &mut Vec<String>,
) {
    let encoder_api_file =
        std::path::PathBuf::from_iter([&std::env::var("OUT_DIR").unwrap(), "basisu_c_api.rs"]);
    let transcoder_api_file = std::path::PathBuf::from_iter([
        &std::env::var("OUT_DIR").unwrap(),
        "basisu_c_transcoder_api.rs",
    ]);
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
    for (default_emcc_args, apis) in [
        (default_encoder_emcc_args, encoder_apis),
        (default_transcoder_emcc_args, transcoder_apis),
    ] {
        let emcc_args = [
            "-sSTRICT".to_string(),
            "-sEXPORT_ES6".to_string(),
            "-sINCOMING_MODULE_JS_API=wasmBinary".to_string(),
            "-sALLOW_MEMORY_GROWTH".to_string(),
            "-sEXPORTED_RUNTIME_METHODS=HEAPU8".to_string(),
            "-sEXPORTED_FUNCTIONS=".to_string() + &apis.join(","),
        ];

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
    }
}
