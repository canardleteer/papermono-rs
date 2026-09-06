//! Board profile runtime detection and read-only model querying.
//!
//! Provides zero-overhead access to the active hardware profile ([`BoardModel`]),
//! dynamically detected at startup.

#[cfg(feature = "c153")]
use core::sync::atomic::{AtomicU8, Ordering};
pub use m5stack_papermono_lite::BoardModel;

#[cfg(feature = "c153")]
static DETECTED_MODEL: AtomicU8 = AtomicU8::new(0);

/// Initializes the detected board model once at startup.
#[cfg(feature = "c153")]
pub fn set_model(model: BoardModel) {
    DETECTED_MODEL.store(model.to_u8(), Ordering::Release);
}

/// Returns the detected board model.
///
/// On unified builds (`c153` feature), this queries the atomic register set during startup.
/// If uninitialized or compiled for Lite only (`not(feature = "c153")`), it returns
/// [`BoardModel::PaperMonoLite`].
#[inline]
pub fn model() -> BoardModel {
    #[cfg(feature = "c153")]
    {
        match BoardModel::from_u8(DETECTED_MODEL.load(Ordering::Acquire)) {
            Some(m) => m,
            None => BoardModel::PaperMonoLite,
        }
    }
    #[cfg(not(feature = "c153"))]
    {
        BoardModel::PaperMonoLite
    }
}
