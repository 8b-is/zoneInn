//! web — the zoneInn Web Radio & Bluetooth Studio lane.
//!
//! Embeds the single-file `zoneinn.vaked.dev` studio (`web/index.html`) directly
//! into the `quantternradio` binary and serves it over `http://127.0.0.1:8088`
//! (`localhost` is a W3C Secure Context, unlocking `navigator.bluetooth`,
//! `AudioContext.setSinkId`, and `navigator.mediaDevices.selectAudioOutput`).
//! Also exposes JSON endpoints (`/api/status`, `/api/frame`, `/api/nand`) so any
//! browser or agent can inspect the live `{−1, 0, +1}` equalizer and NAND ring.

use std::io::{self, Read, Write};
use std::net::TcpListener;

use crate::{ascii, bt, favourites, nand, stations, visuals};

/// Embedded single-file Web Radio & Bluetooth Studio (`web/index.html`).
pub const INDEX_HTML: &str = include_str!("../web/index.html");

/// Embedded SVG favicon (`web/favicon.svg`).
pub const FAVICON_SVG: &str = include_str!("../web/favicon.svg");

/// Default bind address for `quantternradio web`.
pub const DEFAULT_BIND: &str = "127.0.0.1:8088";

/// Escape a string for safe inclusion inside a JSON string literal.
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

/// Build the `/api/favourites` JSON payload listing all 14 Favourite Musics on `radio.vaked.dev`.
pub fn favourites_json() -> String {
    let mut items = Vec::with_capacity(favourites::FAVOURITE_MUSICS.len());
    for t in &favourites::FAVOURITE_MUSICS {
        items.push(format!(
            "{{\"index\":{},\"slug\":\"{}\",\"emoji\":\"{}\",\"title\":\"{}\",\"artist\":\"{}\",\"key_progression\":\"{}\",\"bpm\":{},\"tuning_hz\":{},\"trit_read\":\"{}\",\"web_url\":\"{}\"}}",
            t.index,
            json_escape(t.slug),
            json_escape(t.emoji),
            json_escape(t.title),
            json_escape(t.artist),
            json_escape(t.key_progression),
            t.bpm,
            t.tuning_hz,
            json_escape(t.trit_read),
            json_escape(t.web_url),
        ));
    }
    format!(
        "{{\"station\":\"radio.vaked.dev\",\"count\":{},\"tracks\":[{}]}}",
        favourites::FAVOURITE_MUSICS.len(),
        items.join(",")
    )
}

/// Build the `/api/status` JSON payload describing stations, Bluetooth backend,
/// first-class `Soundcore Select 4 GO` speaker profile, and HF refeed target.
pub fn status_json() -> String {
    let mut dial_items = Vec::new();
    for s in &stations::VAKED_DIAL {
        let url = s
            .stream_url
            .map(|u| format!("\"{}\"", json_escape(u)))
            .unwrap_or_else(|| "null".to_string());
        dial_items.push(format!(
            "{{\"name\":\"{}\",\"stream_url\":{},\"note\":\"{}\"}}",
            json_escape(s.name),
            url,
            json_escape(s.note)
        ));
    }
    let sc = &bt::SOUNDCORE_SELECT_4_GO;
    format!(
        "{{\"service\":\"quantTernRadio\",\"web_url\":\"https://zoneinn.vaked.dev\",\"bluetooth_backend\":\"{:?}\",\"featured_speaker\":{{\"model\":\"{}\",\"name_match\":\"{}\",\"audio_class\":\"{}\"}},\"nand_refeed_repo\":\"PeetPedro/ultrawhale-dogfood\",\"favourites_count\":{},\"stations\":[{}]}}",
        bt::platform(),
        json_escape(sc.model),
        json_escape(sc.name_match),
        json_escape(sc.audio_class),
        favourites::FAVOURITE_MUSICS.len(),
        dial_items.join(",")
    )
}

/// Build the `/api/frame` JSON payload for a given `station` and time `t`.
pub fn frame_json(station: &str, t: f64) -> String {
    let frame = visuals::demo_frame(t);
    let ascii_card = ascii::render(&frame, station, t);
    let trits: Vec<String> = frame.trits.iter().map(|v| v.to_string()).collect();
    format!(
        "{{\"station\":\"{}\",\"t\":{t:.3},\"trits\":[{}],\"dominant\":{},\"frame_golden\":\"0x{:016x}\",\"ascii\":\"{}\"}}",
        json_escape(station),
        trits.join(","),
        frame.dominant,
        frame.hash(),
        json_escape(&ascii_card)
    )
}

/// Build the `/api/nand` JSON payload for `step` of the Ultrawhale NAND oscillator.
pub fn nand_json(step: u64) -> String {
    let mut ring = nand::INITIAL_RING;
    let mut kleene = nand::INITIAL_KLEENE;
    let mut st = nand::step_ring(ring, kleene, 0);
    for s in 0..=step {
        st = nand::step_ring(ring, kleene, s);
        ring = st.ring;
        kleene = st.kleene;
    }
    let ring_s: Vec<String> = st.ring.iter().map(|v| v.to_string()).collect();
    let kleene_s: Vec<String> = st.kleene.iter().map(|v| v.to_string()).collect();
    let trits_s: Vec<String> = st.frame.trits.iter().map(|v| v.to_string()).collect();
    format!(
        "{{\"step\":{},\"ring_trits\":[{}],\"kleene_trits\":[{}],\"spectrum_trits\":[{}],\"glyphs\":\"{}\",\"dominant\":{},\"frame_golden\":\"0x{:016x}\",\"repo\":\"PeetPedro/ultrawhale-dogfood\"}}",
        st.step,
        ring_s.join(","),
        kleene_s.join(","),
        trits_s.join(","),
        nand::format_glyphs(&st.frame.trits),
        st.frame.dominant,
        st.frame.hash()
    )
}

