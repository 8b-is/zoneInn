//! nand — Ultrawhale Ternary NAND Ring Oscillator (`{−1, 0, +1}`).
//!
//! Bridges the 5-stage cross-coupled NAND ring oscillator from `radio.vaked.dev`
//! and `zoneinn.vaked.dev` into `zoneInn`, producing deterministic `{−1, 0, +1}`
//! spectral frames, FNV-1a 64-bit golden hashes, and JSONL refeed rows for
//! `HuggingFace/PeetPedro` (`PeetPedro/ultrawhale-dogfood`).

use crate::tern::{DEFAULT_THRESHOLD, TERN_BANDS, TernFrame};

/// Number of stages in the odd-stage cross-coupled NAND ring oscillator.
pub const RING_STAGES: usize = 5;

/// Canonical initial state for the binary-projected `{−1, +1}` NAND ring.
pub const INITIAL_RING: [i8; RING_STAGES] = [1, -1, 1, -1, 1];

/// Canonical initial state for the Kleene strong-3 `{−1, 0, +1}` NAND ring.
pub const INITIAL_KLEENE: [i8; RING_STAGES] = [1, 0, -1, 0, 1];

/// Project a trit into `{0, 1}` (`+1 -> 1`, `≤0 -> 0`).
#[inline]
pub fn bin(a: i8) -> i8 {
    if a > 0 { 1 } else { 0 }
}

/// Binary-projected NAND gate mapped onto `{−1, +1}`:
/// returns `−1` iff both inputs are `+1`, otherwise `+1`.
#[inline]
pub fn nand(a: i8, b: i8) -> i8 {
    if bin(a) == 1 && bin(b) == 1 { -1 } else { 1 }
}

/// Strong-3 Kleene NAND gate `¬min(a, b)` on `{−1, 0, +1}`:
/// `min(a, b) == 0` stays `0` (rest), `+1 -> −1`, `−1 -> +1`.
#[inline]
pub fn kleene_nand(a: i8, b: i8) -> i8 {
    let m = a.min(b);
    if m == 0 { 0 } else { -m }
}

/// One step of the 5-stage Ultrawhale NAND ring oscillator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NandRingState {
    pub step: u64,
    pub ring: [i8; RING_STAGES],
    pub kleene: [i8; RING_STAGES],
    pub frame: TernFrame,
}

/// Advance the 5-stage cross-coupled NAND ring + Kleene ring by one step
/// and quantize into an 8-band [`TernFrame`].
///
/// Exact parity with `window.zoneInnCore.nandRingStep` in `zoneinn.vaked.dev`.
pub fn step_ring(
    ring: [i8; RING_STAGES],
    kleene: [i8; RING_STAGES],
    step_index: u64,
) -> NandRingState {
    let mut next_ring = [0i8; RING_STAGES];
    let mut next_kleene = [0i8; RING_STAGES];
    let feedback = ring[RING_STAGES - 1];

    for i in 0..RING_STAGES {
        let prev = if i == 0 { feedback } else { ring[i - 1] };
        let partner = ring[(i + 2) % RING_STAGES];
        next_ring[i] = if i == 0 {
            nand(prev, prev)
        } else {
            nand(prev, partner)
        };

        let k_prev = if i == 0 {
            kleene[RING_STAGES - 1]
        } else {
            kleene[i - 1]
        };
        let k_mod = (((step_index + i as u64) % 3) as i8) - 1;
        next_kleene[i] = kleene_nand(k_prev, k_mod);
    }

    let mut bins = [0.0f32; TERN_BANDS];
    for (b, slot) in bins.iter_mut().enumerate() {
        let r_val = next_ring[b % RING_STAGES] as f32;
        let k_val = next_kleene[b % RING_STAGES] as f32;
        *slot = 0.55 * r_val + 0.45 * k_val;
    }
    let frame = TernFrame::from_bins(&bins, DEFAULT_THRESHOLD);

    NandRingState {
        step: step_index,
        ring: next_ring,
        kleene: next_kleene,
        frame,
    }
}

/// Format a trit slice as compact house glyphs (`+`, `·`, `−`).
pub fn format_glyphs(trits: &[i8]) -> String {
    trits
        .iter()
        .map(|&t| match t {
            1 => '+',
            -1 => '−',
            _ => '·',
        })
        .collect()
}

