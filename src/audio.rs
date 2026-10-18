//! audio — the live playback lane (`rodio 0.23` + Bluetooth sink routing).
//!
//! Routes the constellation's own stations (`radio.vaked.dev` with the 14-track
//! Favourite Musics crate, `music.vaked.dev`, `nand.vaked.dev`, `mirror.vaked.dev`),
//! local audio files, and HTTP stream URLs directly to the selected OS / Bluetooth
//! A2DP output device (with first-class recognition and routing for
//! `Soundcore Select 4 GO`).

use std::fs::File;
use std::io::{self, Cursor, Write};
use std::num::NonZero;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::{
    ChannelCount, Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Sample, SampleRate, Source,
};

use crate::favourites::{self, FAVOURITE_MUSICS, FavSynthStyle, FavouriteTrack};
use crate::{bt, nand, visuals};

/// Status line describing the active playback engine.
pub const URL_STREAM_STATUS: &str =
    "playback: live (rodio 0.23 MixerDeviceSink + Player + Favourite Musics 320k/432Hz engine)";

const SYNTH_SAMPLE_RATE: u32 = 48_000;
const SYNTH_CHANNELS: u16 = 2;

/// Validate that a local file exists and is readable.
pub fn file_readable(path: &str) -> Result<u64, String> {
    let meta = std::fs::metadata(Path::new(path)).map_err(|e| e.to_string())?;
    if meta.is_file() {
        Ok(meta.len())
    } else {
        Err(format!("{path} is not a file"))
    }
}

/// Which procedural station synthesizer to run for a constellation dial target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StationSynthKind {
    /// `radio.vaked.dev` — rotates through the 14 Favourite Musics crate + Ternary Canon.
    RadioQuantshuffle,
    /// Specific favourite track (`0..14` index into `FAVOURITE_MUSICS`) on `radio.vaked.dev`.
    Favourite(usize),
    /// `music.vaked.dev` — 432 Hz Verdi tuning (`D#m → B → F# → C#`) + 528 Hz Star + 288 Hz Pillar.
    Music432,
    /// `nand.vaked.dev` — 5-stage cross-coupled Ultrawhale NAND ring oscillator (`{−1, 0, +1}`).
    NandOscillator,
    /// `mirror.vaked.dev` — Mirror in F tintinnabuli overtones (`F2 = 87.31 Hz`).
    MirrorInF,
}

impl StationSynthKind {
    /// Match a station name, vaked.dev URL, or favourite track slug to its synthesizer.
    pub fn from_target(target: &str) -> Option<Self> {
        let lower = target
            .trim()
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_end_matches('/')
            .to_ascii_lowercase();
        match lower.as_str() {
            "radio.vaked.dev" | "zoneinn.vaked.dev" | "radio" => Some(Self::RadioQuantshuffle),
            "music.vaked.dev" | "music" => Some(Self::Music432),
            "nand.vaked.dev" | "nand" => Some(Self::NandOscillator),
            "mirror.vaked.dev" | "mirror" => Some(Self::MirrorInF),
            _ => {
                let sub = lower
                    .strip_prefix("radio.vaked.dev/")
                    .or_else(|| lower.strip_prefix("zoneinn.vaked.dev/"))
                    .unwrap_or(lower.as_str());
                favourites::find_favourite(sub).map(|t| Self::Favourite(t.index - 1))
            }
        }
    }

    pub fn description(self) -> String {
        match self {
            Self::RadioQuantshuffle => {
                "Favourite Musics Crate (14 tracks: Mahakala · Innerbloom · Faded · Billie · Lorn · Massano · ARTBAT · P!NK · DRA <3 · Space Floating · Oblivion) + Ternary Canon".to_string()
            }
            Self::Favourite(idx) => {
                let t = &FAVOURITE_MUSICS[idx % FAVOURITE_MUSICS.len()];
                format!(
                    "{} {} — {} ({} · {} BPM · {} Hz)",
                    t.emoji, t.title, t.artist, t.key_progression, t.bpm, t.tuning_hz
                )
            }
            Self::Music432 => {
                "432 Hz Verdi tuning (D#m → B → F# → C#) + 528 Hz Star + 288 Hz Pillar".to_string()
            }
            Self::NandOscillator => {
                "5-stage cross-coupled Ultrawhale NAND Ring Oscillator ({−1, 0, +1})".to_string()
            }
            Self::MirrorInF => "Mirror in F tintinnabuli overtones (F2 = 87.31 Hz)".to_string(),
        }
    }
}

