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
cargo run -- list                 # the dial + 20 Favourite Musics on radio.vaked.dev
cargo run -- favourites           # inspect the 20 Favourite Musics crate (family+1 & #1 Muse · 320k masters + 432Hz synth)
cargo run -- demo --seconds 30    # the ternary equalizer, offline, no device
cargo run -- play radio.vaked.dev --bt "Soundcore Select 4"
cargo run -- play innerbloom --bt "Soundcore Select 4"
cargo run -- play left-hand-free --bt "Soundcore Select 4"
cargo run -- play radio.vaked.dev --track teardrop --bt "Soundcore Select 4"
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

## The 20 Favourite Musics Crate (`family+1` & `#1 Muse` · `radio.vaked.dev` · `src/favourites.rs`)

`radio.vaked.dev` ships with the **20 Favourite Musics Crate (`family+1` & `#1 Muse`)** — playable directly by slug (`cargo run -- play innerbloom --bt "Soundcore Select 4"`, `cargo run -- play left-hand-free`), by `--track <slug>`, or from the interactive Web Studio crate at [zoneinn.vaked.dev](https://zoneinn.vaked.dev). Every track resolves its local/CDN 320 kbps MP3 master (`/assets/favourites/<slug>.mp3`) with automatic fallback to `zoneInn`'s 432 Hz polyphonic harmonic synthesizer (`VakedStationSource`).

| # | Slug | Title — Artist | Key · BPM | Dedication / Engine |
|---|---|---|---|---|
| 01 | `mahakala` | MAHAKALA (Invocation === Recitation === Song) — 8B × Diamond Way × Techno-Buddhism | Dm9 → C6 → Gsus → D5/A · 108 BPM | 320k MP3 + 432 Hz Synth |
| 02 | `innerbloom` | Innerbloom (Live at Hordern Pavilion / Joshua Tree) — RÜFÜS DU SOL | Cm7 → Ebsus2 → Gm7 → Bb · 122 BPM | Quant Signature of Peter · 320k MP3 + 432 Hz Synth |
| 03 | `faded` | Faded (Atlantis Star `*` & Pillar `△` Answer) — Alan Walker × Iselin Solheim × 8B | D#m → B → F# → C# · 90 BPM | Star 528 Hz & Pillar 288 Hz · 320k MP3 + 432 Hz Synth |
| 04 | `everything-i-wanted` | everything i wanted (Love Mom `*` & Dad `△` Sanctuary) — Billie Eilish × FINNEAS × 8B | Dmaj7 → E6 → C#m7 → F#m7 · 120 BPM | Sanctuary · 320k MP3 + 432 Hz Synth |
| 05 | `anvil` | Anvil (Almost But Never Drops) — Lorn × Deep House | Dm → Bb → F → C · 118 BPM | 320k MP3 + 432 Hz Synth |
| 06 | `the-feeling` | The Feeling / Phoenix (Afterlife Melodic Techno Drop) — Massano × ULTRALOVEGOD | Em → C → G → D · 126 BPM | 320k MP3 + 432 Hz Synth |
| 07 | `love-is-gonna-save-us` | Love Is Gonna Save Us (Backyard Ultra) — ARTBAT × Benny Benassi × Mr Sunshine | Am → F → C → G · 128 BPM | 42 Laps Backyard Ultra · 320k MP3 + 432 Hz Synth |
| 08 | `time-after-time` | Time After Time (Universal Love & Gentle Hug) — P!NK × Cyndi Lauper × Simon Green | C → Em → F → G · 110 BPM | 320k MP3 + 432 Hz Synth |
| 09 | `all-my-favorite-colors` | All My Favorite Colors (Dream Come True Version D+++) — ALEXANDRA DRA `<3` | Fmaj7 → Em7 → Dm7 → Cmaj7 · 88 BPM | Swung Boom-Bap Sanctuary · 320k MP3 + 432 Hz Synth |
| 10 | `space-floating` | Friends of Weed and Love (Space Floating) — ultraLoveGod .p | Fmaj9 → Em7 → Dm9 → Cmaj9 · 76 BPM | 320k MP3 + 432 Hz Synth |
| 11 | `answer-to-oblivion` | The Answer to Oblivion (Chicago \| July 20, 2014) — ULTRALOVEGOD | Am → F → C → G · 132 BPM | 432 Hz + 528 Hz Guardian Bed |
| 12 | `drown-massano` | Drown (Massano Remix · Signal Resolution) — Øostil & Juan Hansen × Massano | Bm → G → D → A · 122 BPM | 320k MP3 + 432 Hz Synth |
| 13 | `sovereign-frequencies` | Sovereign Frequencies (Om Mani Padme Hum v5) — U-Recken × 8B Dharma | Dm → C → Bb → A · 108 BPM | 320k MP3 + 432 Hz Synth |
| 14 | `quantshuffle-canon` | Quantshuffle 1-bit + Ternary Canon (The Love Runtime) — 8b-is | G → Eb → F → D · 108 BPM | Beethoven / Ode to Joy / Für Elise / Pachelbel |
| 15 | `teardrop` | Teardrop (Family+1) — Massive Attack | A → G → Dmaj7 → F#m · 77 BPM | For Yves, Dra `*`, Chris `/\` · 432 Hz Synth |
| 16 | `hunger-of-the-pine` | Hunger of the Pine — alt-J | Cm → Bb → Abmaj7 → Fm · 85 BPM | Family+1 (*This Is All Yours*) · 432 Hz Synth |
| 17 | `left-hand-free` | Left Hand Free (#1 Muse) — alt-J | F#m → B7 → E → C#7 · 106 BPM | For Yves — My Number 1 Muse 💙 · 432 Hz Synth |
| 18 | `glue` | GLUE — BICEP | Cm → Abmaj7 → Eb → Bb · 130 BPM | Family+1 · 2-Step Breakbeat + 432 Hz Synth |
| 19 | `oblivion` | Oblivion — Grimes | Eb → Cm → Ab → Bb · 156 BPM | Family+1 (Live in Chicago, July 20, 2014) · 432 Hz Synth |
| 20 | `not-like-before` | Not Like Before — The Four Owls | Dm7 → Gm7 → Bbmaj7 → A7 · 90 BPM | Family+1 (HighFocusTV) · 432 Hz Synth |

## Safe bluetooth

"Safe" means two things, claimed exactly:

1. **Memory-safe by construction** — no `unsafe` FFI in this crate. On macOS
   and Linux, `src/audio.rs` enumerates CoreAudio / ALSA / PulseAudio output
   devices via `cpal` + `rodio 0.23` (`MixerDeviceSink`) to route directly to a
   paired Bluetooth A2DP sink (`--bt "Soundcore Select 4"`), while `src/bt.rs`
   drives BlueZ on Linux and `zoneinn web` / `zoneinn.vaked.dev` provides live
   Web Bluetooth + `setSinkId` routing in the browser.
2. **Soundcore Select 4 GO first-class** — the speaker ships its own
   profile (`src/bt.rs`), recognized by name before any other device.

## Ultrawhale NAND Ring Oscillator (`src/nand.rs`)

`cargo run -- nand` runs a 5-stage cross-coupled ternary NAND ring oscillator
(`{−1, 0, +1}` + strong-3 Kleene `¬min(a, b)`), quantizes each step into an
8-band `TernFrame` with a 64-bit FNV-1a golden hash, and emits JSONL refeed rows
for [`PeetPedro/ultrawhale-dogfood`](https://huggingface.co/datasets/PeetPedro/ultrawhale-dogfood) and [`peetpedro/ultrawhale-oscillator`](https://huggingface.co/datasets/peetpedro/ultrawhale-oscillator) (`--jsonl`).

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
| 20 favourite musics crate | ✅ live (`src/favourites.rs` — 20 tracks · `family+1` & `#1 Muse`, 320 kbps MP3 + 432 Hz harmonic synth) |
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
