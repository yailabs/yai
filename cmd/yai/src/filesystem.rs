//! Filesystem resource mechanics retained by the current Rust command path.
//!
//! This module owns the demonstrated lexical sandbox check and direct
//! filesystem read/write mechanics. It does not provide constitutional
//! admission, grants, receipts, or crash-safe effect finalization.

use super::*;

pub(super) fn path_inside_sandbox(sandbox: &str, path: &str) -> bool {
    if sandbox.is_empty()
        || path.is_empty()
        || sandbox.split('/').any(|part| part == "..")
        || path.split('/').any(|part| part == "..")
    {
        return false;
    }
    path == sandbox || path.starts_with(&format!("{sandbox}/"))
}

fn fnv_hash(bytes: &[u8]) -> String {
    let mut value = 0xcbf29ce484222325u64;
    for byte in bytes {
        value ^= u64::from(*byte);
        value = value.wrapping_mul(0x100000001b3);
    }
    format!("{value:016x}")
}

pub(super) fn carrier_fs_read(args: &[String]) -> Result<(), String> {
    let sandbox = named_arg(args, "--sandbox")?;
    let path = named_arg(args, "--path")?;
    if !path_inside_sandbox(&sandbox, &path) {
        return Err("path is outside sandbox".to_string());
    }
    let bytes = fs::read(&path).map_err(|error| format!("failed to read {path}: {error}"))?;
    println!("carrier: filesystem");
    println!("effect: fs.read");
    println!("status: observed");
    println!("bytes: {}", bytes.len());
    println!("hash: {}", fnv_hash(&bytes));
    Ok(())
}

pub(super) fn carrier_fs_write(args: &[String]) -> Result<(), String> {
    let sandbox = named_arg(args, "--sandbox")?;
    let path = named_arg(args, "--path")?;
    let content = named_arg(args, "--content")?;
    if !path_inside_sandbox(&sandbox, &path) {
        return Err("path is outside sandbox".to_string());
    }
    let mut file =
        fs::File::create(&path).map_err(|error| format!("failed to open {path}: {error}"))?;
    file.write_all(content.as_bytes())
        .map_err(|error| format!("failed to write {path}: {error}"))?;
    println!("carrier: filesystem");
    println!("effect: fs.write");
    println!("status: executed");
    println!("bytes: {}", content.len());
    println!("hash: {}", fnv_hash(content.as_bytes()));
    Ok(())
}