/// Route an HTTP request path (`GET /path`) to `(status_line, content_type, body)`.
pub fn route_request(path: &str) -> (&'static str, &'static str, String) {
    let clean = path.split('?').next().unwrap_or("/");
    match clean {
        "/" | "/index.html" => ("200 OK", "text/html; charset=utf-8", INDEX_HTML.to_string()),
        "/favicon.svg" => (
            "200 OK",
            "image/svg+xml; charset=utf-8",
            FAVICON_SVG.to_string(),
        ),
        "/api/status" => ("200 OK", "application/json; charset=utf-8", status_json()),
        "/api/favourites" => (
            "200 OK",
            "application/json; charset=utf-8",
            favourites_json(),
        ),
        "/api/frame" => (
            "200 OK",
            "application/json; charset=utf-8",
            frame_json("radio.vaked.dev", 0.0),
        ),
        "/api/nand" => ("200 OK", "application/json; charset=utf-8", nand_json(0)),
        "/api/nand.jsonl" => (
            "200 OK",
            "application/jsonl; charset=utf-8",
            nand::generate_refeed_jsonl(16, "2026-10-18T00:00:00Z"),
        ),
        _ => (
            "404 Not Found",
            "application/json; charset=utf-8",
            "{\"error\":\"not found\"}".to_string(),
        ),
    }
}

/// Serve the zoneInn Web Radio & Bluetooth Studio over HTTP on `bind`.
///
/// If `max_requests` is `Some(n)`, exits cleanly after serving `n` requests
/// (used by deterministic integration tests).
pub fn serve(bind: &str, max_requests: Option<usize>) -> io::Result<()> {
    let listener = TcpListener::bind(bind)?;
    let local_addr = listener.local_addr()?;
    println!("quantTernRadio · web studio live at http://{local_addr}");
    println!("  public mirror: https://zoneinn.vaked.dev");
    println!(
        "  bluetooth:     Web Bluetooth + setSinkId + {:?} (Soundcore Select 4 GO first-class)",
        bt::platform()
    );
    println!("  nand refeed:   PeetPedro/ultrawhale-dogfood (/api/nand.jsonl)");
    println!("thanks Yves & Dra <3");

    let mut served = 0usize;
    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(_) => continue,
        };
        let mut buf = [0u8; 4096];
        let n = stream.read(&mut buf).unwrap_or(0);
        let req = String::from_utf8_lossy(&buf[..n]);
        let path = req
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .unwrap_or("/");

        let clean = path.split('?').next().unwrap_or("/");
        if let Some(rest) = clean.strip_prefix("/assets/favourites/")
            && let Some(slug) = rest.strip_suffix(".mp3")
            && let Some(fav) = favourites::find_favourite(slug)
            && let Some(local_path) = fav.resolve_local_path()
            && let Ok(bytes) = std::fs::read(&local_path)
        {
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: audio/mpeg\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n",
                bytes.len()
            );
            let _ = stream.write_all(header.as_bytes());
            let _ = stream.write_all(&bytes);
            let _ = stream.flush();
        } else {
            let (status, content_type, body) = route_request(path);
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nPermissions-Policy: bluetooth=(self), speaker-selection=(self), autoplay=(self)\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }

        served += 1;
        if let Some(limit) = max_requests
            && served >= limit
        {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_index_contains_bluetooth_and_nand_studio() {
        assert!(INDEX_HTML.contains("window.zoneInnCore"));
        assert!(INDEX_HTML.contains("Soundcore Select 4 GO"));
        assert!(INDEX_HTML.contains("navigator.bluetooth"));
        assert!(INDEX_HTML.contains("PeetPedro/ultrawhale-dogfood"));
        assert!(INDEX_HTML.contains("FAVOURITE_MUSICS"));
    }

    #[test]
    fn routes_return_valid_payloads() {
        let (st, ct, body) = route_request("/api/status");
        assert_eq!(st, "200 OK");
        assert!(ct.contains("application/json"));
        assert!(body.contains("Soundcore Select 4 GO"));
        assert!(body.contains("PeetPedro/ultrawhale-dogfood"));
        assert!(body.contains("\"favourites_count\":14"));

        let (st_fav, _, body_fav) = route_request("/api/favourites");
        assert_eq!(st_fav, "200 OK");
        assert!(body_fav.contains("\"slug\":\"mahakala\""));
        assert!(body_fav.contains("\"slug\":\"innerbloom\""));

        let (st_f, _, body_f) = route_request("/api/frame");
        assert_eq!(st_f, "200 OK");
        assert!(body_f.contains("0x7311bcbd581290fd"));

        let (st_n, _, body_n) = route_request("/api/nand");
        assert_eq!(st_n, "200 OK");
        assert!(body_n.contains("\"ring_trits\":[-1,1,1,-1,1]"));
    }
}
