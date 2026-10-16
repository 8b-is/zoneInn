//! bt — safe bluetooth.
//!
//! "Safe" means two things here, and the crate claims only what it does:
//! 1. **Memory-safe by construction** — this crate is pure Rust; there is
//!    no `unsafe` FFI against the platform bluetooth stacks in this tree.
//!    The Linux backend (BlueZ via the pure-Rust `bluer` crate) is the
//!    first wire; macOS/iOS CoreBluetooth and Windows remain honest stubs
//!    until their pure-Rust bridges land.
//! 2. **First-class Soundcore Select 4 GO** — the speaker ships its own
//!    profile below, so the dial recognizes it by name, model, and class
//!    before anything else.

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
    /// Not yet wired — the scan reports nothing rather than pretending.
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

/// Match a discovered device name against the known profiles.
pub fn match_speaker(device_name: &str) -> Option<&'static SpeakerProfile> {
    KNOWN_SPEAKERS
        .iter()
        .find(|p| device_name.to_ascii_lowercase().contains(&p.name_match.to_ascii_lowercase()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_select_4_go_is_recognized() {
        assert!(match_speaker("Soundcore Select 4 Go").is_some());
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
        // Every non-Linux build is an honest stub until its bridge lands.
        #[cfg(target_os = "linux")]
        assert_eq!(platform(), BtPlatform::Linux);
        #[cfg(not(target_os = "linux"))]
        assert_eq!(platform(), BtPlatform::Stub);
    }
}
