/* wasm_libc_shim.c — libc pieces that musl cannot provide on bare-metal
 * wasm32-unknown-unknown (no OS, no threads, no filesystem, no clock).
 *
 * Split into:
 *   1. pthread stubs        — single-threaded no-ops (musl pthread needs
 *                             futex syscalls).
 *   2. time / sysconf       — no clock: zero times; sane sysconf values.
 *   3. stdio file stubs     — no filesystem: fopen always fails; output
 *                             functions silently succeed. The printf family
 *                             itself comes from nanoprintf (stdio_shim.c).
 *   4. setjmp/longjmp       — no wasm EH support; longjmp traps.
 *   5. locale               — everything is the C.UTF-8 locale; no locale
 *                             files are ever loaded (musl would mmap them).
 *   6. strftime_l           — minimal English-only implementation.
 *   7. __get_tp glue        — the musl multibyte/locale sources we compile
 *                             ask for the "current thread" via __get_tp();
 *                             return a static single-threaded struct.
 */

#include <stddef.h>
#include <stdint.h>
#include <stdarg.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <setjmp.h>
#include <time.h>
#include <unistd.h>
#include <sys/time.h>
#include <locale.h>
#include <pthread.h>

/* struct __locale_struct / struct __locale_map / struct __pthread */
#include "locale_impl.h"
#include "pthread_impl.h"

/* defined in section 7 (single-threaded pthread glue) */
static struct __pthread wasm_main_thread;

/* ────────────────────────── 1. pthread stubs ────────────────────────── */

#define WASM_PTHREAD_KEYS_MAX 64
#define WASM_PTHREAD_ONCE_DONE 1
#define WASM_PTHREAD_EAGAIN 11
#define WASM_PTHREAD_ETIMEDOUT 110

static void *wasm_pthread_keys[WASM_PTHREAD_KEYS_MAX];

int pthread_mutex_init(pthread_mutex_t *restrict mutex,
                       const pthread_mutexattr_t *restrict attr)
{
    (void)mutex;
    (void)attr;
    return 0;
}

int pthread_mutex_destroy(pthread_mutex_t *mutex)
{
    (void)mutex;
    return 0;
}

int pthread_mutex_lock(pthread_mutex_t *mutex)
{
    (void)mutex;
    return 0;
}

int pthread_mutex_unlock(pthread_mutex_t *mutex)
{
    (void)mutex;
    return 0;
}

int pthread_mutex_trylock(pthread_mutex_t *mutex)
{
    (void)mutex;
    return 0;
}

int pthread_cond_init(pthread_cond_t *restrict cond,
                      const pthread_condattr_t *restrict attr)
{
    (void)cond;
    (void)attr;
    return 0;
}

int pthread_cond_destroy(pthread_cond_t *cond)
{
    (void)cond;
    return 0;
}

int pthread_cond_signal(pthread_cond_t *cond)
{
    (void)cond;
    return 0;
}

int pthread_cond_broadcast(pthread_cond_t *cond)
{
    (void)cond;
    return 0;
}

int pthread_cond_wait(pthread_cond_t *restrict cond,
                      pthread_mutex_t *restrict mutex)
{
    /* Cannot block without threads; report a spurious wakeup. */
    (void)cond;
    (void)mutex;
    return 0;
}

int pthread_cond_timedwait(pthread_cond_t *restrict cond,
                           pthread_mutex_t *restrict mutex,
                           const struct timespec *restrict abstime)
{
    /* No signaler can ever arrive on single-threaded wasm: always report
     * the timeout so condition_variable::wait_for/until terminate. */
    (void)cond;
    (void)mutex;
    (void)abstime;
    return WASM_PTHREAD_ETIMEDOUT;
}

int pthread_rwlock_init(pthread_rwlock_t *restrict rwlock,
                        const pthread_rwlockattr_t *restrict attr)
{
    (void)rwlock;
    (void)attr;
    return 0;
}

