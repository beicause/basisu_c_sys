fn write_cmake_args<A, B, C>(
    default_encoder_emcc_args: &[A],
    default_transcoder_emcc_args: &[A],
    encoder_sources: &[B],
    transcoder_sources: &[B],
    emcc_flags: Option<C>,
    out_dir: &std::path::Path,
) where
    A: AsRef<str>,
    B: AsRef<str>,
    C: AsRef<str>,
{
    let (mut encoder_args, mut transcoder_args) = (
        default_encoder_emcc_args
            .iter()
            .map(|arg| arg.as_ref())
            .filter(|&arg| arg != "-Wno-stringop-overflow")
            .collect::<Vec<_>>(),
        default_transcoder_emcc_args
            .iter()
            .map(|arg| arg.as_ref())
            .filter(|&arg| arg != "-Wno-stringop-overflow")
            .collect::<Vec<_>>(),
    );

    if let Some(flags) = &emcc_flags {
        encoder_args.extend(flags.as_ref().split(" ").filter(|s| !s.is_empty()));
        transcoder_args.extend(flags.as_ref().split(" ").filter(|s| !s.is_empty()));
    }

    for (name, sources) in [
        ("encoder_srcs.txt", encoder_sources),
        ("transcoder_srcs.txt", transcoder_sources),
    ] {
        std::fs::write(
            out_dir.join(name),
            sources
                .iter()
                .map(|src| src.as_ref())
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .unwrap();
    }

    for (name, sources) in [
        ("encoder_link_flags.txt", &encoder_args),
        ("transcoder_link_flags.txt", &transcoder_args),
    ] {
        std::fs::write(
            out_dir.join(name),
            sources
                .iter()
                .map(|src| src.as_ref())
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .unwrap();
    }

    for (name, sources) in [
        ("encoder_compile_flags.txt", encoder_args),
        ("transcoder_compile_flags.txt", transcoder_args),
    ] {
        std::fs::write(
            out_dir.join(name),
            sources
                .into_iter()
                .filter(|f| !f.starts_with("-s") || *f == "-sSTRICT")
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .unwrap();
    }
}