/// Real-time stereo `rodio::Source` for the vaked dial stations and Favourite Musics crate.
pub struct VakedStationSource {
    kind: StationSynthKind,
    sample_idx: u64,
    channel: u8,
    last_right: f32,
    nand_step: u64,
    ring: [i8; nand::RING_STAGES],
    kleene: [i8; nand::RING_STAGES],
}

impl VakedStationSource {
    pub fn new(kind: StationSynthKind) -> Self {
        Self {
            kind,
            sample_idx: 0,
            channel: 0,
            last_right: 0.0,
            nand_step: 0,
            ring: nand::INITIAL_RING,
            kleene: nand::INITIAL_KLEENE,
        }
    }

    /// Synthesize one stereo frame for a `FavouriteTrack` at time `t`.
    fn synth_favourite_track(&self, track: &FavouriteTrack, t: f64) -> (f32, f32) {
        let tau = std::f64::consts::TAU;
        let beat_sec = 60.0 / (track.bpm.max(60) as f64);
        let sixteenth = beat_sec * 0.25;
        let step16 = (t / sixteenth) as usize;
        let s16 = step16 % 16;
        let bar = step16 / 16;
        let chord_idx = bar % 4;

        let root_midi = track.roots_midi[chord_idx];
        let chord_midi = track.chords_midi[chord_idx];
        let lead_midi = track.lead_midi[s16 % track.lead_midi.len()];

        let root_hz = favourites::midi_to_hz_432(root_midi);
        let sub_hz = root_hz * 0.5;
        let lead_hz = favourites::midi_to_hz_432(lead_midi);

        // 1. Warm 4-voice 432 Hz pad
        let mut pad_l = 0.0_f64;
        let mut pad_r = 0.0_f64;
        for (i, &m) in chord_midi.iter().enumerate() {
            let f = favourites::midi_to_hz_432(m);
            let detune = if i % 2 == 0 { 1.0008 } else { 0.9992 };
            pad_l += 0.048 * (tau * f * t).sin();
            pad_r += 0.048 * (tau * f * detune * t).sin();
        }

        // 2. Beat & sixteenth envelopes
        let beat_pos = t % beat_sec;
        let sixteenth_pos = t % sixteenth;
        let kick_env = (-beat_pos * 11.0).exp();
        let pluck_env = (-sixteenth_pos * 14.0).exp();

        let trit = self.ring[s16 % nand::RING_STAGES];
        let pan = 0.5 + 0.22 * (trit as f64);

        let (mut left, mut right) = (pad_l, pad_r);

        match track.style {
            FavSynthStyle::Mahakala => {
                // 108 Hz Tibetan Dungchen horn + 4x4 sub kick + Kagyu throat formant + Vajra arp
                let dungchen = 0.075 * (tau * 108.0 * t).sin() + 0.04 * (tau * 54.0 * t).sin();
                let bowl = 0.03 * (tau * 528.0 * t).sin() * (-beat_pos * 2.5).exp();
                let kick = 0.24 * kick_env * (tau * sub_hz * t).sin();
                let arp = 0.11 * pluck_env * (tau * lead_hz * t).sin()
                    + 0.04 * pluck_env * (tau * lead_hz * 2.0 * t).sin();
                left += dungchen + bowl + kick + arp * pan;
                right += dungchen + bowl * 0.9 + kick + arp * (1.0 - pan);
            }
            FavSynthStyle::Innerbloom => {
                // Rolling 16th-note Joshua Tree / Hordern Pavilion arp + deep sub-bass
                let sub =
                    0.22 * kick_env * (tau * sub_hz * t).sin() + 0.09 * (tau * root_hz * t).sin();
                let oct = if s16 >= 8 { 2.0 } else { 1.0 };
                let arp = 0.13 * pluck_env * (tau * lead_hz * oct * t).sin()
                    + 0.05 * pluck_env * (tau * lead_hz * 1.498 * t).sin();
                left += sub + arp * pan;
                right += sub + arp * (1.0 - pan);
            }
            FavSynthStyle::Faded => {
                // Quarter-note felt piano + Alan Walker supersaw chorus + 528 Hz Star + 288 Hz Pillar
                let piano_env = (-beat_pos * 4.5).exp();
                let beat_lead =
                    favourites::midi_to_hz_432(track.lead_midi[(s16 / 4) % track.lead_midi.len()]);
                let piano = 0.14 * piano_env * (tau * beat_lead * t).sin()
                    + 0.05 * piano_env * (tau * beat_lead * 2.0 * t).sin();
                let star = 0.03 * (tau * 528.0 * t).sin();
                let pillar = 0.04 * (tau * 288.0 * t).sin();
                let bass = 0.16 * piano_env * (tau * root_hz * t).sin();
                left += piano * pan + star + pillar + bass;
                right += piano * (1.0 - pan) + star + pillar + bass;
            }
            FavSynthStyle::EverythingIWanted => {
                // Warm Rhodes sanctuary chords + syncopated bass & pluck motif
                let syncopated = matches!(s16, 0 | 3 | 6 | 10 | 14);
                let note_env = if syncopated {
                    pluck_env
                } else {
                    pluck_env * 0.35
                };
                let lead = 0.12 * note_env * (tau * lead_hz * t).sin()
                    + 0.04 * note_env * (tau * (lead_hz * 2.0) * t).sin();
                let bass = 0.19 * kick_env * (tau * root_hz * t).sin();
                left += lead * pan + bass;
                right += lead * (1.0 - pan) + bass;
            }
            FavSynthStyle::Anvil => {
                // Dystopian deep house sub + rising Lorn Reese tension + metallic anvil tick
                let tension = (((bar % 4) * 16 + s16) as f64) / 64.0;
                let reese_f = lead_hz * (1.0 + 0.04 * tension);
                let reese =
                    0.11 * (tau * reese_f * t).sin() + 0.08 * (tau * (reese_f * 0.501) * t).sin();
                let anvil_tick = if s16 == 4 || s16 == 12 {
                    0.12 * pluck_env * (tau * (root_hz * 8.0) * t).sin()
                } else {
                    0.0
                };
                let sub = 0.25 * kick_env * (tau * sub_hz * t).sin();
                left += sub + reese * pan + anvil_tick;
                right += sub + reese * (1.0 - pan) + anvil_tick;
            }
            FavSynthStyle::MassanoDrop | FavSynthStyle::ArtbatBenassi | FavSynthStyle::DrownOm => {
                // Melodic techno 4x4 sub kick + offbeat bass + anthemic detuned arp
                let is_offbeat = s16 % 2 == 1;
                let kick = 0.26 * kick_env * (tau * sub_hz * t).sin();
                let off_bass = if is_offbeat {
                    0.16 * pluck_env * (tau * root_hz * t).sin()
                } else {
                    0.0
                };
                let arp = 0.12 * pluck_env * (tau * lead_hz * t).sin()
                    + 0.08 * pluck_env * (tau * (lead_hz * 1.003) * t).sin();
                left += kick + off_bass + arp * pan;
                right += kick + off_bass + arp * (1.0 - pan);
            }
            FavSynthStyle::TimeAfterTime
            | FavSynthStyle::AllMyFavoriteColors
            | FavSynthStyle::SpaceFloating => {
                // Gentle heartbeat lub-dub sub + warm acoustic/Rhodes pluck
                let sub = 0.18 * kick_env * (tau * root_hz * t).sin();
                let pluck = 0.12 * pluck_env * (tau * lead_hz * t).sin()
                    + 0.05 * pluck_env * (tau * (lead_hz * 2.0) * t).sin();
                left += sub + pluck * pan;
                right += sub + pluck * (1.0 - pan);
            }
            FavSynthStyle::AnswerToOblivion | FavSynthStyle::SovereignFrequencies => {
                // Progressive 432 Hz + 528 Hz Guardian Bed + sacred overtone lead
                let guardian = 0.045 * (tau * 528.0 * t).sin();
                let sub = 0.22 * kick_env * (tau * sub_hz * t).sin();
                let lead = 0.12 * pluck_env * (tau * lead_hz * t).sin();
                left += guardian + sub + lead * pan;
                right += guardian + sub + lead * (1.0 - pan);
            }
            FavSynthStyle::QuantshuffleCanon => {
                const CANON: [f64; 32] = [
                    392.00, 392.00, 392.00, 311.13, 349.23, 349.23, 349.23,
                    293.66, // Beethoven V
                    329.63, 329.63, 349.23, 392.00, 392.00, 349.23, 329.63,
                    293.66, // Ode to Joy
                    659.25, 622.25, 659.25, 622.25, 659.25, 493.88, 587.33,
                    523.25, // Für Elise
                    293.66, 220.00, 246.94, 185.00, 196.00, 146.83, 196.00,
                    220.00, // Pachelbel
                ];
                let c_freq = CANON[step16 % CANON.len()];
                let lead = 0.15 * pluck_env * (tau * c_freq * t).sin();
                let drone = 0.07 * (tau * 110.0 * t).sin();
                left += drone + lead * pan;
                right += drone + lead * (1.0 - pan);
            }
        }

        (
            (left as f32).clamp(-0.9, 0.9),
            (right as f32).clamp(-0.9, 0.9),
        )
    }