int pthread_rwlock_destroy(pthread_rwlock_t *rwlock)
{
    (void)rwlock;
    return 0;
}

int pthread_rwlock_rdlock(pthread_rwlock_t *rwlock)
{
    (void)rwlock;
    return 0;
}

int pthread_rwlock_wrlock(pthread_rwlock_t *rwlock)
{
    (void)rwlock;
    return 0;
}

int pthread_rwlock_unlock(pthread_rwlock_t *rwlock)
{
    (void)rwlock;
    return 0;
}

int pthread_rwlock_tryrdlock(pthread_rwlock_t *rwlock)
{
    (void)rwlock;
    return 0;
}

int pthread_rwlock_trywrlock(pthread_rwlock_t *rwlock)
{
    (void)rwlock;
    return 0;
}

int pthread_once(pthread_once_t *once_control, void (*init)(void))
{
    /* Single-threaded: no race on the once control word. */
    if (__c11_atomic_load((_Atomic int *)once_control, __ATOMIC_ACQUIRE) !=
        WASM_PTHREAD_ONCE_DONE) {
        init();
        __c11_atomic_store((_Atomic int *)once_control, WASM_PTHREAD_ONCE_DONE,
                           __ATOMIC_RELEASE);
    }
    return 0;
}

pthread_t pthread_self(void)
{
    return (pthread_t)&wasm_main_thread;
}

/* musl defines pthread_equal as a macro in <pthread.h> — nothing to link. */

int pthread_create(pthread_t *restrict thread,
                   const pthread_attr_t *restrict attr,
                   void *(*start)(void *), void *restrict arg)
{
    /* No threads on this target — refuse so callers don't assume a thread
     * is running. */
    (void)thread;
    (void)attr;
    (void)start;
    (void)arg;
    return WASM_PTHREAD_EAGAIN;
}

int pthread_join(pthread_t thread, void **retval)
{
    (void)thread;
    (void)retval;
    return 0;
}

int pthread_detach(pthread_t thread)
{
    (void)thread;
    return 0;
}

int pthread_key_create(pthread_key_t *key, void (*dtor)(void *))
{
    static int next_key;
    (void)dtor;
    if (next_key >= WASM_PTHREAD_KEYS_MAX)
        return WASM_PTHREAD_EAGAIN;
    *key = next_key++;
    return 0;
}

int pthread_key_delete(pthread_key_t key)
{
    if (key < 0 || key >= WASM_PTHREAD_KEYS_MAX)
        return EINVAL;
    wasm_pthread_keys[key] = NULL;
    return 0;
}

void *pthread_getspecific(pthread_key_t key)
{
    if (key < 0 || key >= WASM_PTHREAD_KEYS_MAX)
        return NULL;
    return wasm_pthread_keys[key];
}

int pthread_setspecific(pthread_key_t key, const void *value)
{
    if (key < 0 || key >= WASM_PTHREAD_KEYS_MAX)
        return EINVAL;
    wasm_pthread_keys[key] = (void *)value;
    return 0;
}

/* ────────────────────────── 2. time / sysconf ───────────────────────── */

/* No clock on bare-metal wasm32: report the epoch (1970-01-01T00:00:00Z).
 * Differencing two calls yields zero, which is the closest we can get to a
 * monotonic clock without host imports. */
int clock_gettime(int clockid, struct timespec *ts)
{
    (void)clockid;
    if (ts) {
        ts->tv_sec = 0;
        ts->tv_nsec = 0;
    }
    return 0;
}

int gettimeofday(struct timeval *tv, void *tz)
{
    (void)tz;
    if (tv) {
        tv->tv_sec = 0;
        tv->tv_usec = 0;
    }
    return 0;
}

