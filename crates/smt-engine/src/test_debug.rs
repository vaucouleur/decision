#![forbid(unsafe_code)]
//! Cross-crate test toggle for enabling eq-sharing debug output.

#[cfg(feature = "test-debug")]
mod inner {
    use std::sync::atomic::{AtomicBool, Ordering};

    static EQSHARE_DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);

    pub fn set_eqshare_debug_enabled(enabled: bool) {
        EQSHARE_DEBUG_ENABLED.store(enabled, Ordering::SeqCst);
    }

    pub fn eqshare_debug_enabled() -> bool {
        EQSHARE_DEBUG_ENABLED.load(Ordering::SeqCst)
    }
}

#[cfg(feature = "test-debug")]
pub use inner::{eqshare_debug_enabled, set_eqshare_debug_enabled};

#[cfg(not(feature = "test-debug"))]
pub fn eqshare_debug_enabled() -> bool { false }

#[cfg(not(feature = "test-debug"))]
pub fn set_eqshare_debug_enabled(_enabled: bool) {}
