//! cli_e2e — the quantTernRadio CLI, end to end.
//!
//! Runs the real binary (no TTY, no devices, no network) across the
//! honest lanes: the dial + bluetooth backend, the menu snapshot, and
//! the quantTernAscii one-shot render. The golden hash is pinned so any
//! change to the quantizer is a visible event.

use std::process::{Command, Output};

fn zoneinn(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_zoneinn"))
        .args(args)
        .output()
        .expect("zoneinn runs")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

#[test]
fn list_shows_the_dial_and_the_bluetooth_backend() {
    let out = zoneinn(&["list"]);
    assert!(out.status.success(), "list failed: {}", stdout(&out));
    let s = stdout(&out);
    assert!(s.contains("radio.vaked.dev"));
    assert!(s.contains("music.vaked.dev"));
    assert!(s.contains("bluetooth backend:"));
    assert!(s.contains("Soundcore Select 4 GO first-class"));
}

#[test]
fn bt_lane_names_backend_and_first_class_speaker() {
    let out = zoneinn(&["bt"]);
    assert!(out.status.success(), "bt failed: {}", stdout(&out));
    let s = stdout(&out);
    assert!(s.contains("bluetooth backend:"));
    assert!(s.contains("known speakers:"));
    assert!(s.contains("Soundcore Select 4 GO"));
    assert!(s.contains("first-class"));
}

#[test]
fn menu_snapshot_lists_the_whole_dial() {
    let out = zoneinn(&["menu", "--snapshot"]);
    assert!(out.status.success(), "menu failed: {}", stdout(&out));
    let s = stdout(&out);
    assert!(s.contains("quantTernRadio · the dial menu"));
    assert!(s.contains("[1] radio.vaked.dev"));
    assert!(s.contains("[2] music.vaked.dev"));
    assert!(s.contains("[3] play <url>"));
    assert!(s.contains("bluetooth backend:"));
}

#[test]
fn quantternascii_renders_the_pinned_frame() {
    let out = zoneinn(&["quantTernAscii"]);
    assert!(
        out.status.success(),
        "quantTernAscii failed: {}",
        stdout(&out)
    );
    let s = stdout(&out);
    assert!(s.contains("quantTernRadio · zoneInn · {−1, 0, +1}"));
    assert!(s.contains("station: radio.vaked.dev"));
    assert!(s.contains("bands"));
    assert!(s.contains("trits"));
    assert!(s.contains("dominant"));
    // Pinned at the first one-shot render (t = 0.0, default station):
    // the demo frame's FNV-1a golden. A quantizer change is a visible event.
    assert!(s.contains("frame golden 0x7311bcbd581290fd"));
}

#[test]
fn quantternascii_is_deterministic_and_flags_apply() {
    let a = zoneinn(&[
        "quantTernAscii",
        "--station",
        "music.vaked.dev",
        "--t",
        "10",
    ]);
    let b = zoneinn(&[
        "quantTernAscii",
        "--station",
        "music.vaked.dev",
        "--t",
        "10",
    ]);
    assert!(a.status.success() && b.status.success());
    let sa = stdout(&a);
    assert_eq!(sa, stdout(&b), "same frame must render identically");
    assert!(sa.contains("station: music.vaked.dev"));
    assert!(sa.contains("frame at t = 10.0s"));
}

#[test]
fn unknown_subcommands_are_rejected_not_ignored() {
    // The e2e pin: clap must never silently swallow a lane.
    let out = zoneinn(&["no-such-lane"]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("unrecognized subcommand"));
}

#[test]
fn web_dump_status_exposes_dial_bluetooth_and_nand_repo() {
    let out = zoneinn(&["web", "--dump-status"]);
    assert!(
        out.status.success(),
        "web --dump-status failed: {}",
        stdout(&out)
    );
    let s = stdout(&out);
    assert!(s.contains("\"web_url\":\"https://zoneinn.vaked.dev\""));
    assert!(s.contains("Soundcore Select 4 GO"));
    assert!(s.contains("PeetPedro/ultrawhale-dogfood"));
}

#[test]
fn nand_oscillator_renders_steps_and_jsonl_refeed() {
    let out = zoneinn(&["nand", "--steps", "4"]);
    assert!(out.status.success(), "nand failed: {}", stdout(&out));
    let s = stdout(&out);
    assert!(s.contains("ultrawhale NAND ring oscillator"));
    assert!(s.contains("PeetPedro/ultrawhale-dogfood"));
    assert!(s.contains("step 00"));
    assert!(s.contains("golden 0x"));

    let jl = zoneinn(&["nand", "--steps", "3", "--jsonl"]);
    assert!(jl.status.success(), "nand --jsonl failed: {}", stdout(&jl));
    let jl_out = stdout(&jl);
    let lines: Vec<&str> = jl_out.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].contains("\"dim\":\"nand_oscillator\""));
    assert!(lines[0].contains("\"repo\":\"PeetPedro/ultrawhale-dogfood\""));
}

#[test]
fn favourites_crate_lists_all_20_tracks() {
    let out = zoneinn(&["favourites"]);
    assert!(out.status.success(), "favourites failed: {}", stdout(&out));
    let s = stdout(&out);
    assert!(s.contains("radio.vaked.dev · Favourite Musics Crate"));
    assert!(s.contains("mahakala"));
    assert!(s.contains("innerbloom"));
    assert!(s.contains("faded"));
    assert!(s.contains("everything-i-wanted"));
    assert!(s.contains("anvil"));
    assert!(s.contains("teardrop"));
    assert!(s.contains("hunger-of-the-pine"));
    assert!(s.contains("left-hand-free"));
    assert!(s.contains("glue"));
    assert!(s.contains("oblivion"));
    assert!(s.contains("not-like-before"));
}