long sysconf(int name)
{
    switch (name) {
    case _SC_ARG_MAX:
        return 131072;
    case _SC_CLK_TCK:
        return 100;
    case _SC_OPEN_MAX:
        return 256;
    case _SC_PAGESIZE:
        return 65536;
    case _SC_NPROCESSORS_CONF:
    case _SC_NPROCESSORS_ONLN:
        return 1;
    case _SC_MONOTONIC_CLOCK:
        return 1;
    case _SC_ATEXIT_MAX:
        return 32;
    default:
        return -1;
    }
}

/* ──────────────────────── 3. stdio file stubs ──────────────────────── */

/* No filesystem: fopen always fails, every other file operation reports
 * failure. The output entry points silently succeed (nothing depends on
 * their return value) and never dereference the stream, so NULL is fine.
 *
 * With musl's src/include first in the include path, <stdio.h> turns
 * stdin/stdout/stderr into macros over __stdin_FILE etc.; the C++ and
 * basisu objects were compiled with the public header (extern FILE *), so
 * define the plain data symbols here. */
#undef stdin
#undef stdout
#undef stderr
/* musl's <stdio.h> declares these `FILE *const` — match the declared type. */
FILE *const stdin = NULL;
FILE *const stdout = NULL;
FILE *const stderr = NULL;

FILE *fopen(const char *path, const char *mode)
{
    (void)path;
    (void)mode;
    return NULL;
}

int fclose(FILE *stream)
{
    (void)stream;
    return EOF;
}

size_t fread(void *ptr, size_t size, size_t nmemb, FILE *stream)
{
    (void)ptr;
    (void)size;
    (void)nmemb;
    (void)stream;
    return 0;
}

int fseek(FILE *stream, long offset, int whence)
{
    (void)stream;
    (void)offset;
    (void)whence;
    return -1;
}

long ftell(FILE *stream)
{
    (void)stream;
    return -1L;
}

off_t ftello(FILE *stream)
{
    (void)stream;
    return (off_t)-1;
}

int ferror(FILE *stream)
{
    (void)stream;
    return 0;
}

int fflush(FILE *stream)
{
    (void)stream;
    return 0;
}

int fputs(const char *s, FILE *stream)
{
    (void)s;
    (void)stream;
    return EOF;
}

int puts(const char *s)
{
    (void)s;
    return EOF;
}

int putchar(int c)
{
    (void)c;
    return EOF;
}

int fputc(int c, FILE *stream)
{
    (void)c;
    (void)stream;
    return 0;
}

size_t fwrite(const void *ptr, size_t size, size_t nmemb, FILE *stream)
{
    (void)ptr;
    (void)size;
    (void)nmemb;
    (void)stream;
    return 0;
}

/* musl stdio locking — single-threaded: never contended. FILE* streams are
 * always NULL (fopen fails), so never dereference. */
int __lockfile(FILE *f)
{
    (void)f;
    return 0;
}

void __unlockfile(FILE *f)
{
    (void)f;
}

void __lock(int *lock)
{
    (void)lock;
}

void __unlock(int *lock)
{
    (void)lock;
}

int vasprintf(char **s, const char *fmt, va_list ap)
{
    va_list ap2;
    va_copy(ap2, ap);
    int n = vsnprintf(NULL, 0, fmt, ap2);
    va_end(ap2);
    if (n < 0)
        return -1;
    char *buf = (char *)malloc((size_t)n + 1);
    if (!buf)
        return -1;
    *s = buf;
    return vsnprintf(buf, (size_t)n + 1, fmt, ap);
}

/* ──────────────────────── 4. setjmp / longjmp ──────────────────────── */

/* Bare-metal wasm32 has no setjmp lowering (no EH proposal, no emscripten
 * JS support): setjmp never takes the jump path, longjmp traps. Only used
 * for error handling in jpgd — a malformed JPEG aborts instead of being
 * reported as an error. */
int setjmp(jmp_buf env)
{
    (void)env;
    return 0;
}

void longjmp(jmp_buf env, int val)
{
    (void)env;
    (void)val;
    __builtin_trap();
}

