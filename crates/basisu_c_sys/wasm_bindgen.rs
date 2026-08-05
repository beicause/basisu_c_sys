use std::fmt::Write as _;

struct FnDecl {
    attrs: String,        // `#[...]` groups before `fn`, space-joined (e.g. `#[must_use]`), empty if none
    name: String,
    args: String,         // text between `( ... )`, whitespace-normalized to one line; "" if none
    ret: Option<String>,  // text after `->`, whitespace-normalized; None = void fn
}

fn parse_block_fn(body: &str) -> Option<FnDecl> {
    // The `fn` keyword, preceded and followed by whitespace (`pub fn <name>`).
    let fn_idx = body.find(" fn ")?;
    let pre = &body[..fn_idx];

    // Attributes: every `#[...]` group before `fn`. Corpus attrs (`#[must_use]`)
    // contain no nested brackets or strings, so the first `]` closes the group.
    let mut attrs = Vec::new();
    let mut scan = pre;
    while let Some(h) = scan.find('#') {
        if !scan[h..].starts_with("#[") {
            break;
        }
        let open = h + 1;
        let close = scan[open..].find(']')?;
        attrs.push(scan[h..open + 1 + close].to_string());
        scan = &scan[open + 1 + close..];
    }
    let attrs = attrs.join(" ");

    // `<name>`: identifier immediately after `fn `.
    let after_fn = &body[fn_idx + " fn ".len()..];
    let name: String = after_fn
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    if name.is_empty() {
        return None;
    }
    let after_name = &after_fn[name.len()..];
    let paren_open = after_name.find('(')?;

    // `<args>`: balanced-paren scan (corpus has no parens inside types, but the
    // depth counter handles them anyway).
    let args_start = paren_open + 1;
    let mut depth = 1i32;
    let mut close = None;
    for (off, c) in after_name[args_start..].char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(args_start + off);
                    break;
                }
            }
            _ => {}
        }
    }
    let close = close.expect("unterminated parameter list in extern \"C\" block");
    let args_raw = &after_name[args_start..close];

    // `-> <ret> ;` or `;` — everything after the `)`.
    let tail = &after_name[close + 1..];
    let mut ret = None;
    if let Some(arrow) = tail.find("->") {
        let after_arrow = &tail[arrow + 2..];
        let semi = after_arrow
            .find(';')
            .expect("expected `;` after return type in extern \"C\" block");
        let ret_raw = after_arrow[..semi].trim();
        ret = Some(ret_raw.split_whitespace().collect::<Vec<_>>().join(" "));
    }

    let args = args_raw.split_whitespace().collect::<Vec<_>>().join(" ");

    Some(FnDecl { attrs, name, args, ret })
}

