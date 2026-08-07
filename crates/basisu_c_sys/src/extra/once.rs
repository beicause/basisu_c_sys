//! A tiny `Once` for one-time C library initialization, `core`-only.
//!
//! `std::sync::OnceLock` is not available in `no_std`, so this replaces it
//! with a plain atomic state machine that has the same blocking semantics:
//! the first caller runs the closure, every other caller blocks (spins)
//! until it finishes.

use core::sync::atomic::{AtomicU8, Ordering};

const UNINITIALIZED: u8 = 0;
const INITIALIZING: u8 = 1;
const INITIALIZED: u8 = 2;

/// A blocking one-time initialization primitive with `OnceLock`-like
/// semantics.
pub(crate) struct Once(AtomicU8);

impl Once {
    /// Creates a new `Once` in the uninitialized state.
    pub(crate) const fn new() -> Self {
        Self(AtomicU8::new(UNINITIALIZED))
    }

    /// Runs `f` exactly once. Concurrent callers block until it completes.
    ///
    /// Note: if `f` panics, the state is left as `INITIALIZING` and later
    /// callers spin forever — same spirit as `OnceLock` panicking on a
    /// poisoned init. The closures used here are FFI calls that don't panic.
    pub(crate) fn call_once(&self, f: impl FnOnce()) {
        if self.0.load(Ordering::Acquire) == INITIALIZED {
            return;
        }
        if self
            .0
            .compare_exchange(
                UNINITIALIZED,
                INITIALIZING,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
        {
            f();
            self.0.store(INITIALIZED, Ordering::Release);
        } else {
            while self.0.load(Ordering::Acquire) != INITIALIZED {
                core::hint::spin_loop();
            }
        }
    }

    /// Returns `true` if the closure has already run to completion.
    pub(crate) fn is_initialized(&self) -> bool {
        self.0.load(Ordering::Acquire) == INITIALIZED
    }
}

#[cfg(test)]
mod tests {
    use super::Once;
    use alloc::vec::Vec;
    use core::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn new_starts_uninitialized() {
        let once = Once::new();
        assert!(!once.is_initialized());
    }

    #[test]
    fn call_once_runs_closure_and_initializes() {
        let once = Once::new();
        let calls = AtomicUsize::new(0);
        once.call_once(|| {
            calls.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(once.is_initialized());
    }

    #[test]
    fn closure_side_effects_visible_to_later_callers() {
        let once = Once::new();
        let value = AtomicUsize::new(0);
        once.call_once(|| {
            value.store(42, Ordering::SeqCst);
        });
        assert!(once.is_initialized());
        assert_eq!(value.load(Ordering::SeqCst), 42);
    }

    #[test]
    fn late_callers_do_not_rerun_closure() {
        let once = Once::new();
        let calls = AtomicUsize::new(0);
        for _ in 0..100 {
            once.call_once(|| {
                calls.fetch_add(1, Ordering::SeqCst);
            });
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(once.is_initialized());
    }

    #[test]
    fn concurrent_callers_run_closure_exactly_once() {
        const THREADS: usize = 32;
        let once = Arc::new(Once::new());
        let calls = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(Barrier::new(THREADS));

        let mut handles = Vec::with_capacity(THREADS);
        for _ in 0..THREADS {
            let once = Arc::clone(&once);
            let calls = Arc::clone(&calls);
            let barrier = Arc::clone(&barrier);
            handles.push(thread::spawn(move || {
                // Start all threads as simultaneously as possible so several
                // race to win the compare_exchange and the rest must spin.
                barrier.wait();
                once.call_once(|| {
                    // Keep the initializer busy so spinning threads have
                    // something to actually block on.
                    thread::sleep(Duration::from_millis(50));
                    calls.fetch_add(1, Ordering::SeqCst);
                });
                // Once call_once returns, the closure must have completed.
                assert!(once.is_initialized());
            }));
        }
        for handle in handles {
            handle.join().expect("thread panicked");
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(once.is_initialized());
    }

    #[test]
    fn separate_instances_are_independent() {
        let a = Once::new();
        let b = Once::new();
        a.call_once(|| {});
        assert!(a.is_initialized());
        assert!(!b.is_initialized());
        b.call_once(|| {});
        assert!(b.is_initialized());
    }

    #[test]
    fn can_be_used_in_static_context() {
        static INIT: Once = Once::new();
        assert!(!INIT.is_initialized());
        INIT.call_once(|| {});
        assert!(INIT.is_initialized());
        // A second call on the same static is a no-op.
        INIT.call_once(|| panic!("closure must not run again"));
    }
}