/* ─────────────────────────── 5. locale ─────────────────────────────── */

/* Everything is the C.UTF-8 locale. The shared handle below is only ever
 * compared/passed around — no compiled code dereferences it. */
static struct __locale_struct wasm_c_locale;

locale_t newlocale(int mask, const char *name, locale_t base)
{
    (void)mask;
    (void)name;
    (void)base;
    return (locale_t)&wasm_c_locale;
}

locale_t duplocale(locale_t loc)
{
    (void)loc;
    return (locale_t)&wasm_c_locale;
}

static locale_t wasm_current_locale;

locale_t uselocale(locale_t new_loc)
{
    locale_t old = wasm_current_locale;
    if (new_loc)
        wasm_current_locale =
            (new_loc == LC_GLOBAL_LOCALE) ? NULL : new_loc;
    return old ? old : LC_GLOBAL_LOCALE;
}

void freelocale(locale_t loc)
{
    (void)loc;
}

/* ─────────────────────────── 6. strftime_l ─────────────────────────── */

static const char *const wasm_months[12] = {
    "January", "February", "March",     "April",   "May",      "June",
    "July",    "August",   "September", "October", "November", "December",
};

static const char *const wasm_months_abbr[12] = {
    "Jan", "Feb", "Mar", "Apr", "May", "Jun",
    "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
};

static const char *const wasm_days[7] = {
    "Sunday", "Monday", "Tuesday", "Wednesday",
    "Thursday", "Friday", "Saturday",
};

static const char *const wasm_days_abbr[7] = {
    "Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat",
};

static int wasm_append(char *buf, size_t max, size_t *len, const char *s)
{
    size_t n = strlen(s);
    if (*len + n + 1 > max)
        return -1;
    memcpy(buf + *len, s, n + 1);
    *len += n;
    return 0;
}

static int wasm_append_num(char *buf, size_t max, size_t *len, long v,
                           int width)
{
    char tmp[32];
    char fmt[16];
    snprintf(fmt, sizeof fmt, "%%0%dl", width);
    snprintf(tmp, sizeof tmp, fmt, v);
    return wasm_append(buf, max, len, tmp);
}