/// Format a trit slice as a JSON array (`[1,-1,0,...]`).
fn format_i8_json(trits: &[i8]) -> String {
    let inner: Vec<String> = trits.iter().map(|t| t.to_string()).collect();
    format!("[{}]", inner.join(","))
}

/// Generate `count` deterministic JSONL rows for refeeding the Ultrawhale
/// NAND oscillator into `PeetPedro/ultrawhale-dogfood`.
pub fn generate_refeed_jsonl(count: usize, ts_iso: &str) -> String {
    let mut ring = INITIAL_RING;
    let mut kleene = INITIAL_KLEENE;
    let mut out = String::new();

    for s in 0..count {
        let st = step_ring(ring, kleene, s as u64);
        ring = st.ring;
        kleene = st.kleene;
        let hash_hex = format!("0x{:016x}", st.frame.hash());
        let short_hash = &hash_hex[2..10];
        let glyphs = format_glyphs(&st.frame.trits);
        let row = format!(
            "{{\"id\":\"ultrawhale-nand-osc-{s}-{short_hash}\",\"ts\":\"{ts_iso}\",\"dim\":\"nand_oscillator\",\"repo\":\"PeetPedro/ultrawhale-dogfood\",\"source\":\"8b-is/zoneInn\",\"step\":{s},\"ring_trits\":{},\"kleene_trits\":{},\"spectrum_trits\":{},\"glyphs\":\"{glyphs}\",\"dominant\":{},\"frame_golden\":\"{hash_hex}\"}}\n",
            format_i8_json(&st.ring),
            format_i8_json(&st.kleene),
            format_i8_json(&st.frame.trits),
            st.frame.dominant,
        );
        out.push_str(&row);
    }
    out
}

/// Render a human-readable summary of `steps` iterations of the NAND oscillator.
pub fn describe_run(steps: usize) -> String {
    let mut ring = INITIAL_RING;
    let mut kleene = INITIAL_KLEENE;
    let mut out = String::new();
    out.push_str("quantTernRadio · ultrawhale NAND ring oscillator · {−1, 0, +1}\n");
    out.push_str("target: HuggingFace/PeetPedro (PeetPedro/ultrawhale-dogfood)\n\n");

    let count = steps.max(1);
    for s in 0..count {
        let st = step_ring(ring, kleene, s as u64);
        ring = st.ring;
        kleene = st.kleene;
        out.push_str(&format!(
            "  step {:02} · ring {} · kleene {} · trits {} · golden 0x{:016x}\n",
            st.step,
            format_glyphs(&st.ring),
            format_glyphs(&st.kleene),
            format_glyphs(&st.frame.trits),
            st.frame.hash()
        ));
    }
    out.push_str("\nthanks Yves & Dra · 0 + 1 · vaked.dev\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nand_truth_table_matches_spec() {
        assert_eq!(nand(1, 1), -1);
        assert_eq!(nand(1, -1), 1);
        assert_eq!(nand(-1, 1), 1);
        assert_eq!(nand(-1, -1), 1);
        assert_eq!(kleene_nand(1, 0), 0);
        assert_eq!(kleene_nand(-1, 0), 1);
        assert_eq!(kleene_nand(1, 1), -1);
    }

    #[test]
    fn step_ring_is_deterministic_and_oscillates() {
        let s0 = step_ring(INITIAL_RING, INITIAL_KLEENE, 0);
        let s0_again = step_ring(INITIAL_RING, INITIAL_KLEENE, 0);
        assert_eq!(s0, s0_again);
        assert_eq!(s0.ring, [-1, 1, 1, -1, 1]);
        assert!(s0.frame.hash() != 0);

        let s1 = step_ring(s0.ring, s0.kleene, 1);
        assert_ne!(s0.ring, s1.ring);
    }

    #[test]
    fn refeed_jsonl_emits_valid_lines() {
        let jsonl = generate_refeed_jsonl(4, "2026-10-18T00:00:00Z");
        let lines: Vec<&str> = jsonl.lines().collect();
        assert_eq!(lines.len(), 4);
        assert!(lines[0].contains("\"dim\":\"nand_oscillator\""));
        assert!(lines[0].contains("\"repo\":\"PeetPedro/ultrawhale-dogfood\""));
        assert!(lines[0].contains("\"source\":\"8b-is/zoneInn\""));
        assert!(lines[0].contains("\"frame_golden\":\"0x"));
    }
}
