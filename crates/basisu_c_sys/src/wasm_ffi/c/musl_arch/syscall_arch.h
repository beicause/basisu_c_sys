/* syscall_arch.h — bare-metal wasm32 replacement for emscripten's version.
 *
 * The emscripten arch header forwards to wasi/api.h + emscripten JS
 * syscall glue; bare-metal wasm32-unknown-unknown has no OS and never
 * executes a syscall. The musl sources we compile (string, ctype, math,
 * multibyte, stdlib, the scanf machinery) only pull this header in
 * transitively through stdio_impl.h/pthread_impl.h. Static-inline helpers
 * in those headers still call __syscallN/SYS_futex, so declare the former
 * and define the latter; nothing ever calls them on this target.
 */
#ifndef _SYSCALL_ARCH_H
#define _SYSCALL_ARCH_H

#define __SYSCALL_LL_E(x) (x)
#define __SYSCALL_LL_O(x) (x)

/* musl's syscall.h dispatches __syscall(...) to __syscall1..7 and expects
 * them declared here (real archs define them as inline wrappers). */
long __syscall1(long, long);
long __syscall2(long, long, long);
long __syscall3(long, long, long, long);
long __syscall4(long, long, long, long, long);
long __syscall5(long, long, long, long, long, long);
long __syscall6(long, long, long, long, long, long, long);
long __syscall7(long, long, long, long, long, long, long, long);

/* emscripten's bits/syscall.h omits futex, so syscall.h's fallback
 * `#define SYS_futex SYS_futex_time64` would reference an undefined macro. */
#define SYS_futex 202

#endif
