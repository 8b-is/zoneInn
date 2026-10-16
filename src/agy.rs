//! agy — the antigravity visuals lane.
//!
//! `agy` (the antigravity CLI, ultra subscription) can render ASCII art
//! for the now-playing track — the foobar2000-style visual that is more
//! than an equalizer. This module shells out to it when it is on `PATH`;
//! otherwise it returns `None` — the radio runs fine on the ternary
//! equalizer alone.

use std::process::Command;

/// Ask agy for a small ASCII card for the current track.
pub fn track_art(artist: &str, title: &str) -> Option<String> {
    let bin = std::env::var("AGY_BIN").unwrap_or_else(|_| "agy".to_string());
    track_art_with(&bin, artist, title)
}

/// The pure form: run `bin` with the art prompt.
fn track_art_with(bin: &str, artist: &str, title: &str) -> Option<String> {
    let prompt = format!(
        "render a tiny ASCII-art card (max 12 lines, 40 cols) for the track '{title}' by {artist}. no commentary."
    );
    let out = Command::new(bin).arg("-p").arg(&prompt).output().ok()?;
    if out.status.success() {
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        (!s.is_empty()).then_some(s)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_agy_returns_none() {
        assert_eq!(track_art_with("/definitely/not/agy", "test", "test"), None);
    }
}
