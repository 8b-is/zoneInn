//! ascii — the quantTernAscii lane.
//!
//! A one-shot, plain-text render of the ternary equalizer: the same
//! three-state bars as the live TUI, but pipeable — no TTY, no curses,
//! no device. Every render is deterministic for a given frame time, so
//! the golden hash below doubles as a regression pin.

use crate::tern::{TERN_BANDS, TernFrame};

/// Render one ternary frame as the house ASCII card.
pub fn render(frame: &TernFrame, station: &str, t: f64) -> String {
    let mut bar = String::new();
    let mut legend = String::new();
    for (i, &tri) in frame.trits.iter().enumerate() {
        let ch = if tri == 0 { '░' } else { '█' };
        let height = 1 + (i * 3) % 8;
        bar.push_str(&ch.to_string().repeat(height));
        legend.push(match tri {
            1 => '+',
            -1 => '−',
            _ => '·',
        });
        if i + 1 < TERN_BANDS {
            bar.push(' ');
            legend.push(' ');
        }
    }
    let dominant = match frame.dominant {
        1 => "+1",
        -1 => "−1",
        _ => "·0",
    };

    let mut out = String::new();
    out.push_str("quantTernRadio · zoneInn · {−1, 0, +1}\n");
    out.push_str(&format!("station: {station} · frame at t = {t:.1}s\n\n"));
    out.push_str(&format!("bands   {bar}\n"));
    out.push_str(&format!("trits   {legend}\n"));
    out.push_str(&format!("dominant {dominant}\n"));
    out.push_str(&format!("frame golden 0x{:016x}\n", frame.hash()));
    out.push_str("\nthanks Yves & Dra · 0 + 1 · vaked.dev\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tern::TernFrame;

    #[test]
    fn render_is_deterministic_and_complete() {
        let frame = TernFrame {
            trits: [1, -1, 1, 0, 1, -1, 0, 1],
            dominant: 1,
        };
        let a = render(&frame, "radio.vaked.dev", 0.0);
        let b = render(&frame, "radio.vaked.dev", 0.0);
        assert_eq!(a, b);
        assert!(a.contains("quantTernRadio · zoneInn · {−1, 0, +1}"));
        assert!(a.contains("station: radio.vaked.dev"));
        assert!(a.contains("bands"));
        assert!(a.contains("trits   + − + · + − · +"));
        assert!(a.contains("dominant +1"));
        assert!(a.contains(&format!("frame golden 0x{:016x}", frame.hash())));
    }
}