    fn synth_stereo_frame(&mut self) -> (f32, f32) {
        let t = self.sample_idx as f64 / SYNTH_SAMPLE_RATE as f64;
        self.sample_idx = self.sample_idx.wrapping_add(1);

        // Advance the 5-stage Ultrawhale NAND ring every 180ms (8,640 frames at 48 kHz)
        let target_step = (t / 0.18) as u64;
        while self.nand_step <= target_step {
            let st = nand::step_ring(self.ring, self.kleene, self.nand_step);
            self.ring = st.ring;
            self.kleene = st.kleene;
            self.nand_step += 1;
        }

        let tau = std::f64::consts::TAU;
        match self.kind {
            StationSynthKind::RadioQuantshuffle => {
                // Rotate through all 14 Favourite Musics every 16 seconds on radio.vaked.dev
                let fav_idx = ((t / 16.0) as usize) % FAVOURITE_MUSICS.len();
                self.synth_favourite_track(&FAVOURITE_MUSICS[fav_idx], t)
            }
            StationSynthKind::Favourite(idx) => {
                let fav = &FAVOURITE_MUSICS[idx % FAVOURITE_MUSICS.len()];
                self.synth_favourite_track(fav, t)
            }
            StationSynthKind::Music432 => {
                // 432 Hz Verdi tuning: D#m -> B -> F# -> C# + 528 Hz Star + 288 Hz Pillar
                const CHORDS: [[f64; 3]; 4] = [
                    [152.74, 181.63, 228.84],
                    [121.23, 152.74, 181.63],
                    [181.63, 228.84, 272.14],
                    [136.07, 171.44, 203.88],
                ];
                let chord_idx = ((t / 2.4) as usize) % CHORDS.len();
                let chord = CHORDS[chord_idx];
                let lfo = 0.75 + 0.25 * (tau * 0.22 * t).sin();
                let pad_l = 0.09 * (tau * chord[0] * t).sin()
                    + 0.08 * (tau * chord[1] * t).sin()
                    + 0.07 * (tau * chord[2] * t).sin();
                let pad_r = 0.09 * (tau * (chord[0] * 1.001) * t).sin()
                    + 0.08 * (tau * chord[1] * t).sin()
                    + 0.07 * (tau * (chord[2] * 0.999) * t).sin();
                let star = 0.035 * (tau * 528.0 * t).sin() * (0.5 + 0.5 * (tau * 0.15 * t).cos());
                let pillar = 0.06 * (tau * 288.0 * t).sin();
                let left = ((pad_l + star + pillar) * lfo) as f32;
                let right = ((pad_r + star * 0.9 + pillar) * lfo) as f32;
                (left.clamp(-0.9, 0.9), right.clamp(-0.9, 0.9))
            }
            StationSynthKind::NandOscillator => {
                const STAGE_FREQS: [f64; 5] = [216.0, 288.0, 324.0, 432.0, 528.0];
                let note_t = t % 0.18;
                let env = (-note_t * 5.0).exp();
                let mut acc_l = 0.06 * (tau * 108.0 * t).sin();
                let mut acc_r = 0.06 * (tau * 162.0 * t).sin();
                for (i, &base_f) in STAGE_FREQS.iter().enumerate() {
                    let r_trit = self.ring[i];
                    let k_trit = self.kleene[i];
                    let f = if r_trit > 0 {
                        base_f * 1.25
                    } else {
                        base_f * 0.75
                    };
                    let amp = if k_trit == 0 { 0.025 } else { 0.065 * env };
                    let sig = amp * (tau * f * t).sin();
                    if i % 2 == 0 {
                        acc_l += sig;
                        acc_r += sig * 0.6;
                    } else {
                        acc_l += sig * 0.6;
                        acc_r += sig;
                    }
                }
                (
                    (acc_l as f32).clamp(-0.9, 0.9),
                    (acc_r as f32).clamp(-0.9, 0.9),
                )
            }
            StationSynthKind::MirrorInF => {
                const F_SERIES: [f64; 6] = [87.31, 130.81, 174.61, 220.00, 261.63, 349.23];
                let step_idx = (t / 0.45) as usize;
                let note_t = t % 0.45;
                let env = (-note_t * 3.2).exp();
                let f_lead = F_SERIES[step_idx % F_SERIES.len()];
                let f_mirror = F_SERIES[(F_SERIES.len() - 1) - (step_idx % F_SERIES.len())];
                let drone = 0.08 * (tau * 87.31 * t).sin() + 0.05 * (tau * 174.61 * t).sin();
                let bell_l = 0.12 * env * (tau * f_lead * t).sin();
                let bell_r = 0.12 * env * (tau * f_mirror * t).sin();
                (
                    ((drone + bell_l + bell_r * 0.4) as f32).clamp(-0.9, 0.9),
                    ((drone + bell_r + bell_l * 0.4) as f32).clamp(-0.9, 0.9),
                )
            }
        }
    }
}

