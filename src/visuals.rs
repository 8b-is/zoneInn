//! visuals — the quantTernRadio terminal.
//!
//! A live ASCII equalizer in the house aesthetic: the spectrum is run
//! through the ternary quantizer, so every band renders as one of three
//! states — `+` (rising, green), `·` (rest, dim), `−` (falling, red).
//! The radio shows the signal the way the ledger keeps it.

use std::io::{self, Stdout};
use std::time::{Duration, Instant};

use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{self, Event, KeyCode};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::{Frame, Terminal};
use rustfft::FftPlanner;
use rustfft::num_complex::Complex;

use crate::tern::{DEFAULT_THRESHOLD, TERN_BANDS, TernFrame};

const FFT_SIZE: usize = 1024;
const TICK_HZ: f64 = 30.0;

/// Synthesize one FFT window of a slowly breathing chord, so the demo
/// runs anywhere — no audio device, no network, no models.
fn synth_window(t: f64) -> Vec<Complex<f32>> {
    let mut buf = Vec::with_capacity(FFT_SIZE);
    for i in 0..FFT_SIZE {
        let s = i as f64 / FFT_SIZE as f64;
        let v = 0.55 * (t * 0.7 + s * 12.0 * std::f64::consts::TAU).sin()
            + 0.30 * (t * 1.1 + s * 21.0 * std::f64::consts::TAU).sin()
            + 0.15 * (t * 0.3 + s * 3.0 * std::f64::consts::TAU).sin();
        buf.push(Complex::new(v as f32, 0.0));
    }
    buf
}

/// Collapse the FFT magnitudes into [`TERN_BANDS`] log-spaced bands.
fn band_bins(spectrum: &[f32]) -> [f32; TERN_BANDS] {
    let mut out = [0.0f32; TERN_BANDS];
    for (b, o) in out.iter_mut().enumerate() {
        let lo = ((spectrum.len() / 2) as f64).powf(b as f64 / TERN_BANDS as f64) as usize;
        let hi = ((spectrum.len() / 2) as f64).powf((b + 1) as f64 / TERN_BANDS as f64) as usize;
        let hi = hi.max(lo + 1).min(spectrum.len());
        let mut acc = 0.0f32;
        for &v in &spectrum[lo..hi] {
            acc += v;
        }
        *o = (acc / (hi - lo) as f32).sqrt() / 40.0;
    }
    out
}

/// One synthetic demo frame at time `t` — shared by the TUI and the
/// one-shot ASCII lane, so both read the same signal.
pub fn demo_frame(t: f64) -> TernFrame {
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(FFT_SIZE);
    let mut buf = synth_window(t);
    fft.process(&mut buf);
    let spectrum: Vec<f32> = buf.iter().map(|c| c.norm()).collect();
    TernFrame::from_bins(&band_bins(&spectrum), DEFAULT_THRESHOLD)
}

fn draw(frame: &mut Frame, band: &TernFrame, station: &str, t: f64) {
    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(vec![Span::styled(
        "quantTernRadio · zoneInn · {−1, 0, +1}",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(ratatui::style::Modifier::BOLD),
    )]));
    lines.push(Line::from(vec![Span::styled(
        format!("station: {station} · demo signal (offline)"),
        Style::default().fg(Color::Gray),
    )]));
    lines.push(Line::from(""));

    // The ternary equalizer: three states per band, block-rendered.
    let mut bar = String::new();
    let mut legend = String::new();
    for (i, &t) in band.trits.iter().enumerate() {
        let ch = match t {
            1 => '█',
            -1 => '█',
            _ => '░',
        };
        let height = 1 + (i * 3) % 8;
        bar.push_str(&ch.to_string().repeat(height));
        legend.push(match t {
            1 => '+',
            -1 => '−',
            _ => '·',
        });
        if i + 1 < TERN_BANDS {
            bar.push(' ');
            legend.push(' ');
        }
    }
    // Colorize the bar by segments (three spans, kept simple).
    lines.push(Line::from(vec![
        Span::styled("bands   ", Style::default().fg(Color::Gray)),
        Span::styled(bar.clone(), Style::default().fg(Color::Green)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("trits   ", Style::default().fg(Color::Gray)),
        Span::styled(legend, Style::default().fg(Color::Cyan)),
    ]));
    lines.push(Line::from(vec![Span::styled(
        format!(
            "dominant {}",
            if band.dominant == 1 {
                "+1"
            } else if band.dominant == -1 {
                "−1"
            } else {
                "·0"
            }
        ),
        Style::default().fg(Color::Yellow),
    )]));
    lines.push(Line::from(vec![Span::styled(
        format!("frame golden 0x{:016x}", band.hash()),
        Style::default().fg(Color::DarkGray),
    )]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        format!("t = {t:.1}s · q to quit"),
        Style::default().fg(Color::DarkGray),
    )]));
    lines.push(Line::from(vec![Span::styled(
        "thanks Yves & Dra · safe bluetooth · Soundcore Select 4 GO first-class · 0 + 1 · vaked.dev",
        Style::default().fg(Color::Magenta),
    )]));

    frame.render_widget(Paragraph::new(lines), frame.area());
}

/// Run the demo TUI for `seconds` (0 = until `q`).
pub fn run_demo(station: &str, seconds: u64) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout: Stdout = io::stdout();
    execute!(stdout, ratatui::crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let start = Instant::now();
    let mut last_tick = start;

    loop {
        if event::poll(Duration::from_millis(16))?
            && let Event::Key(key) = event::read()?
            && key.code == KeyCode::Char('q')
        {
            break;
        }
        let now = Instant::now();
        if now.duration_since(last_tick).as_secs_f64() >= 1.0 / TICK_HZ {
            last_tick = now;
            let t = now.duration_since(start).as_secs_f64();
            let frame = demo_frame(t);

            terminal.draw(|f| draw(f, &frame, station, t))?;
        }
        if seconds > 0 && now.duration_since(start).as_secs() >= seconds {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        ratatui::crossterm::terminal::LeaveAlternateScreen
    )?;
    Ok(())
}
