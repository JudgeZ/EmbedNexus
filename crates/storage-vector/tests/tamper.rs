#![cfg(feature = "encryption")]

#[test]
#[ignore = "M3 integrity checks pending"]
fn detects_tampered_envelope() {
    // TODO: manipulate ciphertext/tag and expect integrity error on open.
    todo!("integrity guard to be implemented");
}

