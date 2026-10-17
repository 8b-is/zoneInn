//! tern — the ternary heart of quantTernRadio.
//!
//! Every spectrum frame is quantized to `{-1, 0, +1}` (the constellation's
//! affect alphabet): a band reads `+1` (rising), `0` (rest), or `−1`
//! (falling). The visualizer draws only three states — the radio shows the
//! signal the way the ledger keeps it.

/// The quantTernRadio spectrum runs eight bands — one byte of trits per
/// frame, the same width as the family's 8-trit gate vector.
pub const TERN_BANDS: usize = 8;

/// Affirm threshold: a band must swing at least this far from rest to read
/// `±1`. Inside the band it is a genuine zero, never guessed.
pub const DEFAULT_THRESHOLD: f32 = 0.08;

/// Quantize one band to the ternary alphabet.
#[inline]
pub fn quantize_band(v: f32, threshold: f32) -> i8 {
    if v >= threshold {
        1
    } else if v <= -threshold {
        -1
    } else {
        0
    }
}

/// Quantize a full spectrum frame (any length; first [`TERN_BANDS`] bins
/// are used, shorter input zero-pads).
pub fn quantize_spectrum(bins: &[f32], threshold: f32) -> [i8; TERN_BANDS] {
    let mut out = [0i8; TERN_BANDS];
    for (i, o) in out.iter_mut().enumerate() {
        *o = bins.get(i).map_or(0, |&v| quantize_band(v, threshold));
    }
    out
}

/// One ternary frame: the eight trits plus the dominant state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TernFrame {
    pub trits: [i8; TERN_BANDS],
    /// The band the frame leans on (`+1`, `0`, or `−1`).
    pub dominant: i8,
}

impl TernFrame {
    pub fn from_bins(bins: &[f32], threshold: f32) -> Self {
        let trits = quantize_spectrum(bins, threshold);
        let mut plus = 0i32;
        let mut minus = 0i32;
        for &t in &trits {
            plus += (t == 1) as i32;
            minus += (t == -1) as i32;
        }
        let dominant = if plus > minus {
            1
        } else if minus > plus {
            -1
        } else {
            0
        };
        Self { trits, dominant }
    }

    /// FNV-1a over the eight trits — the pinned frame golden.
    pub fn hash(&self) -> u64 {
        let mut h: u64 = 0xcbf29ce484222325;
        for &t in &self.trits {
            h ^= (t + 1) as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h
    }

    /// The frame as a `qt:`-style string: `−`/`·`/`+` glyphs.
    pub fn glyphs(&self) -> String {
        self.trits
            .iter()
            .map(|&t| match t {
                1 => '+',
                -1 => '−',
                _ => '·',
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn band_thresholds_are_honest() {
        assert_eq!(quantize_band(0.5, DEFAULT_THRESHOLD), 1);
        assert_eq!(quantize_band(-0.5, DEFAULT_THRESHOLD), -1);
        assert_eq!(quantize_band(0.03, DEFAULT_THRESHOLD), 0);
        assert_eq!(quantize_band(-0.03, DEFAULT_THRESHOLD), 0);
        // Exactly at threshold binds affirm — boundary belongs to the signal.
        assert_eq!(quantize_band(DEFAULT_THRESHOLD, DEFAULT_THRESHOLD), 1);
    }

    #[test]
    fn short_input_zero_pads() {
        let frame = TernFrame::from_bins(&[0.9, -0.9], DEFAULT_THRESHOLD);
        assert_eq!(frame.trits[0], 1);
        assert_eq!(frame.trits[1], -1);
        assert!(frame.trits[2..].iter().all(|&t| t == 0));
    }

    #[test]
    fn dominant_state_is_counted() {
        let frame = TernFrame::from_bins(
            &[0.9, 0.9, 0.9, -0.9, -0.9, 0.0, 0.0, 0.0],
            DEFAULT_THRESHOLD,
        );
        assert_eq!(frame.dominant, 1);
        assert_eq!(frame.glyphs(), "+++−−···");
    }

    #[test]
    fn golden_frame_hash_is_pinned() {
        // Canonical frame: + − + 0 + − 0 + — pinned at the first render.
        let frame = TernFrame {
            trits: [1, -1, 1, 0, 1, -1, 0, 1],
            dominant: 1,
        };
        assert_eq!(frame.hash(), 16_208_427_172_629_396_947);
    }

    #[test]
    fn frames_are_deterministic() {
        let bins = [0.1, -0.2, 0.0, 0.5, -0.01, 0.09, -0.5, 0.0];
        let a = TernFrame::from_bins(&bins, DEFAULT_THRESHOLD);
        let b = TernFrame::from_bins(&bins, DEFAULT_THRESHOLD);
        assert_eq!(a, b);
        assert_eq!(a.hash(), b.hash());
    }
}
