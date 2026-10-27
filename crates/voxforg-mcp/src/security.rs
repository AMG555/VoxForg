//! Security controls and input hardening for VoxForg MCP server.
//!
//! Enforces:
//! - Strict path validation to prevent directory traversal and system file overwrites.
//! - Bounds checking on textual and audio base64/hex inputs to prevent memory exhaustion (DoS).
//! - Safe base64 / hex audio decoding with memory-efficient decoding.

use std::path::{Path, PathBuf};

/// Maximum allowed character length for text inputs to prevent memory exhaustion.
pub const MAX_TEXT_INPUT_CHARS: usize = 100_000;

/// Maximum allowed string length for base64 / hex audio payloads (~32 MB).
pub const MAX_AUDIO_PAYLOAD_CHARS: usize = 32 * 1024 * 1024;

/// Permitted file extensions for input audio/media files.
const ALLOWED_INPUT_EXTENSIONS: &[&str] = &[
    "wav", "wave", "mp3", "mp4", "mkv", "mov", "webm", "flac", "ogg", "pcm", "json",
];

/// Permitted file extensions for generated output files.
const ALLOWED_OUTPUT_EXTENSIONS: &[&str] = &["wav", "pcm", "json", "srt", "vtt", "txt"];

/// Validate that text input does not exceed safe length.
pub fn validate_text_length(text: &str, field_name: &str) -> Result<(), String> {
    if text.len() > MAX_TEXT_INPUT_CHARS {
        return Err(format!(
            "Input field '{field_name}' exceeds maximum allowed length of {} characters (received {})",
            MAX_TEXT_INPUT_CHARS,
            text.len()
        ));
    }
    Ok(())
}

/// Validate that audio string payload does not exceed memory safety caps.
pub fn validate_audio_payload_length(payload: &str, field_name: &str) -> Result<(), String> {
    if payload.len() > MAX_AUDIO_PAYLOAD_CHARS {
        return Err(format!(
            "Audio payload '{field_name}' exceeds maximum allowed length of {} bytes (received {})",
            MAX_AUDIO_PAYLOAD_CHARS,
            payload.len()
        ));
    }
    Ok(())
}

/// Validate and sanitize an input file path to prevent directory traversal and sensitive file access.
pub fn validate_safe_input_path(raw_path: &str) -> Result<PathBuf, String> {
    validate_safe_path_internal(raw_path, ALLOWED_INPUT_EXTENSIONS, false)
}

/// Validate and sanitize an output file path to ensure safe directory target and permitted extension.
pub fn validate_safe_output_path(raw_path: &str) -> Result<PathBuf, String> {
    validate_safe_path_internal(raw_path, ALLOWED_OUTPUT_EXTENSIONS, true)
}

fn validate_safe_path_internal(
    raw_path: &str,
    allowed_exts: &[&str],
    is_output: bool,
) -> Result<PathBuf, String> {
    let trimmed = raw_path.trim();
    if trimmed.is_empty() {
        return Err("File path cannot be empty".to_string());
    }

    // Disallow null bytes
    if trimmed.contains('\0') {
        return Err("File path contains forbidden null byte".to_string());
    }

    let path = Path::new(trimmed);

    // Check for directory traversal sequences ("..")
    for component in path.components() {
        if let std::path::Component::ParentDir = component {
            return Err("Parent directory traversal ('..') is strictly forbidden for security".to_string());
        }
    }

    // Extension validation
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .ok_or_else(|| {
            format!(
                "File path must have an allowed extension: [{}]",
                allowed_exts.join(", ")
            )
        })?;

    if !allowed_exts.contains(&ext.as_str()) {
        return Err(format!(
            "File extension '.{ext}' is not permitted. Allowed extensions: [{}]",
            allowed_exts.join(", ")
        ));
    }

    // Block sensitive system directories
    let path_str_lower = trimmed.to_lowercase().replace('/', "\\");
    let forbidden_prefixes = [
        "c:\\windows",
        "c:\\program files",
        "c:\\program files (x86)",
        "c:\\system volume information",
        "/etc",
        "/proc",
        "/sys",
        "/bin",
        "/sbin",
        "/usr/bin",
        "/dev",
        "/root",
    ];

    for prefix in forbidden_prefixes {
        if path_str_lower.starts_with(prefix) {
            return Err(format!("Access to system directory '{prefix}' is blocked for security"));
        }
    }

    // Ensure parent directory exists for output files
    if is_output {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                let _ = std::fs::create_dir_all(parent);
            }
        }
    }

    Ok(path.to_path_buf())
}

/// Safely decode audio from hex, base64 (with or without data URL), or raw bytes.
pub fn safe_decode_audio_bytes(payload: &str) -> Result<Vec<u8>, String> {
    validate_audio_payload_length(payload, "audio")?;

    let trimmed = payload.trim();
    if trimmed.is_empty() {
        return Err("Audio payload cannot be empty".to_string());
    }

    // Check data URI prefix
    let clean = if let Some(idx) = trimmed.find(";base64,") {
        &trimmed[idx + 8..]
    } else if let Some(idx) = trimmed.find(',') {
        &trimmed[idx + 1..]
    } else {
        trimmed
    };

    let clean = clean.trim();

    // Check if hex encoded
    if clean.len() >= 8 && clean.len() % 2 == 0 && clean.chars().take(32).all(|c| c.is_ascii_hexdigit()) {
        if let Ok(bytes) = hex::decode(clean) {
            return Ok(bytes);
        }
    }

    // Base64 decode
    decode_base64(clean).ok_or_else(|| "Failed to decode base64 or hex audio payload".to_string())
}

fn decode_base64(input: &str) -> Option<Vec<u8>> {
    let mut table = [0xFFu8; 256];
    for (i, &b) in b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
        .iter()
        .enumerate()
    {
        table[b as usize] = i as u8;
    }
    table[b'-' as usize] = 62;
    table[b'_' as usize] = 63;

    let filtered: Vec<u8> = input
        .bytes()
        .filter(|&b| !b.is_ascii_whitespace())
        .collect();
    if filtered.is_empty() {
        return None;
    }

    let mut out = Vec::with_capacity(filtered.len() * 3 / 4);
    let mut buf = 0u32;
    let mut bits = 0;

    for &b in &filtered {
        if b == b'=' {
            break;
        }
        let val = table[b as usize];
        if val == 0xFF {
            continue;
        }
        buf = (buf << 6) | (val as u32);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }

    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_safe_path_traversal() {
        assert!(validate_safe_output_path("../malicious.wav").is_err());
        assert!(validate_safe_output_path("dir/../../secret.wav").is_err());
        assert!(validate_safe_output_path("safe_output.wav").is_ok());
        assert!(validate_safe_output_path("sub/folder/audio.pcm").is_ok());
    }

    #[test]
    fn test_validate_safe_path_extensions() {
        assert!(validate_safe_output_path("audio.exe").is_err());
        assert!(validate_safe_output_path("audio.bat").is_err());
        assert!(validate_safe_output_path("audio.wav").is_ok());
        assert!(validate_safe_output_path("speech.srt").is_ok());
    }

    #[test]
    fn test_validate_text_length_limit() {
        assert!(validate_text_length("normal short text", "prompt").is_ok());
        let huge = "a".repeat(MAX_TEXT_INPUT_CHARS + 1);
        assert!(validate_text_length(&huge, "prompt").is_err());
    }
}
