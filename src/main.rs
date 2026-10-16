//! quantTernRadio — the zoneInn radio CLI.

use clap::{Parser, Subcommand};
use zoneinn::{agy, audio, bt, stations, visuals, voice};

#[derive(Parser)]
#[command(name = "quantternradio", version, about = "the zoneInn radio — {−1, 0, +1} on air")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// List the stations on the dial.
    List,
    /// Run the offline demo — the ternary equalizer, no audio device.
    Demo {
        /// Station name for the header (default: radio.vaked.dev).
        #[arg(long, default_value = "radio.vaked.dev")]
        station: String,
        /// Stop after N seconds (0 = until q).
        #[arg(long, default_value_t = 0)]
        seconds: u64,
    },
    /// Tune a station by name or a raw stream URL.
    Play {
        /// Station name or http(s) URL.
        target: String,
        /// Bluetooth speaker to send to (name substring).
        #[arg(long)]
        bt: Option<String>,
    },
    /// Speak a line through the SUNO OSS voice lane.
    Voice {
        /// The line to speak.
        text: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::List => {
            println!("quantTernRadio · the vaked dial");
            for s in &stations::VAKED_DIAL {
                let url = s.stream_url.unwrap_or("stream not wired yet");
                println!("  {:24} {} · {}", s.name, url, s.note);
            }
            println!("  {:24} any URL (tuneIn included)", "play <url>");
            println!();
            println!(
                "bluetooth backend: {:?} · Soundcore Select 4 GO first-class",
                bt::platform()
            );
            println!("thanks Yves & Dra <3");
        }
        Cmd::Demo { station, seconds } => {
            if let Err(e) = visuals::run_demo(&station, seconds) {
                eprintln!("demo failed: {e}");
                std::process::exit(1);
            }
        }
        Cmd::Play { target, bt } => {
            let url = if target.starts_with("http") {
                Some(target.clone())
            } else {
                stations::find(&target).and_then(|s| s.stream_url.map(str::to_string))
            };
            match url {
                Some(_u) => {
                    println!("tuned: {target}");
                    println!("{}", audio::URL_STREAM_STATUS);
                }
                None => {
                    println!("station found on the dial, stream not wired yet: {target}");
                    println!("{}", audio::URL_STREAM_STATUS);
                }
            }
            if let Some(art) = agy::track_art("unknown", &target) {
                println!();
                println!("{art}");
            }
            if let Some(speaker) = bt {
                match bt::match_speaker(&speaker) {
                    Some(p) => println!("speaker: {} — {}", p.model, p.note),
                    None => println!("speaker not recognized: {speaker}"),
                }
            }
        }
        Cmd::Voice { text } => {
            let cue = voice::VoiceCue::new(text.join(" "));
            match voice::speak(&cue) {
                Ok(true) => println!("spoken"),
                Ok(false) => println!("unspoken (no bark endpoint): {}", cue.text),
                Err(e) => println!("{e}"),
            }
        }
    }
}
