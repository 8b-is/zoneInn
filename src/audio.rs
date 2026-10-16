//! audio — the playback lane.
//!
//! Playback is the declared next slice: rodio 0.23 restructured its stream
//! API (Player/SpatialPlayer), so this crate does not fight it yet. What
//! ships today is honest validation plus the status constant — the
//! visualizer needs no audio device at all.

use std::path::Path;

/// The status line for URL/file playback — stated, never stubbed silently.
pub const URL_STREAM_STATUS: &str =
    "playback: next slice (rodio 0.23 Player + symphonia + http reader)";

/// Validate that a local file exists and is readable.
pub fn file_readable(path: &str) -> Result<u64, String> {
    let meta = std::fs::metadata(Path::new(path)).map_err(|e| e.to_string())?;
    if meta.is_file() {
        Ok(meta.len())
    } else {
        Err(format!("{path} is not a file"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_is_an_error_not_a_panic() {
        assert!(file_readable("/definitely/not/a/file.mp3").is_err());
    }

    #[test]
    fn this_source_file_is_readable() {
        let n = file_readable(file!()).expect("source file readable");
        assert!(n > 0);
    }
}
