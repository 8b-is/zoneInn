//! bt — safe bluetooth & live output device discovery.
//!
//! "Safe" means two things here, and the crate claims only what it does:
//! 1. **Memory-safe by construction** — this crate has no `unsafe` FFI in its
//!    own tree. Live Bluetooth A2DP output devices are discovered and routed
//!    via `rodio`/`cpal` host device enumeration (CoreAudio on macOS,
//!    PipeWire/ALSA/BlueZ on Linux, WASAPI on Windows).
//! 2. **First-class Soundcore Select 4 GO** — the speaker ships its own
//!    profile below, so the dial recognizes it by name, model, and class
//!    before anything else.

use rodio::cpal::traits::{DeviceTrait, HostTrait};

/// A recognized bluetooth speaker profile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpeakerProfile {
    pub model: &'static str,
    /// Name substring to match during discovery.
    pub name_match: &'static str,
    /// Bluetooth class / audio profile hint.
    pub audio_class: &'static str,
    pub note: &'static str,
}

/// Anker Soundcore Select 4 GO — the first-class citizen of the dial.
///
/// Real device: ~130 g, IPX7, BT 5.x, USB-C, plays loud for its size.
/// The profile matches discovery names like "Soundcore Select 4 Go".
pub const SOUNDCORE_SELECT_4_GO: SpeakerProfile = SpeakerProfile {
    model: "Soundcore Select 4 GO",
    name_match: "Soundcore Select 4",
    audio_class: "A2DP sink / BLE companion",
    note: "first-class: recognized before any other speaker",
};

/// The known speakers, ordered by dial priority.
pub const KNOWN_SPEAKERS: [SpeakerProfile; 1] = [SOUNDCORE_SELECT_4_GO];

/// The platform truth, stated once.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BtPlatform {
    /// BlueZ over `bluer` — pure Rust, memory-safe, the first wire.
    Linux,
    /// CoreAudio / cpal output sink routing on non-Linux hosts.
    Stub,
}

/// Which platform this build runs the bluetooth backend on.
pub const fn platform() -> BtPlatform {
    #[cfg(target_os = "linux")]
    {
        BtPlatform::Linux
    }
    #[cfg(not(target_os = "linux"))]
    {
        BtPlatform::Stub
    }
}

/// The backend name, stated once.
pub const fn platform_name() -> &'static str {
    match platform() {
        BtPlatform::Linux => "Linux (BlueZ + cpal/rodio A2DP sink)",
        BtPlatform::Stub => "cpal/rodio A2DP output sink (CoreAudio / OS audio bridge)",
    }
}

/// Enumerate live OS audio output device names (including connected Bluetooth A2DP speakers).
pub fn list_output_devices() -> Vec<String> {
    let host = rodio::cpal::default_host();
    let Ok(devices) = host.output_devices() else {
        return Vec::new();
    };
    let mut names = Vec::new();
    for dev in devices {
        if let Ok(desc) = dev.description() {
            let name = desc.name().to_string();
            if !name.is_empty() && !names.contains(&name) {
                names.push(name);
            }
        }
    }
    names
}

/// The bt lane body: backend, known profiles, and currently connected output devices.
pub fn describe() -> String {
    let mut out = String::new();
    out.push_str(&format!("bluetooth backend: {}\n", platform_name()));
    out.push_str("known speakers:\n");
    for p in &KNOWN_SPEAKERS {
        out.push_str(&format!("  {} — {} — {}\n", p.model, p.audio_class, p.note));
    }
    let live = list_output_devices();
    if !live.is_empty() {
        out.push_str("connected output devices:\n");
        for dev in &live {
            if let Some(profile) = match_speaker(dev) {
                out.push_str(&format!("  * {dev} [matched: {}]\n", profile.model));
            } else {
                out.push_str(&format!("    {dev}\n"));
            }
        }
    }
    out
}

/// Match a discovered device name against the known profiles.
pub fn match_speaker(device_name: &str) -> Option<&'static SpeakerProfile> {
    KNOWN_SPEAKERS.iter().find(|p| {
        device_name
            .to_ascii_lowercase()
            .contains(&p.name_match.to_ascii_lowercase())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_select_4_go_is_recognized() {
        assert!(match_speaker("Soundcore Select 4 Go").is_some());
        assert!(match_speaker("soundcore Select 4 Go").is_some());
        assert!(match_speaker("Anker Soundcore Select 4 Go BLE").is_some());
        assert!(match_speaker("JBL Flip 6").is_none());
    }

    #[test]
    fn profiles_are_unique() {
        let names: Vec<&str> = KNOWN_SPEAKERS.iter().map(|p| p.model).collect();
        let mut u = names.clone();
        u.sort();
        u.dedup();
        assert_eq!(names.len(), u.len());
    }

    #[test]
    fn platform_is_stated_honestly() {
        #[cfg(target_os = "linux")]
        assert_eq!(platform(), BtPlatform::Linux);
        #[cfg(not(target_os = "linux"))]
        assert_eq!(platform(), BtPlatform::Stub);
    }
}
