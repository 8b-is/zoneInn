# zoneInn — quantTernRadio

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
cargo run -- list                 # the dial
cargo run -- demo --seconds 30    # the ternary equalizer, offline, no device
cargo run -- play <url>           # tune a stream (playback: next slice)
cargo run -- play radio.vaked.dev --bt "Soundcore Select 4"
cargo run -- voice "you are listening to radio.vaked.dev"
```

## The ternary equalizer

Every spectrum frame collapses to eight bands and quantizes to
`{−1, 0, +1}` — the constellation's affect alphabet. A band reads `+`
(rising), `·` (rest), or `−` (falling); inside the threshold it is a
genuine zero, never guessed. The frame's FNV-1a hash is pinned in
`src/tern.rs` (`0xe0eb9e81c4a71d13…` for the canonical frame) so any
change to the quantizer is a visible event.

## Safe bluetooth

"Safe" means two things, claimed exactly:

1. **Memory-safe by construction** — no `unsafe` FFI in this crate. The
   Linux backend (BlueZ via the pure-Rust `bluer` crate) is the first
   wire; macOS/Windows are honest stubs until their bridges land.
2. **Soundcore Select 4 GO first-class** — the speaker ships its own
   profile (`src/bt.rs`), recognized by name before any other device.

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
| stream playback | ⬜ next slice (rodio 0.23 Player + symphonia + http) |
| bluetooth backend | ⬜ Linux wire next (bluer); profile logic ✅ |
| SUNO OSS voice bridge | ⬜ HTTP bridge next (bark-selfhost endpoint) |
| agy visuals | ✅ shell-out lane live |

*the constellation · 0 + 1 · fine touch from within · vaked.dev*
