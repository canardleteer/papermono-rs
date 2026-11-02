//! MCU-independent BLE crypto registration and ChaCha12 random driver.
//!
//! Follows TrouBLE's ESP passkey example at revision `866ee711`. The hardware
//! entry point supplies 32 bytes from esp-hal's enabled TRNG before BLE starts;
//! this module owns no GPIO, ADC, I2C or wireless peripheral. Only the AES-128
//! ECB, AES-128 CMAC and P-256 ECDH software drivers are enabled in the manifest.
//! The same source is exercised by the host crypto integration test.

use core::cell::RefCell;

// Importing the backend retains its global driver registrations at final link.
use embassy_crypto_rustcrypto as _;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use rand_chacha::rand_core::{Rng as _, SeedableRng as _};
use rand_chacha::ChaCha12Rng;

/// RAM-only CSPRNG shared by crypto callers under a short critical section.
///
/// `None` makes use before startup seeding fail explicitly. The blocking mutex
/// protects the `RefCell` across executor tasks and interrupts, following the
/// Embassy Book's shared-resource pattern; no lock is held across an await.
static RNG: Mutex<CriticalSectionRawMutex, RefCell<Option<ChaCha12Rng>>> =
    Mutex::new(RefCell::new(None));

/// Initializes the RAM-only generator from 256 bits of hardware entropy.
///
/// Call once at boot before creating the BLE stack. The caller must enable a
/// true entropy source while collecting the seed and release any temporary ADC
/// source before wireless startup. No fallible I/O or persistent writes occur.
pub fn seed(seed: [u8; 32]) {
    RNG.lock(|rng| rng.replace(Some(ChaCha12Rng::from_seed(seed))));
}

/// The application's single Embassy RNG driver, independent of MCU hardware.
struct Driver;

impl embassy_crypto::driver::Rng for Driver {
    /// Fills a crypto buffer from ChaCha12 without awaiting or allocating.
    ///
    /// Panics if boot has not seeded the generator; an uninitialized or weak
    /// random fallback must never be used for passkeys or ECDH private keys.
    fn fill_bytes(buf: &mut [u8]) {
        RNG.lock(|rng| {
            rng.borrow_mut()
                .as_mut()
                .expect("CSPRNG used before it was seeded")
                .fill_bytes(buf);
        });
    }
}

embassy_crypto::rng_impl!(Driver);