fn parse_extern_fns(content: &str) -> Vec<FnDecl> {
    let mut fns = Vec::new();
    let mut rest = content;
    while let Some(start) = rest.find("extern \"C\" {") {
        let body_start = start + "extern \"C\" {".len();
        // The block body contains no strings/comments in this corpus, so a plain
        // brace-depth scan finds the matching `}` exactly.
        let mut depth = 1usize;
        let mut i = body_start;
        while depth > 0 {
            let c = rest[i..]
                .chars()
                .next()
                .expect("unterminated extern \"C\" block");
            match c {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
            i += c.len_utf8();
        }
        let body = &rest[body_start..i - 1];
        // Loud on malformed/non-fn blocks: the corpus has exactly one fn per block.
        fns.push(parse_block_fn(body).expect("expected exactly one `fn` in extern \"C\" block"));
        rest = &rest[i..];
    }
    fns
}

fn arg_names(args: &str) -> Vec<String> {
    // Corpus: every arg is `<ident>: <type>`, no `mut`, no `_`, no generics (verified),
    // so the first identifier of each top-level comma-separated segment is the pattern name.
    let mut names = Vec::new();
    let mut depth = 0i32;
    let mut segment = String::new();
    let end_segment = |names: &mut Vec<String>, segment: &str| {
        let seg = segment.trim();
        let seg = match seg.strip_prefix("mut") {
            Some(rest) if rest.starts_with(|c: char| c.is_whitespace()) => rest.trim_start(),
            _ => seg,
        };
        let ident: String = seg
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if !ident.is_empty() {
            names.push(ident);
        }
    };
    for c in args.chars() {
        match c {
            '(' | '[' | '{' => {
                depth += 1;
                segment.push(c);
            }
            ')' | ']' | '}' => {
                depth -= 1;
                segment.push(c);
            }
            ',' if depth == 0 => {
                end_segment(&mut names, &segment);
                segment.clear();
            }
            _ => segment.push(c),
        }
    }
    end_segment(&mut names, &segment);
    names
}

fn gen_binding_funcs(file_content: &str) -> Vec<String> {
    parse_extern_fns(file_content)
        .into_iter()
        .map(|decl| {
            let mut out = String::new();
            write!(out, "#[wasm_bindgen(method, js_name = \"_{}\")] pub fn {}(", decl.name, decl.name)
                .unwrap();
            write!(out, "this: &Basisu").unwrap();
            if !decl.args.is_empty() {
                write!(out, ", {}", decl.args).unwrap();
            }
            match &decl.ret {
                Some(ret) if ret == "Bool32" => write!(out, ") -> u32;").unwrap(),
                Some(ret) => write!(out, ") -> {ret};").unwrap(),
                None => write!(out, ");").unwrap(),
            }
            out
        })
        .collect()
}

fn gen_public_funcs(file_content: &str) -> Vec<String> {
    parse_extern_fns(file_content)
        .into_iter()
        .map(|decl| {
            let arg_list = arg_names(&decl.args).join(", ");
            let mut out = String::new();
            write!(
                out,
                "{attrs}pub unsafe fn {name}({args})",
                attrs = decl.attrs, name = decl.name, args = decl.args
            )
            .unwrap();
            if let Some(ret) = &decl.ret {
                write!(out, " -> {ret}").unwrap();
            }
            writeln!(out, " {{").unwrap();
            writeln!(out, "    BASISU_INSTANCE.with(|inst| {{").unwrap();
            writeln!(out, "        let inst = inst.get().unwrap();").unwrap();
            if decl.ret.as_deref() == Some("Bool32") {
                writeln!(out, "        Bool32(inst.{}({arg_list}))", decl.name).unwrap();
            } else {
                writeln!(out, "        inst.{}({arg_list})", decl.name).unwrap();
            }
            writeln!(out, "    }})").unwrap();
            write!(out, "}}").unwrap();
            out
        })
        .collect()
}

fn write_binding_file(encoder_apis: &[String], transcoder_apis: &[String]) -> String {
    let mut out = String::new();
    writeln!(out, "#[wasm_bindgen]").unwrap();
    writeln!(out, "extern \"C\" {{").unwrap();
    writeln!(out, "    #[derive(Debug)]").unwrap();
    writeln!(out, "    pub type Basisu;").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "    #[wasm_bindgen(method, getter, js_name = HEAPU8)]").unwrap();
    writeln!(out, "    pub(crate) fn wasm_heap_memory(this: &Basisu) -> Uint8Array;").unwrap();
    writeln!(out).unwrap();
    for api in encoder_apis.iter().chain(transcoder_apis) {
        writeln!(out, "    {api}").unwrap();
    }
    writeln!(out, "}}").unwrap();
    out
}

