//! menu — the interactive dial menu.
//!
//! The radio's front panel: every station on the vaked dial plus the
//! generic `play <url>` slot, navigated with arrow keys. `snapshot()`
//! renders the same menu as plain text — the E2E and pipe-friendly form,
//! no TTY required.

use std::io::{self, Stdout};

use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{self, Event, KeyCode};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::{Frame, Terminal};

use crate::{bt, stations};

/// The menu as a text snapshot — one render, no TTY needed.
pub fn snapshot() -> String {
    let mut out = String::new();
    out.push_str("quantTernRadio · the dial menu\n");
    for (i, s) in stations::VAKED_DIAL.iter().enumerate() {
        out.push_str(&format!("  [{}] {} — {}\n", i + 1, s.name, s.note));
    }
    out.push_str(&format!(
        "  [{}] play <url> — any URL (tuneIn included)\n",
        stations::VAKED_DIAL.len() + 1
    ));
    out.push_str(&format!(
        "bluetooth backend: {} · {} known speaker(s)\n",
        bt::platform_name(),
        bt::KNOWN_SPEAKERS.len()
    ));
    out.push_str("↑↓ select · enter tune · q quit\n");
    out
}

/// One menu row: the label and what tuning it means.
fn entries() -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = stations::VAKED_DIAL
        .iter()
        .map(|s| (s.name.to_string(), s.note.to_string()))
        .collect();
    v.push((
        "play <url>".to_string(),
        "any URL (tuneIn included)".to_string(),
    ));
    v
}

fn draw(frame: &mut Frame, state: &mut ListState, selected: usize) {
    let items: Vec<ListItem> = entries()
        .iter()
        .enumerate()
        .map(|(i, (label, note))| {
            let cursor = if i == selected { "› " } else { "  " };
            ListItem::new(Line::from(vec![
                Span::styled(cursor.to_string(), Style::default().fg(Color::Cyan)),
                Span::styled(
                    label.clone(),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(ratatui::style::Modifier::BOLD),
                ),
                Span::styled(format!(" — {note}"), Style::default().fg(Color::Gray)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("quantTernRadio · the dial menu")
                .border_style(Style::default().fg(Color::Magenta)),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_stateful_widget(list, frame.area(), state);

    let footer = Line::from(vec![
        Span::styled(
            format!(
                "bluetooth backend: {} · Soundcore Select 4 GO first-class",
                bt::platform_name()
            ),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled("  ·  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "↑↓ select · enter tune · q quit",
            Style::default().fg(Color::Yellow),
        ),
    ]);
    let area = frame.area();
    frame.render_widget(
        ratatui::widgets::Paragraph::new(footer),
        ratatui::layout::Rect::new(area.x, area.y + area.height - 1, area.width, 1),
    );
}

/// Run the interactive menu until enter (tuned) or q (quit).
pub fn run_interactive() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout: Stdout = io::stdout();
    execute!(stdout, ratatui::crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let items = entries();
    let mut state = ListState::default();
    state.select(Some(0));
    let mut tuned: Option<String> = None;

    while tuned.is_none() {
        let selected = state.selected().unwrap_or(0);
        terminal.draw(|f| draw(f, &mut state, selected))?;
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Up => {
                    let next = state.selected().unwrap_or(0).saturating_sub(1);
                    state.select(Some(next));
                }
                KeyCode::Down => {
                    let next = (state.selected().unwrap_or(0) + 1).min(items.len() - 1);
                    state.select(Some(next));
                }
                KeyCode::Enter => {
                    if let Some(i) = state.selected() {
                        tuned = Some(items[i].0.clone());
                    }
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        ratatui::crossterm::terminal::LeaveAlternateScreen
    )?;

    match tuned {
        Some(target) if target == "play <url>" => {
            println!("menu: type `zoneinn play <url>` to tune any URL (tuneIn included)");
        }
        Some(target) => {
            if let Err(e) = crate::audio::play_target(&target, None, 0) {
                eprintln!("playback failed: {e}");
            }
        }
        None => println!("the dial keeps spinning · 0 + 1"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_lists_every_dial_station() {
        let s = snapshot();
        for station in &stations::VAKED_DIAL {
            assert!(s.contains(station.name), "snapshot misses {}", station.name);
        }
        assert!(s.contains("play <url>"));
        assert!(s.contains("bluetooth backend:"));
    }
}
