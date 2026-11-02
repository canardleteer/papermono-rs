//! Host regression harness for the exact RNG and driver glue linked by firmware.

// Supplies a host critical-section implementation for the firmware mutex.
use critical_section as _;
use embassy_crypto::{p256::SecretKey, rng_fill_bytes, Aes128, Aes128Cmac};

#[path = "../../firmware/embassy-debug/src/csprng.rs"]
mod csprng;

fn hex<const N: usize>(text: &str) -> [u8; N] {
    assert_eq!(text.len(), N * 2);
    std::array::from_fn(|i| u8::from_str_radix(&text[2 * i..2 * i + 2], 16).unwrap())
}

#[test]
fn rng_requires_seeding_and_registered_ecdh_agrees() {
    // Keep initialization and all random consumers in this test so scheduling
    // cannot seed the shared global before the uninitialized-use probe.
    let mut bytes = [0xA5; 32];
    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        rng_fill_bytes(&mut bytes);
    }));
    let failure = failure.expect_err("uninitialized RNG must fail");
    assert_eq!(
        failure
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| failure.downcast_ref::<&str>().copied()),
        Some("CSPRNG used before it was seeded")
    );
    assert_eq!(bytes, [0xA5; 32]);

    csprng::seed([0; 32]);
    rng_fill_bytes(&mut bytes);
    // Portable ChaCha12 zero-seed prefix, also documented by rand_chacha 0.10.
    assert_eq!(&bytes[..8], &hex::<8>("9bf49a6a0755f953"));
    let first = bytes;
    rng_fill_bytes(&mut bytes);
    assert_ne!(bytes, first);
    csprng::seed([1; 32]);
    rng_fill_bytes(&mut bytes);
    assert_ne!(bytes, first);

    // These calls resolve the registered P-256 ECDH driver at final link and
    // generate both private scalars through the registered firmware RNG.
    let alice = SecretKey::generate().unwrap();
    let bob = SecretKey::generate().unwrap();
    let alice_public = alice.public_key().unwrap();
    let bob_public = bob.public_key().unwrap();
    assert_ne!(alice_public, bob_public);
    let shared = alice.diffie_hellman(&bob_public).unwrap();
    assert_eq!(
        shared.as_bytes(),
        bob.diffie_hellman(&alice_public).unwrap().as_bytes()
    );
    assert_ne!(shared.as_bytes(), &[0; 32]);
    assert!(SecretKey::from_bytes(&[0; 32]).is_err());
}

#[test]
fn registered_aes_ecb_and_cmac_match_nist_vectors() {
    // NIST SP 800-38A F.1.1 (ECB) and SP 800-38B examples 1/2 (CMAC).
    let key = hex("2b7e151628aed2a6abf7158809cf4f3c");
    let plaintext = hex("6bc1bee22e409f96e93d7e117393172a");
    let cipher = Aes128::new(&key);
    let mut block = plaintext;
    cipher.encrypt_block(&mut block);
    assert_eq!(block, hex("3ad77bb40d7a3660a89ecaf32466ef97"));
    cipher.decrypt_block(&mut block);
    assert_eq!(block, plaintext);
    assert_eq!(
        Aes128Cmac::new(&key).finalize(),
        hex("bb1d6929e95937287fa37d129b756746")
    );
    let mut mac = Aes128Cmac::new(&key);
    mac.update(&plaintext[..7]);
    mac.update(&plaintext[7..]);
    assert_eq!(mac.finalize(), hex("070a16b46b4d4144f79bdd9dd04a287c"));
}
