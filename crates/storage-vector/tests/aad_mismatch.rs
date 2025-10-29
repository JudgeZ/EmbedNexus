#![cfg(feature = "encryption")]
use storage_vector::encryption::{aes_gcm::AesGcmEncrypter, Encrypter, KeyHandle};
use storage_vector::store::{build_aad, fs as vs_fs};
use tempfile::tempdir;
use zeroize::Zeroizing;

#[test]
fn aad_mismatch_fails_decrypt() {
    let tmp = tempdir().unwrap();
    let root = tmp.path().join("vs");
    let repo = "repo-a";
    let key = "k";
    let payload = b"hello".to_vec();
    // Produce an encrypted envelope and write it directly to FS to isolate AAD behavior.
    let enc = AesGcmEncrypter::new();
    let kh = KeyHandle {
        key_id: "k1".into(),
        key_bytes: Zeroizing::new([3u8; 32]),
    };
    let good_aad = build_aad(repo, &kh.key_id, key);
    let env = enc.seal(&kh, &payload, &good_aad).expect("seal");
    vs_fs::atomic_write_bytes(&root, repo, key, &env).expect("write env");

    // Dump raw envelope bytes and attempt open() with mismatched AAD
    let raw = vs_fs::read_bytes(&root, repo, key).expect("raw present");
    // Wrong repo id in AAD should fail
    let bad_aad = build_aad("repo-b", &kh.key_id, key);
    let res = enc.open(&kh, &raw, &bad_aad);
    assert!(res.is_err());
}
