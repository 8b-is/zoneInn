//! quantTernRadio — the zoneInn radio CLI.

use clap::{Parser, Subcommand};
use zoneinn::{agy, ascii, audio, bt, favourites, menu, nand, stations, visuals, voice, web};

#[derive(Parser)]
#[command(
    name = "quantternradio",
    version,
    about = "the zoneInn radio — {−1, 0, +1} on air"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// List the stations on the dial and the Favourite Musics crate.
    List,
    /// List the 20 Favourite Musics on radio.vaked.dev.
    #[command(visible_aliases = ["favs", "tracks"])]
    Favourites,
    /// Run the offline demo — the ternary equalizer, no audio device.
    Demo {
        /// Station name for the header (default: radio.vaked.dev).
        #[arg(long, default_value = "radio.vaked.dev")]
        station: String,
        /// Stop after N seconds (0 = until q).
        #[arg(long, default_value_t = 0)]
        seconds: u64,
    },
    /// Tune a station by name, favourite track slug, or a raw stream URL.
    Play {
        /// Station name, favourite track slug (e.g. innerbloom, mahakala), or http(s) URL.
        target: String,
        /// Bluetooth speaker to send to (name substring, e.g. "Soundcore Select 4").
        #[arg(long)]
        bt: Option<String>,
        /// Select a specific track from the radio.vaked.dev Favourite Musics crate (1..20 or slug).
        #[arg(long)]
        track: Option<String>,
        /// Force pure 432 Hz / 528 Hz procedural synthesis instead of local 320k masters.
        #[arg(long)]
        synth: bool,
        /// Stop after N seconds (0 = play continuously until Ctrl-C).
        #[arg(long, default_value_t = 0)]
        seconds: u64,
    },
    /// Speak a line through the SUNO OSS voice lane.
    Voice {
        /// The line to speak.
        text: Vec<String>,
    },
    /// The interactive dial menu — arrow keys, enter to tune, q to quit.
    Menu {
        /// Render one text snapshot and exit (no TTY needed).
        #[arg(long)]
        snapshot: bool,
    },
    /// One-shot ASCII render of the ternary equalizer.
    #[command(name = "quantTernAscii")]
    QuantTernAscii {
        /// Station name for the header (default: radio.vaked.dev).
        #[arg(long, default_value = "radio.vaked.dev")]
        station: String,
        /// Frame time in seconds (default: 0.0).
        #[arg(long, default_value_t = 0.0)]
        t: f64,
    },
    /// The safe bluetooth lane — backend and known speakers.
    Bt,
    /// Serve the interactive Web Radio & Bluetooth Studio (zoneinn.vaked.dev).
    Web {
        /// Local bind address (default: 127.0.0.1:8088).
        #[arg(long, default_value = web::DEFAULT_BIND)]
        bind: String,
        /// Print the JSON status payload and exit without binding a socket.
        #[arg(long)]
        dump_status: bool,
    },
    /// Step the Ultrawhale ternary NAND ring oscillator or emit HF refeed JSONL.
    Nand {
        /// Number of oscillator steps to run (default: 8).
        #[arg(long, default_value_t = 8)]
        steps: usize,
        /// Emit JSONL refeed rows for PeetPedro/ultrawhale-dogfood.
        #[arg(long)]
        jsonl: bool,
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
                "favourite musics on radio.vaked.dev ({} tracks):",
                favourites::FAVOURITE_MUSICS.len()
            );
            for t in &favourites::FAVOURITE_MUSICS {
                println!(
                    "  [{:02}] {} {:22} · {} — {} ({} BPM · {} Hz)",
                    t.index, t.emoji, t.slug, t.title, t.artist, t.bpm, t.tuning_hz
                );
            }
            println!();
            println!(
                "bluetooth backend: {:?} · Soundcore Select 4 GO first-class",
                bt::platform()
            );
            println!("thanks Yves & Dra <3");
        }
        Cmd::Favourites => {
            print!("{}", favourites::describe_crate());
        }
        Cmd::Demo { station, seconds } => {
            if let Err(e) = visuals::run_demo(&station, seconds) {
                eprintln!("demo failed: {e}");
                std::process::exit(1);
            }
        }
        Cmd::Play {
            target,
            bt,
            track,
            synth,
            seconds,
        } => {
            if std::env::var("ZONEINN_AGY").is_ok()
                && let Some(art) = agy::track_art("8b-is", &target)
            {
                println!("{art}");
            }
            if let Err(e) = audio::play_target_with_options(
                &target,
                bt.as_deref(),
                seconds,
                track.as_deref(),
                synth,
            ) {
                eprintln!("playback failed: {e}");
                std::process::exit(1);
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
        Cmd::Menu { snapshot } => {
            if snapshot {
                print!("{}", menu::snapshot());
            } else if let Err(e) = menu::run_interactive() {
                eprintln!("menu failed: {e}");
                std::process::exit(1);
            }
        }
        Cmd::QuantTernAscii { station, t } => {
            let frame = visuals::demo_frame(t);
            print!("{}", ascii::render(&frame, &station, t));
        }
        Cmd::Bt => {
            print!("{}", bt::describe());
        }
        Cmd::Web { bind, dump_status } => {
            if dump_status {
                println!("{}", web::status_json());
            } else if let Err(e) = web::serve(&bind, None) {
                eprintln!("web studio failed on {bind}: {e}");
                std::process::exit(1);
            }
        }
        Cmd::Nand { steps, jsonl } => {
            if jsonl {
                print!(
                    "{}",
                    nand::generate_refeed_jsonl(steps, "2026-10-18T00:00:00Z")
                );
            } else {
                print!("{}", nand::describe_run(steps));
            }
        }
    }
}
