// stdio_shim.c — variadic printf-family shims for wasm32-unknown-unknown.
//
// These must be C: stable Rust cannot *define* C-ABI variadic functions.
// Each wrapper formats through nanoprintf (vsnprintf, provided by
// nanoprintf.c in the same archive) into a scratch buffer and discards the
// result — there is no stdout on this target, but the return value and ABI
// stay correct.
//
// The non-variadic output symbols (fputc, fwrite, stdout, stderr, ...) are
// in wasm_libc_shim.c; numeric conversions (atof, lrintf, ...) come from
// the vendored musl sources compiled by wasm_libc.rs.

#include <stdarg.h>
#include <stddef.h>
#include <stdio.h>

static int discard_printf(const char *format, va_list args)
{
    char buf[512];
    return vsnprintf(buf, sizeof buf, format, args);
}

int printf(const char *format, ...)
{
    va_list args;
    va_start(args, format);
    int const ret = discard_printf(format, args);
    va_end(args);
    return ret;
}

int vprintf(const char *format, va_list args)
{
    return discard_printf(format, args);
}

int vfprintf(FILE *stream, const char *format, va_list args)
{
    (void)stream;
    return discard_printf(format, args);
}

int fprintf(FILE *stream, const char *format, ...)
{
    (void)stream;
    va_list args;
    va_start(args, format);
    int const ret = discard_printf(format, args);
    va_end(args);
    return ret;
}