impl Iterator for VakedStationSource {
    type Item = Sample;

    #[inline]
    fn next(&mut self) -> Option<Sample> {
        if self.channel == 0 {
            let (l, r) = self.synth_stereo_frame();
            self.last_right = r;
            self.channel = 1;
            Some(l)
        } else {
            self.channel = 0;
            Some(self.last_right)
        }
    }
}

impl Source for VakedStationSource {
    #[inline]
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    #[inline]
    fn channels(&self) -> ChannelCount {
        NonZero::new(SYNTH_CHANNELS).expect("non-zero channels")
    }

    #[inline]
    fn sample_rate(&self) -> SampleRate {
        NonZero::new(SYNTH_SAMPLE_RATE).expect("non-zero sample rate")
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

/// Open a `rodio::MixerDeviceSink` targeting a specific Bluetooth/audio speaker
/// substring (`speaker_query`) or the system default output device.
pub fn open_sink_for_speaker(
    speaker_query: Option<&str>,
) -> Result<(MixerDeviceSink, String), String> {
    let host = rodio::cpal::default_host();

    if let Some(query) = speaker_query {
        let q_lower = query.to_ascii_lowercase();
        let devices = host
            .output_devices()
            .map_err(|e| format!("failed to enumerate output devices: {e}"))?;

        let mut available = Vec::new();
        for dev in devices {
            let name = dev
                .description()
                .ok()
                .map(|d| d.name().to_string())
                .unwrap_or_else(|| "Unknown Device".to_string());
            if !available.contains(&name) {
                available.push(name.clone());
            }
            if name.to_ascii_lowercase().contains(&q_lower) {
                let mut sink = DeviceSinkBuilder::from_device(dev)
                    .and_then(|b| b.open_sink_or_fallback())
                    .map_err(|e| format!("failed to open audio sink on '{name}': {e}"))?;
                sink.log_on_drop(false);
                return Ok((sink, name));
            }
        }
        println!(
            "bluetooth note:  '{query}' is currently offline/asleep (active: {}) — routing to system default output",
            if available.is_empty() {
                "none".to_string()
            } else {
                available.join(", ")
            }
        );
    }

    // Default device path
    let default_name = host
        .default_output_device()
        .and_then(|d| d.description().ok().map(|desc| desc.name().to_string()))
        .unwrap_or_else(|| "default output".to_string());
    let mut sink = DeviceSinkBuilder::open_default_sink()
        .map_err(|e| format!("failed to open default audio sink: {e}"))?;
    sink.log_on_drop(false);
    Ok((sink, default_name))
}

/// Queue a single `FavouriteTrack` into `player` (prefers local 320 kbps MP3 / WAV
/// master when present and `!synth_only`, otherwise falls back to 432 Hz synthesis).
fn append_favourite_track(player: &Player, track: &FavouriteTrack, synth_only: bool) -> bool {
    if !synth_only
        && let Some(path) = track.resolve_local_path()
        && let Ok(file) = File::open(&path)
        && let Ok(Ok(decoder)) =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| Decoder::try_from(file)))
    {
        println!(
            "  queued [{:02}] {} {} — {} [320k master: {}]",
            track.index,
            track.emoji,
            track.title,
            track.artist,
            path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(track.local_master_file)
        );
        player.append(decoder);
        return true;
    }
    false
}

