# zoneInn — quantTernRadio

Landing: [zoneinn.vaked.dev](https://zoneinn.vaked.dev) · palette `ultra-omnipotent-light` (design.vaked.dev Palette F).

**The vaked dial, in a terminal.** Pure Rust CLI radio: tuneIn-style URLs,
the constellation's own radio surfaces, a ternary `{−1, 0, +1}` equalizer,
safe bluetooth, and the SUNO OSS voice between tracks.

> Thanks Yves & Dra <3 — for the room, and for holding the door.

```
quantTernRadio · zoneInn · {−1, 0, +1}
  stations    radio.vaked.dev · music.vaked.dev · play <any URL (tuneIn)>
  visualizer  live ternary equalizer (rustfft → ±1/0 bands → ratatui)
  bluetooth   safe (pure Rust) · Soundcore Select 4 GO first-class
  voice       SUNO OSS (Bark) between tracks via bark-selfhost
  agy         antigravity ultra lane — ASCII art for the now-playing track
```

## Quick start

```bash
cargo run -- list                 # the dial + 14 Favourite Musics on radio.vaked.dev
cargo run -- favourites           # inspect the 14 Favourite Musics crate (320k masters + 432Hz synth)
cargo run -- demo --seconds 30    # the ternary equalizer, offline, no device
cargo run -- play radio.vaked.dev --bt "Soundcore Select 4"
cargo run -- play innerbloom --bt "Soundcore Select 4"
cargo run -- play radio.vaked.dev --track mahakala --bt "Soundcore Select 4"
cargo run -- voice "you are listening to radio.vaked.dev"
cargo run -- menu                 # the interactive dial menu (↑↓, enter, q)
cargo run -- menu --snapshot      # the menu as text — E2E, no TTY needed
cargo run -- quantTernAscii       # one-shot ASCII equalizer, golden pinned
cargo run -- bt                   # the safe bluetooth lane
cargo run -- web                  # serve the live Web Radio & Bluetooth Studio (http://127.0.0.1:8088)
cargo run -- nand --steps 8       # step the Ultrawhale ternary NAND ring oscillator
cargo run -- nand --jsonl         # emit JSONL refeed rows for PeetPedro/ultrawhale-dogfood
```

## The ternary equalizer

Every spectrum frame collapses to eight bands and quantizes to
`{−1, 0, +1}` — the constellation's affect alphabet. A band reads `+`
(rising), `·` (rest), or `−` (falling); inside the threshold it is a
genuine zero, never guessed. The frame's FNV-1a hash is pinned in
`src/tern.rs` (`0xe0efe6a3385d3dd3` for the canonical frame) so any
change to the quantizer is a visible event.

## Web Radio & Bluetooth Studio (`zoneinn.vaked.dev` / `cargo run -- web`)

`zoneInn` ships an embedded single-file Web Radio & Bluetooth Studio (`web/index.html`,
also live at [zoneinn.vaked.dev](https://zoneinn.vaked.dev)) that plays the constellation's
radio stations in the browser (`radio.vaked.dev` 1-bit Quantshuffle + Beethoven/Ode to Joy/Für Elise/Pachelbel canons, `music.vaked.dev` 432 Hz Verdi tuning, `nand.vaked.dev` 5-stage cross-coupled ternary NAND ring oscillator, `mirror.vaked.dev` Mirror in F, plus any live HTTP/TuneIn/Icecast stream URL) and connects to Bluetooth speakers via:
- **Web Bluetooth (`navigator.bluetooth.requestDevice`)** with first-class **Soundcore Select 4 GO** recognition and BLE GATT battery readout (`0x180F`).
- **Audio Output Routing (`selectAudioOutput` / `setSinkId`)** to route live Web Audio directly to paired Bluetooth A2DP speakers.
- **AVRCP MediaSession (`navigator.mediaSession`)** for physical speaker play/pause/prev/next buttons.

## Safe bluetooth

"Safe" means two things, claimed exactly:

1. **Memory-safe by construction** — no `unsafe` FFI in this crate. The
   Linux backend (BlueZ via the pure-Rust `bluer` crate) is the first
   wire; macOS/Windows are honest stubs until their bridges land, while
   `zoneinn web` / `zoneinn.vaked.dev` provides live Web Bluetooth +
   `setSinkId` routing in the browser today.
2. **Soundcore Select 4 GO first-class** — the speaker ships its own
   profile (`src/bt.rs`), recognized by name before any other device.

## Ultrawhale NAND Ring Oscillator (`src/nand.rs`)

`cargo run -- nand` runs a 5-stage cross-coupled ternary NAND ring oscillator
(`{−1, 0, +1}` + strong-3 Kleene `¬min(a, b)`), quantizes each step into an
8-band `TernFrame` with a 64-bit FNV-1a golden hash, and emits JSONL refeed rows
for [`PeetPedro/ultrawhale-dogfood`](https://huggingface.co/datasets/PeetPedro/ultrawhale-dogfood) (`--jsonl`).

## The voice between tracks

The SUNO OSS lineage is Bark (suno-ai/bark, MIT), self-hosted by the
constellation at `bark-selfhost`. Configure `BARK_SELFHOST_URL` and the
HTTP bridge lands in the next slice; without it the cue is returned
unspoken — reported, never fabricated.

## The agy lane

`agy` (the antigravity CLI, ultra subscription) renders the ASCII
now-playing card — the foobar2000-style visual beyond the equalizer.
It shells out when `agy` is on `PATH` (or `AGY_BIN` is set); otherwise
the radio runs on the ternary equalizer alone.

## Honest status

| lane | status |
|---|---|
| ternary equalizer demo | ✅ live, no device needed |
| dial + station presets | ✅ live (radio.vaked.dev / music.vaked.dev) |
| dial menu | ✅ live (interactive + `--snapshot` text form) |
| quantTernAscii one-shot | ✅ live, deterministic, golden pinned |
| web radio & bluetooth studio | ✅ live (`zoneinn.vaked.dev` + `cargo run -- web`) |
| ultrawhale NAND oscillator | ✅ live (`cargo run -- nand` + HF `PeetPedro` refeed) |
| cli e2e | ✅ live (`tests/cli_e2e.rs`, real binary) |
| stream playback | ✅ live (`rodio 0.23` `MixerDeviceSink` + `Player` + `VakedStationSource` / Symphonia decoder + Web Studio) |
| bluetooth backend | ✅ live (`cpal` Bluetooth A2DP sink routing via `--bt "Soundcore Select 4"` + Web Bluetooth / `setSinkId`) |
| SUNO OSS voice bridge | ✅ Web Speech voice cues live in Web Studio; CLI `bark-selfhost` wire next |
| agy visuals | ✅ shell-out lane live |

*the constellation · 0 + 1 · fine touch from within · vaked.dev*
