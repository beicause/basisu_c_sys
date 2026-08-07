/* pthread_arch.h — bare-metal wasm32 replacement for emscripten's version.
 *
 * wasm32 has no thread-pointer register. The musl multibyte/locale code we
 * compile asks for the "current thread" via __get_tp(); we return a pointer
 * to a static single-threaded pthread struct (see wasm_libc_shim.c).
 */
#ifndef _PTHREAD_ARCH_H
#define _PTHREAD_ARCH_H

#include <stdint.h>

uintptr_t __get_tp(void);

#define TP_ADJ(p) (p)

#define CANCEL_REG_IP 16

#endif