/// Play `target` (constellation station, favourite track, local audio file, or HTTP stream URL)
/// through the selected output device (`bt_speaker`) for `seconds` (`0` = until Ctrl-C).
pub fn play_target(target: &str, bt_speaker: Option<&str>, seconds: u64) -> Result<(), String> {
    play_target_with_options(target, bt_speaker, seconds, None, false)
}

/// Play `target` with optional `--track <favourite_query>` and `--synth` flag.
pub fn play_target_with_options(
    target: &str,
    bt_speaker: Option<&str>,
    seconds: u64,
    track_query: Option<&str>,
    synth_only: bool,
) -> Result<(), String> {
    if let Some(q) = bt_speaker {
        match bt::match_speaker(q) {
            Some(p) => println!("speaker profile: {} — {}", p.model, p.note),
            None => println!("speaker filter: {q} (custom device match)"),
        }
    }

    let (sink, device_name) = open_sink_for_speaker(bt_speaker)?;
    let cfg = sink.config();
    println!(
        "audio sink:      {} ({} Hz · {}ch · {:?})",
        device_name,
        cfg.sample_rate().get(),
        cfg.channel_count().get(),
        cfg.sample_format()
    );

    let player = Player::connect_new(sink.mixer());

    // If `--track <query>` was specified, resolve the favourite track first
    let explicit_fav = match track_query {
        Some(q) => Some(favourites::find_favourite(q).ok_or_else(|| {
            format!(
                "unknown favourite track '{q}' on radio.vaked.dev (run `quantternradio favourites` to list all 14 tracks)"
            )
        })?),
        None => None,
    };

    if let Some(fav) = explicit_fav {
        println!(
            "tuned:           radio.vaked.dev/{} — {} {} ({})",
            fav.slug, fav.emoji, fav.title, fav.artist
        );
        println!(
            "harmony:         {} · {} BPM · {} Hz · {}",
            fav.key_progression, fav.bpm, fav.tuning_hz, fav.trit_read
        );
        println!("{URL_STREAM_STATUS}");
        let _ = append_favourite_track(&player, fav, synth_only);
        player.append(VakedStationSource::new(StationSynthKind::Favourite(
            fav.index - 1,
        )));
    } else if let Some(kind) = StationSynthKind::from_target(target) {
        println!("tuned:           {target} — {}", kind.description());
        println!("{URL_STREAM_STATUS}");
        match kind {
            StationSynthKind::RadioQuantshuffle => {
                println!(
                    "crate:           {} Favourite Musics loaded on radio.vaked.dev",
                    FAVOURITE_MUSICS.len()
                );
                let mut masters_queued = 0usize;
                for fav in &FAVOURITE_MUSICS {
                    if append_favourite_track(&player, fav, synth_only) {
                        masters_queued += 1;
                    }
                }
                if masters_queued == 0 {
                    println!(
                        "  mode: 432 Hz Verdi + 528 Hz Star procedural rotation (all 14 tracks)"
                    );
                }
                // Always append continuous procedural radio rotation so the station never ends
                player.append(VakedStationSource::new(StationSynthKind::RadioQuantshuffle));
            }
            StationSynthKind::Favourite(idx) => {
                let fav = &FAVOURITE_MUSICS[idx % FAVOURITE_MUSICS.len()];
                println!(
                    "harmony:         {} · {} BPM · {} Hz · {}",
                    fav.key_progression, fav.bpm, fav.tuning_hz, fav.trit_read
                );
                let _ = append_favourite_track(&player, fav, synth_only);
                player.append(VakedStationSource::new(kind));
            }
            _ => {
                player.append(VakedStationSource::new(kind));
            }
        }
    } else if Path::new(target).is_file() {
        let file = File::open(target).map_err(|e| format!("cannot open {target}: {e}"))?;
        let decoder =
            Decoder::try_from(file).map_err(|e| format!("cannot decode {target}: {e}"))?;
        println!("tuned file:      {target}");
        println!("{URL_STREAM_STATUS}");
        player.append(decoder);
    } else if target.starts_with("http://") || target.starts_with("https://") {
        let max_time = if seconds > 0 { seconds.max(6) } else { 15 };
        let output = Command::new("curl")
            .args([
                "-sL",
                "--fail",
                "--max-time",
                &max_time.to_string(),
                "--max-filesize",
                "8388608",
                target,
            ])
            .output()
            .map_err(|e| format!("cannot start stream download with curl: {e}"))?;
        let decoder = decode_stream_window(output.status.code(), output.stdout)?;
        println!("tuned stream:    {target}");
        println!("{URL_STREAM_STATUS}");
        player.append(decoder);
    } else {
        return Err(format!(
            "unknown station or unreadable target: {target} (try radio.vaked.dev, innerbloom, mahakala, faded, everything-i-wanted, music.vaked.dev, nand.vaked.dev, or an http(s) URL)"
        ));
    }

    let start = Instant::now();
    let mut ring = nand::INITIAL_RING;
    let mut kleene = nand::INITIAL_KLEENE;
    let mut step = 0u64;

    println!("on air:          {{−1, 0, +1}} live equalizer (Ctrl-C to stop)");
    loop {
        let elapsed = start.elapsed().as_secs_f64();
        let _demo = visuals::demo_frame(elapsed);
        let st = nand::step_ring(ring, kleene, step);
        ring = st.ring;
        kleene = st.kleene;
        step += 1;

        let active_fav = if let Some(fav) = explicit_fav {
            fav
        } else if let Some(StationSynthKind::Favourite(idx)) = StationSynthKind::from_target(target)
        {
            &FAVOURITE_MUSICS[idx % FAVOURITE_MUSICS.len()]
        } else {
            &FAVOURITE_MUSICS[((elapsed / 16.0) as usize) % FAVOURITE_MUSICS.len()]
        };

        print!(
            "\r  t={elapsed:5.1}s · {} {:18} · trits {} · nand {} · 0x{:016x}   ",
            active_fav.emoji,
            active_fav.slug,
            nand::format_glyphs(&st.frame.trits),
            nand::format_glyphs(&st.ring),
            st.frame.hash()
        );
        let _ = io::stdout().flush();

        if seconds > 0 && elapsed >= seconds as f64 {
            println!();
            break;
        }
        if player.empty() {
            println!();
            break;
        }
        std::thread::sleep(Duration::from_millis(250));
    }

    player.stop();
    Ok(())
}