size_t strftime_l(char *restrict s, size_t max, const char *restrict fmt,
                  const struct tm *restrict tm, locale_t loc)
{
    (void)loc;
    if (!s || !max)
        return 0;
    s[0] = '\0';
    size_t len = 0;

    for (const char *p = fmt; *p; p++) {
        if (*p != '%') {
            if (len + 2 > max)
                return 0;
            s[len++] = *p;
            s[len] = '\0';
            continue;
        }
        char c = *++p;
        const char *out = NULL;
        char one[2] = {0, 0};
        switch (c) {
        case '%':
            one[0] = '%';
            out = one;
            break;
        case 'a':
            out = wasm_days_abbr[tm->tm_wday];
            break;
        case 'A':
            out = wasm_days[tm->tm_wday];
            break;
        case 'b':
        case 'h':
            out = wasm_months_abbr[tm->tm_mon];
            break;
        case 'B':
            out = wasm_months[tm->tm_mon];
            break;
        case 'C':
            if (wasm_append_num(s, max, &len, 100 + tm->tm_year / 100, 2) < 0)
                return 0;
            continue;
        case 'd':
            if (wasm_append_num(s, max, &len, tm->tm_mday, 2) < 0)
                return 0;
            continue;
        case 'D':
            if (wasm_append_num(s, max, &len, tm->tm_mon + 1, 2) < 0 ||
                wasm_append(s, max, &len, "/") < 0 ||
                wasm_append_num(s, max, &len, tm->tm_mday, 2) < 0 ||
                wasm_append(s, max, &len, "/") < 0 ||
                wasm_append_num(s, max, &len, tm->tm_year % 100, 2) < 0)
                return 0;
            continue;
        case 'e':
            if (wasm_append_num(s, max, &len, tm->tm_mday, 2) < 0)
                return 0;
            continue;
        case 'F':
            if (wasm_append_num(s, max, &len, tm->tm_year + 1900, 4) < 0 ||
                wasm_append(s, max, &len, "-") < 0 ||
                wasm_append_num(s, max, &len, tm->tm_mon + 1, 2) < 0 ||
                wasm_append(s, max, &len, "-") < 0 ||
                wasm_append_num(s, max, &len, tm->tm_mday, 2) < 0)
                return 0;
            continue;
        case 'H':
            if (wasm_append_num(s, max, &len, tm->tm_hour, 2) < 0)
                return 0;
            continue;
        case 'I': {
            int h12 = tm->tm_hour % 12;
            if (h12 == 0)
                h12 = 12;
            if (wasm_append_num(s, max, &len, h12, 2) < 0)
                return 0;
            continue;
        }
        case 'j':
            if (wasm_append_num(s, max, &len, tm->tm_yday + 1, 3) < 0)
                return 0;
            continue;
        case 'm':
            if (wasm_append_num(s, max, &len, tm->tm_mon + 1, 2) < 0)
                return 0;
            continue;
        case 'M':
            if (wasm_append_num(s, max, &len, tm->tm_min, 2) < 0)
                return 0;
            continue;
        case 'n':
            one[0] = '\n';
            out = one;
            break;
        case 'p':
            out = tm->tm_hour < 12 ? "AM" : "PM";
            break;
        case 'r': {
            int h12 = tm->tm_hour % 12;
            if (h12 == 0)
                h12 = 12;
            if (wasm_append_num(s, max, &len, h12, 2) < 0 ||
                wasm_append(s, max, &len, ":") < 0 ||
                wasm_append_num(s, max, &len, tm->tm_min, 2) < 0 ||
                wasm_append(s, max, &len, ":") < 0 ||
                wasm_append_num(s, max, &len, tm->tm_sec, 2) < 0 ||
                wasm_append(s, max, &len,
                            tm->tm_hour < 12 ? "AM" : "PM") < 0)
                return 0;
            continue;
        }
        case 'R':
            if (wasm_append_num(s, max, &len, tm->tm_hour, 2) < 0 ||
                wasm_append(s, max, &len, ":") < 0 ||
                wasm_append_num(s, max, &len, tm->tm_min, 2) < 0)
                return 0;
            continue;
        case 'S':
            if (wasm_append_num(s, max, &len, tm->tm_sec, 2) < 0)
                return 0;
            continue;
        case 't':
            one[0] = '\t';
            out = one;
            break;
        case 'T':
            if (wasm_append_num(s, max, &len, tm->tm_hour, 2) < 0 ||
                wasm_append(s, max, &len, ":") < 0 ||
                wasm_append_num(s, max, &len, tm->tm_min, 2) < 0 ||
                wasm_append(s, max, &len, ":") < 0 ||
                wasm_append_num(s, max, &len, tm->tm_sec, 2) < 0)
                return 0;
            continue;
        case 'u': {
            int w = tm->tm_wday;
            if (w == 0)
                w = 7;
            if (wasm_append_num(s, max, &len, w, 1) < 0)
                return 0;
            continue;
        }
        case 'w':
            if (wasm_append_num(s, max, &len, tm->tm_wday, 1) < 0)
                return 0;
            continue;
        case 'x':
            /* locale-specific: %m/%d/%y */
            if (wasm_append_num(s, max, &len, tm->tm_mon + 1, 2) < 0 ||
                wasm_append(s, max, &len, "/") < 0 ||
                wasm_append_num(s, max, &len, tm->tm_mday, 2) < 0 ||
                wasm_append(s, max, &len, "/") < 0 ||
                wasm_append_num(s, max, &len, tm->tm_year % 100, 2) < 0)
                return 0;
            continue;
        case 'X':
            /* locale-specific: %H:%M:%S */
            if (wasm_append_num(s, max, &len, tm->tm_hour, 2) < 0 ||
                wasm_append(s, max, &len, ":") < 0 ||
                wasm_append_num(s, max, &len, tm->tm_min, 2) < 0 ||
                wasm_append(s, max, &len, ":") < 0 ||
                wasm_append_num(s, max, &len, tm->tm_sec, 2) < 0)
                return 0;
            continue;
        case 'c':
            /* locale-specific: %a %b %e %H:%M:%S %Y */
            if (wasm_append(s, max, &len, wasm_days_abbr[tm->tm_wday]) < 0 ||
                wasm_append(s, max, &len, " ") < 0 ||
                wasm_append(s, max, &len, wasm_months_abbr[tm->tm_mon]) < 0 ||
                wasm_append(s, max, &len, " ") < 0 ||
                wasm_append_num(s, max, &len, tm->tm_mday, 2) < 0 ||
                wasm_append(s, max, &len, " ") < 0 ||
                wasm_append_num(s, max, &len, tm->tm_hour, 2) < 0 ||
                wasm_append(s, max, &len, ":") < 0 ||
                wasm_append_num(s, max, &len, tm->tm_min, 2) < 0 ||
                wasm_append(s, max, &len, ":") < 0 ||
                wasm_append_num(s, max, &len, tm->tm_sec, 2) < 0 ||
                wasm_append(s, max, &len, " ") < 0 ||
                wasm_append_num(s, max, &len, tm->tm_year + 1900, 4) < 0)
                return 0;
            continue;
        case 'y':
            if (wasm_append_num(s, max, &len, tm->tm_year % 100, 2) < 0)
                return 0;
            continue;
        case 'Y':
            if (wasm_append_num(s, max, &len, tm->tm_year + 1900, 4) < 0)
                return 0;
            continue;
        case 'z':
            out = "+0000";
            break;
        case 'Z':
            out = "";
            break;
        case '\0':
            /* trailing % — emit literally and stop */
            if (len + 2 > max)
                return 0;
            s[len++] = '%';
            s[len] = '\0';
            return len;
        default:
            /* Unknown specifier: emit literally (glibc behavior). */
            if (len + 3 > max)
                return 0;
            s[len++] = '%';
            s[len++] = c;
            s[len] = '\0';
            continue;
        }
        if (out && wasm_append(s, max, &len, out) < 0)
            return 0;
    }
    return len;
}

