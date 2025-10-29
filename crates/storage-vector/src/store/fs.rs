use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

fn encode_component(s: &str) -> String {
    // simple percent-encoding for path safety
    s.bytes()
        .flat_map(|b| match b {
            b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'-' | b'_' | b'.' => vec![b],
            _ => format!("%{:02X}", b).into_bytes(),
        })
        .map(|b| b as char)
        .collect()
}

pub fn make_path(root: &Path, repo_id: &str, key: &str) -> PathBuf {
    root.join(encode_component(repo_id)).join(encode_component(key))
}

pub fn atomic_write_bytes(root: &Path, repo_id: &str, key: &str, bytes: &[u8]) -> std::io::Result<()> {
    let path = make_path(root, repo_id, key);
    if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
    let tmp = path.with_extension("tmp");
    {
        let mut f = File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    fs::rename(tmp, &path)?;
    Ok(())
}

pub fn read_bytes(root: &Path, repo_id: &str, key: &str) -> std::io::Result<Vec<u8>> {
    let path = make_path(root, repo_id, key);
    let mut buf = Vec::new();
    let mut f = File::open(path)?;
    f.read_to_end(&mut buf)?;
    Ok(buf)
}