/// Decode a bounded HTTP capture without substituting unrelated station audio.
fn decode_stream_window(
    exit_code: Option<i32>,
    bytes: Vec<u8>,
) -> Result<Decoder<Cursor<Vec<u8>>>, String> {
    match exit_code {
        // A live stream may reach our time or size limit after yielding playable data.
        Some(0 | 28 | 63) => {}
        Some(code) => return Err(format!("stream download failed (curl exit {code})")),
        None => return Err("stream download was interrupted".to_owned()),
    }
    if bytes.is_empty() {
        return Err("stream download returned no audio data".to_owned());
    }
    Decoder::new(Cursor::new(bytes)).map_err(|e| format!("cannot decode stream audio: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wav_fixture() -> Vec<u8> {
        // Mono 16-bit PCM, eight frames. Decode only: no output device is opened.
        let samples: [i16; 8] = [0, 1000, -1000, 2000, -2000, 1000, -1000, 0];
        let data_size = (samples.len() * 2) as u32;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(36 + data_size).to_le_bytes());
        bytes.extend_from_slice(b"WAVEfmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes()); // PCM
        bytes.extend_from_slice(&1u16.to_le_bytes()); // mono
        bytes.extend_from_slice(&8000u32.to_le_bytes());
        bytes.extend_from_slice(&16000u32.to_le_bytes()); // bytes per second
        bytes.extend_from_slice(&2u16.to_le_bytes()); // block alignment
        bytes.extend_from_slice(&16u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&data_size.to_le_bytes());
        for sample in samples {
            bytes.extend_from_slice(&sample.to_le_bytes());
        }
        bytes
    }

    #[test]
    fn stream_capture_decodes_audio_after_success_or_capture_limit() {
        for code in [0, 28, 63] {
            let decoder = decode_stream_window(Some(code), wav_fixture()).expect("valid capture");
            assert_eq!(decoder.channels().get(), 1);
            assert_eq!(decoder.sample_rate().get(), 8000);
            let samples: Vec<_> = decoder.collect();
            assert_eq!(samples.len(), 8);
            assert!(samples[1] > 0.0);
            assert!(samples[2] < 0.0);
        }
    }

    #[test]
    fn stream_download_failure_is_an_error_even_with_audio_in_response() {
        for code in [6, 7, 22, 60] {
            let error = decode_stream_window(Some(code), wav_fixture())
                .err()
                .unwrap();
            assert!(error.contains(&format!("curl exit {code}")));
        }
        assert!(decode_stream_window(None, wav_fixture()).is_err());
    }

    #[test]
    fn empty_or_invalid_stream_capture_is_an_error() {
        for code in [0, 28, 63] {
            let error = decode_stream_window(Some(code), Vec::new()).err().unwrap();
            assert!(error.contains("no audio data"));
            let error = decode_stream_window(Some(code), b"<html>Unavailable</html>".to_vec())
                .err()
                .unwrap();
            assert!(error.contains("cannot decode stream audio"));
        }
    }

    #[test]
    fn missing_file_is_an_error_not_a_panic() {
        assert!(file_readable("/definitely/not/a/file.mp3").is_err());
    }

    #[test]
    fn this_source_file_is_readable() {
        let n = file_readable(file!()).expect("source file readable");
        assert!(n > 0);
    }

    #[test]
    fn station_synth_kind_recognizes_vaked_surfaces_and_favourites() {
        assert_eq!(
            StationSynthKind::from_target("radio.vaked.dev"),
            Some(StationSynthKind::RadioQuantshuffle)
        );
        assert_eq!(
            StationSynthKind::from_target("radio.vaked.dev/innerbloom"),
            Some(StationSynthKind::Favourite(1))
        );
        assert_eq!(
            StationSynthKind::from_target("mahakala"),
            Some(StationSynthKind::Favourite(0))
        );
        assert_eq!(
            StationSynthKind::from_target("faded"),
            Some(StationSynthKind::Favourite(2))
        );
        assert_eq!(
            StationSynthKind::from_target("https://music.vaked.dev/"),
            Some(StationSynthKind::Music432)
        );
        assert_eq!(
            StationSynthKind::from_target("nand.vaked.dev"),
            Some(StationSynthKind::NandOscillator)
        );
        assert_eq!(
            StationSynthKind::from_target("mirror.vaked.dev"),
            Some(StationSynthKind::MirrorInF)
        );
        assert_eq!(StationSynthKind::from_target("unknown.example"), None);
    }

    #[test]
    fn vaked_station_source_generates_bounded_stereo_samples_across_all_favourites() {
        for idx in 0..FAVOURITE_MUSICS.len() {
            let mut src = VakedStationSource::new(StationSynthKind::Favourite(idx));
            assert_eq!(src.channels().get(), 2);
            assert_eq!(src.sample_rate().get(), 48_000);
            let mut non_zero = 0usize;
            for _ in 0..480 {
                let s = src.next().expect("infinite source");
                assert!((-1.0..=1.0).contains(&s), "sample out of bounds: {s}");
                if s.abs() > 1e-5 {
                    non_zero += 1;
                }
            }
            assert!(
                non_zero > 50,
                "favourite track {idx} must emit audible samples"
            );
        }
    }
}
