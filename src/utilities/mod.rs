use std::fs::File;
use std::io::Read;
use std::path::Path;

// Far above any program or set the app writes; a wrong pick must not fill the memory.
const FILE_LIMIT: u64 = 16 * 1024 * 1024;

pub fn read_text(path: &Path) -> anyhow::Result<String> {
    let mut bytes = Vec::new();

    File::open(path)?
        .take(FILE_LIMIT + 1)
        .read_to_end(&mut bytes)?;
    anyhow::ensure!(
        bytes.len() as u64 <= FILE_LIMIT,
        "the file is larger than 16 MB"
    );

    Ok(String::from_utf8(bytes)?)
}

/**
 * Take a string num representation f.e 1234 or 0x1234
 * Strip 0x or 0X prefix if present
 * Parse the value and returnf if successful; otherwise None
 */
pub fn parse_value(raw: &str) -> Option<i32> {
    let raw = raw.trim();

    match raw.strip_prefix("0x").or_else(|| raw.strip_prefix("0X")) {
        Some(hex) => i32::from_str_radix(hex, 16).ok(),
        None => raw.parse().ok(),
    }
}
