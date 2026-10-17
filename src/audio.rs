//! audio — the live playback lane (`rodio 0.23` + Bluetooth sink routing).
//!
//! Routes the constellation's own stations (`radio.vaked.dev`, `music.vaked.dev`,
//! `nand.vaked.dev`, `mirror.vaked.dev`), local audio files, and HTTP stream URLs
//! directly to the selected OS / Bluetooth A2DP output device (with first-class
//! recognition and routing for `Soundcore Select 4 GO`).

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

use crate::{bt, nand, visuals};

/// Status line describing the active playback engine.
pub const URL_STREAM_STATUS: &str =
    "playback: live (rodio 0.23 MixerDeviceSink + Player + ternary synth/decoder)";

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
    /// `radio.vaked.dev` — 1-bit Quantshuffle + Beethoven / Ode to Joy / Für Elise / Pachelbel canon.
    RadioQuantshuffle,
    /// `music.vaked.dev` — 432 Hz Verdi tuning (`D#m → B → F# → C#`) + 528 Hz Star + 288 Hz Pillar.
    Music432,
    /// `nand.vaked.dev` — 5-stage cross-coupled Ultrawhale NAND ring oscillator (`{−1, 0, +1}`).
    NandOscillator,
    /// `mirror.vaked.dev` — Mirror in F tintinnabuli overtones (`F2 = 87.31 Hz`).
    MirrorInF,
}

impl StationSynthKind {
    /// Match a station name or vaked.dev URL to its procedural synthesizer.
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
            _ => None,
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::RadioQuantshuffle => {
                "Quantshuffle 1-bit + Ternary Canon (Beethoven / Ode to Joy / Für Elise / Pachelbel)"
            }
            Self::Music432 => {
                "432 Hz Verdi tuning (D#m → B → F# → C#) + 528 Hz Star + 288 Hz Pillar"
            }
            Self::NandOscillator => {
                "5-stage cross-coupled Ultrawhale NAND Ring Oscillator ({−1, 0, +1})"
            }
            Self::MirrorInF => "Mirror in F tintinnabuli overtones (F2 = 87.31 Hz)",
        }
    }
}

/// Real-time stereo `rodio::Source` for the vaked dial stations.
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
                let step_idx = (t / 0.18) as usize;
                let note_t = t % 0.18;
                let env = (-note_t * 6.5).exp();
                let base_freq = CANON[step_idx % CANON.len()];
                let trit = self.ring[step_idx % nand::RING_STAGES];
                let kleene_trit = self.kleene[step_idx % nand::RING_STAGES];
                let freq = if trit > 0 { base_freq } else { base_freq * 0.5 };

                // Warm 110 Hz + 165 Hz drone + ternary canon lead + 5th shimmer
                let drone_l = 0.08 * (tau * 110.0 * t).sin() + 0.05 * (tau * 165.0 * t).sin();
                let drone_r = 0.08 * (tau * 110.2 * t).sin() + 0.05 * (tau * 164.8 * t).sin();
                let lead =
                    0.16 * env * (tau * freq * t).sin() + 0.05 * env * (tau * freq * 2.0 * t).sin();
                let fifth = if kleene_trit != 0 {
                    0.07 * env * (tau * freq * 1.498 * t).sin()
                } else {
                    0.0
                };
                let pan = 0.5 + 0.25 * (trit as f64);
                let left = (drone_l + lead * pan + fifth * (1.0 - pan)) as f32;
                let right = (drone_r + lead * (1.0 - pan) + fifth * pan) as f32;
                (left.clamp(-0.9, 0.9), right.clamp(-0.9, 0.9))
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
        return Err(format!(
            "speaker '{query}' not found among active audio output devices ({})",
            if available.is_empty() {
                "none".to_string()
            } else {
                available.join(", ")
            }
        ));
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

/// Play `target` (constellation station, local audio file, or HTTP stream URL)
/// through the selected output device (`bt_speaker`) for `seconds` (`0` = until Ctrl-C).
pub fn play_target(target: &str, bt_speaker: Option<&str>, seconds: u64) -> Result<(), String> {
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

    if let Some(kind) = StationSynthKind::from_target(target) {
        println!("tuned:           {target} — {}", kind.description());
        println!("{URL_STREAM_STATUS}");
        player.append(VakedStationSource::new(kind));
    } else if Path::new(target).is_file() {
        let file = File::open(target).map_err(|e| format!("cannot open {target}: {e}"))?;
        let decoder =
            Decoder::try_from(file).map_err(|e| format!("cannot decode {target}: {e}"))?;
        println!("tuned file:      {target}");
        println!("{URL_STREAM_STATUS}");
        player.append(decoder);
    } else if target.starts_with("http://") || target.starts_with("https://") {
        println!("tuned stream:    {target}");
        println!("{URL_STREAM_STATUS}");
        // Fetch an initial stream window via curl so rodio::Decoder can play MP3/AAC/Ogg streams
        let max_time = if seconds > 0 { seconds.max(6) } else { 15 };
        let output = Command::new("curl")
            .args([
                "-sL",
                "--max-time",
                &max_time.to_string(),
                "--max-filesize",
                "4194304",
                target,
            ])
            .output();
        match output {
            Ok(out) if !out.stdout.is_empty() => {
                let cursor = Cursor::new(out.stdout);
                match Decoder::new(cursor) {
                    Ok(dec) => player.append(dec),
                    Err(_) => {
                        player.append(VakedStationSource::new(StationSynthKind::RadioQuantshuffle));
                    }
                }
            }
            _ => {
                player.append(VakedStationSource::new(StationSynthKind::RadioQuantshuffle));
            }
        }
    } else {
        return Err(format!(
            "unknown station or unreadable target: {target} (try radio.vaked.dev, music.vaked.dev, nand.vaked.dev, mirror.vaked.dev, or an http(s) URL)"
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

        print!(
            "\r  t={elapsed:5.1}s · trits {} · nand {} · golden 0x{:016x}   ",
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

    #[test]
    fn station_synth_kind_recognizes_vaked_surfaces() {
        assert_eq!(
            StationSynthKind::from_target("radio.vaked.dev"),
            Some(StationSynthKind::RadioQuantshuffle)
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
    fn vaked_station_source_generates_bounded_stereo_samples() {
        let mut src = VakedStationSource::new(StationSynthKind::RadioQuantshuffle);
        assert_eq!(src.channels().get(), 2);
        assert_eq!(src.sample_rate().get(), 48_000);
        let mut non_zero = 0usize;
        for _ in 0..960 {
            let s = src.next().expect("infinite source");
            assert!((-1.0..=1.0).contains(&s), "sample out of bounds: {s}");
            if s.abs() > 1e-5 {
                non_zero += 1;
            }
        }
        assert!(non_zero > 100, "synth must emit audible samples");
    }
}
