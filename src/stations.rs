//! stations — the quantTernRadio dial.
//!
//! First-class: the constellation's own surfaces. Everything else rides the
//! generic `--url` path — tuneIn streams are URLs too, and the dial never
//! pretends a stream exists that has not been verified.

/// A station on the dial.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Station {
    pub name: &'static str,
    /// Streaming URL, when the station owns one. `None` = surface exists,
    /// stream not yet wired — listed honestly, never fabricated.
    pub stream_url: Option<&'static str>,
    pub owner: &'static str,
    pub note: &'static str,
}

/// The vaked dial — the two surfaces the constellation owns.
pub const VAKED_DIAL: [Station; 2] = [
    Station {
        name: "radio.vaked.dev",
        stream_url: Some("https://radio.vaked.dev"),
        owner: "8b-is",
        note: "the radio surface — Quantshuffle 1-bit + Ternary Canon ({−1, 0, +1})",
    },
    Station {
        name: "music.vaked.dev",
        stream_url: Some("https://music.vaked.dev"),
        owner: "8b-is",
        note: "the living ambient sound node — 432 Hz Verdi tuning + 528 Hz Star",
    },
];

/// Look up a station by name (exact, case-insensitive).
pub fn find(name: &str) -> Option<&'static Station> {
    VAKED_DIAL
        .iter()
        .find(|s| s.name.eq_ignore_ascii_case(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dial_names_are_unique_and_nonempty() {
        let names: Vec<&str> = VAKED_DIAL.iter().map(|s| s.name).collect();
        let mut uniq = names.clone();
        uniq.sort();
        uniq.dedup();
        assert_eq!(names.len(), uniq.len());
        assert!(names.iter().all(|n| !n.is_empty()));
    }

    #[test]
    fn lookup_is_case_insensitive() {
        assert!(find("radio.vaked.dev").is_some());
        assert!(find("RADIO.VAKED.DEV").is_some());
        assert!(find("no-such-station").is_none());
    }

    #[test]
    fn the_dial_never_fabricates_streams() {
        // Every listed stream must be a URL or honestly None — the check
        // pins the invariant, not the streams.
        for s in &VAKED_DIAL {
            if let Some(url) = s.stream_url {
                assert!(url.starts_with("http"), "{} stream is not a URL", s.name);
            }
        }
    }
}