size_t strftime(char *restrict s, size_t max, const char *restrict fmt,
                const struct tm *restrict tm)
{
    return strftime_l(s, max, fmt, tm, NULL);
}

/* ───────────────────── 7. single-threaded pthread glue ─────────────── */

/* The musl multibyte/locale sources compiled into the libc archive query
 * the current thread's locale through __pthread_self() → __get_tp().
 * wasm32 has no thread pointer: hand out a static, always-UTF-8 thread. */
static struct __pthread wasm_main_thread;
static struct __locale_map wasm_utf8_map;
static struct __locale_struct wasm_utf8_locale = {
    .cat = {&wasm_utf8_map, 0, 0, 0, 0, 0},
};

uintptr_t __get_tp(void)
{
    wasm_main_thread.self = &wasm_main_thread;
    wasm_main_thread.locale = &wasm_utf8_locale;
    return (uintptr_t)&wasm_main_thread;
}

/* ───────────────── 8. musl-internal allocator aliases ──────────────── */

/* musl's malloc.c is not compiled (it needs brk/mmap syscalls); the Rust
 * dlmalloc provides malloc/calloc. musl's atexit.c calls the internal
 * __libc_malloc/__libc_calloc aliases — forward them to the public ones. */
void *__libc_malloc(size_t n)
{
    return malloc(n);
}

void *__libc_calloc(size_t n, size_t s)
{
    return calloc(n, s);
}