pub fn generate() {
    let encoder_api_file =
        std::path::PathBuf::from_iter([&std::env::var("OUT_DIR").unwrap(), "basisu_c_api.rs"]);
    let transcoder_api_file = std::path::PathBuf::from_iter([
        &std::env::var("OUT_DIR").unwrap(),
        "basisu_c_transcoder_api.rs",
    ]);
    let encoder_api_file = std::fs::read_to_string(encoder_api_file).unwrap();
    let transcoder_api_file = std::fs::read_to_string(transcoder_api_file).unwrap();

    let encoder_binding_apis = gen_binding_funcs(&encoder_api_file);
    let transcoder_binding_apis = gen_binding_funcs(&transcoder_api_file);

    std::fs::write(
        std::path::PathBuf::from_iter([&std::env::var("OUT_DIR").unwrap(), "wasm_encoder_binding.rs"]),
        write_binding_file(&encoder_binding_apis, &transcoder_binding_apis),
    )
    .unwrap();
    std::fs::write(
        std::path::PathBuf::from_iter([&std::env::var("OUT_DIR").unwrap(), "wasm_transcoder_binding.rs"]),
        write_binding_file(&[], &transcoder_binding_apis),
    )
    .unwrap();

    let encoder_pub_funcs = gen_public_funcs(&encoder_api_file);
    let transcoder_pub_funcs = gen_public_funcs(&transcoder_api_file);

    let mut encoder_pub = String::new();
    for func in &encoder_pub_funcs {
        writeln!(encoder_pub, "{func}").unwrap();
    }
    std::fs::write(
        std::path::PathBuf::from_iter([&std::env::var("OUT_DIR").unwrap(), "wasm_encoder_pub_funcs.rs"]),
        encoder_pub,
    )
    .unwrap();

    let mut transcoder_pub = String::new();
    for func in &transcoder_pub_funcs {
        writeln!(transcoder_pub, "{func}").unwrap();
    }
    std::fs::write(
        std::path::PathBuf::from_iter([&std::env::var("OUT_DIR").unwrap(), "wasm_transcoder_pub_funcs.rs"]),
        transcoder_pub,
    )
    .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mirrors the exact shape of the real bindgen output (basisu_c_api.rs / basisu_c_transcoder_api.rs):
    // one fn per `unsafe extern "C" { }` block, multi-line arg lists with trailing commas,
    // `#[must_use]` on Bool32-returning fns, void fns without `->`, qualified types.
    const SAMPLE: &str = r#"/* automatically generated by rust-bindgen 0.72.1 */

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct Bool32(pub u32);
unsafe extern "C" {
    #[must_use]
    pub fn bu_alloc(size: u64) -> Bool32;
}
unsafe extern "C" {
    pub fn bu_comp_params_set_image_rgba32(
        params_ofs: u64,
        image_index: u32,
        img_data_ofs: u64,
    ) -> u32;
}
unsafe extern "C" {
    pub fn bu_init();
}
unsafe extern "C" {
    pub fn bu_get_version() -> u32;
}
unsafe extern "C" {
    pub fn bu_set_quality_level(quality_level: ::core::ffi::c_int);
}
"#;

    #[test]
    fn parses_all_fns_in_order() {
        let fns = parse_extern_fns(SAMPLE);
        let names: Vec<_> = fns.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "bu_alloc",
                "bu_comp_params_set_image_rgba32",
                "bu_init",
                "bu_get_version",
                "bu_set_quality_level"
            ]
        );
    }

    #[test]
    fn parses_bool32_return_and_must_use() {
        let fns = parse_extern_fns(SAMPLE);
        let alloc = &fns[0];
        assert_eq!(alloc.attrs, "#[must_use]");
        assert_eq!(alloc.args, "size: u64");
        assert_eq!(alloc.ret.as_deref(), Some("Bool32"));
    }

    #[test]
    fn parses_void_fn() {
        let fns = parse_extern_fns(SAMPLE);
        let init = &fns[2];
        assert_eq!(init.args, "");
        assert_eq!(init.ret, None);
    }

    #[test]
    fn normalizes_multiline_args() {
        let fns = parse_extern_fns(SAMPLE);
        assert_eq!(
            fns[1].args,
            "params_ofs: u64, image_index: u32, img_data_ofs: u64,"
        );
    }

    #[test]
    fn preserves_qualified_types() {
        let fns = parse_extern_fns(SAMPLE);
        assert_eq!(fns[4].args, "quality_level: ::core::ffi::c_int");
    }

    #[test]
    fn arg_names_of_multiline_args() {
        let fns = parse_extern_fns(SAMPLE);
        assert_eq!(
            arg_names(&fns[1].args),
            ["params_ofs", "image_index", "img_data_ofs"]
        );
    }

    #[test]
    fn arg_names_empty() {
        assert!(arg_names("").is_empty());
    }

    #[test]
    fn arg_names_strips_mut_guard() {
        assert_eq!(arg_names("mut x: u32, y: u64"), ["x", "y"]);
        // `mut` must only be stripped when followed by whitespace — never from inside an identifier.
        assert_eq!(arg_names("mutable_thing: u32"), ["mutable_thing"]);
    }

    #[test]
    fn binding_rewrites_bool32_to_u32() {
        let out = gen_binding_funcs(SAMPLE);
        assert_eq!(
            out[0],
            "#[wasm_bindgen(method, js_name = \"_bu_alloc\")] pub fn bu_alloc(this: &Basisu, size: u64) -> u32;"
        );
    }

    #[test]
    fn binding_prepends_this_for_void_and_no_args() {
        let out = gen_binding_funcs(SAMPLE);
        assert_eq!(
            out[2],
            "#[wasm_bindgen(method, js_name = \"_bu_init\")] pub fn bu_init(this: &Basisu);"
        );
    }

    #[test]
    fn binding_keeps_other_returns() {
        let out = gen_binding_funcs(SAMPLE);
        assert_eq!(
            out[3],
            "#[wasm_bindgen(method, js_name = \"_bu_get_version\")] pub fn bu_get_version(this: &Basisu) -> u32;"
        );
    }

    #[test]
    fn binding_drops_original_attrs() {
        let out = gen_binding_funcs(SAMPLE);
        assert!(!out[0].contains("must_use"));
    }

    #[test]
    fn wrapper_wraps_bool32_return() {
        let out = gen_public_funcs(SAMPLE);
        assert_eq!(
            out[0],
            "#[must_use]pub unsafe fn bu_alloc(size: u64) -> Bool32 {\n    BASISU_INSTANCE.with(|inst| {\n        let inst = inst.get().unwrap();\n        Bool32(inst.bu_alloc(size))\n    })\n}"
        );
    }

    #[test]
    fn wrapper_keeps_void_ret_and_calls_plain() {
        let out = gen_public_funcs(SAMPLE);
        let init = &out[2];
        assert!(init.starts_with("pub unsafe fn bu_init() {"));
        assert!(init.contains("inst.bu_init()"));
        assert!(!init.contains("->"));
    }

    #[test]
    fn wrapper_passes_arg_names_in_order() {
        let out = gen_public_funcs(SAMPLE);
        assert!(out[1].contains(
            "inst.bu_comp_params_set_image_rgba32(params_ofs, image_index, img_data_ofs)"
        ));
    }

    #[test]
    fn wrapper_keeps_non_bool32_ret_unwrapped() {
        let out = gen_public_funcs(SAMPLE);
        assert!(out[3].starts_with("pub unsafe fn bu_get_version() -> u32 {"));
        assert!(out[3].contains("inst.bu_get_version()"));
    }

    #[test]
    fn binding_file_assembles_header_and_apis() {
        let apis = gen_binding_funcs(SAMPLE);
        let file = write_binding_file(&apis[..1], &apis[1..2]);
        assert!(file.starts_with("#[wasm_bindgen]\nextern \"C\" {\n"));
        assert!(file.contains("pub type Basisu;"));
        assert!(file.contains("wasm_heap_memory"));
        assert!(file.contains("_bu_alloc"));
        assert!(file.contains("_bu_comp_params_set_image_rgba32"));
        assert!(file.ends_with("}\n"));
    }

    #[test]
    fn parses_no_fns_from_comment_only() {
        assert!(parse_extern_fns("/* nothing here */").is_empty());
    }

    #[should_panic(expected = "expected exactly one `fn` in extern \"C\" block")]
    #[test]
    fn non_fn_block_panics() {
        parse_extern_fns("extern \"C\" {\n    pub static FOO: u32;\n}");
    }

    #[should_panic(expected = "unterminated extern \"C\" block")]
    #[test]
    fn unterminated_block_panics() {
        parse_extern_fns("extern \"C\" {\n    pub fn bu_x();\n");
    }
}
