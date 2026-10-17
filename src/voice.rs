//! voice — the SUNO OSS voice between tracks.
//!
//! The open-source voice model in the SUNO lineage is Bark
//! (suno-ai/bark, MIT) — the constellation self-hosts it at
//! `bark-selfhost`. Between tracks, the radio speaks the station ident
//! through that endpoint; when no endpoint is configured the cue is
//! returned as text, honestly unspoken.

/// One voice cue — a line the radio would speak between tracks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VoiceCue {
    pub text: String,
    /// The speaker/voice preset, when the backend supports it.
    pub voice: Option<String>,
}

impl VoiceCue {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            voice: None,
        }
    }
}

/// The bark-selfhost endpoint, from the environment.
pub fn bark_endpoint() -> Option<String> {
    std::env::var("BARK_SELFHOST_URL")
        .ok()
        .filter(|v| !v.is_empty())
}

/// Speak a cue. With an endpoint configured this would POST the text to
/// the self-hosted Bark instance (the next slice wires the HTTP client);
/// without one, the cue returns unspoken — reported, not fabricated.
pub fn speak(cue: &VoiceCue) -> Result<bool, VoiceError> {
    match bark_endpoint() {
        Some(endpoint) => Err(VoiceError::NotWired(endpoint)),
        // No endpoint: the cue stays unspoken, honestly reported.
        None => {
            let _ = cue;
            Ok(false)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum VoiceError {
    /// The endpoint exists but the HTTP bridge is not wired yet.
    NotWired(String),
}

impl std::fmt::Display for VoiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VoiceError::NotWired(e) => write!(
                f,
                "bark endpoint configured ({e}) but the HTTP bridge is the next slice"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cue_is_text_and_voice() {
        let cue = VoiceCue::new("you are listening to radio.vaked.dev");
        assert_eq!(cue.text, "you are listening to radio.vaked.dev");
        assert_eq!(cue.voice, None);
    }

    #[test]
    fn no_endpoint_means_unspoken_and_honest() {
        // The environment in tests has no BARK_SELFHOST_URL by default;
        // if one leaks in, this still only asserts the honest contract.
        let spoken = speak(&VoiceCue::new("test")).unwrap_or(false);
        assert!(
            !spoken,
            "no endpoint configured — the cue must stay unspoken"
        );
    }
}
