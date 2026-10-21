//! Loopback JSON-lines control server: one request per line, one reply per line.
//! This is the transport the MCP server (`soundcraft-cli mcp --connect PORT`) wraps.
//!
//! Lines are bounded ([`MAX_LINE`]): a longer one is refused and the connection closed, without
//! buffering the rest. So is the number of connections served at once ([`MAX_CONNECTIONS`]).
//!
//! Every connection must authenticate first: its first request is
//! `{"method": "auth", "params": {"token": "<64 hex>"}}` with this launch's token
//! ([`crate::control_token`]). Anything else gets [`AUTH_REQUIRED`] and the connection is closed,
//! so another program on the machine (or a web page) can't drive the app.

use serde_json::{Value, json};
use soundcraft_ui_egui::ControlRequest;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

/// Longest accepted request line (bytes, without the newline). The largest payload a request
/// carries is a plugin state for `mix.insert_state`: base64 of at most `b64::MAX_DECODED_BYTES`
/// (64 MiB, so about 85 MiB of text). Room for that and 1 MiB of JSON around it; replies
/// (screenshots) aren't bounded by this.
const MAX_LINE: usize = soundcraft_engine::model::b64::MAX_DECODED_BYTES.div_ceil(3) * 4 + (1 << 20);
// With MAX_CONNECTIONS, at most 1.5 GiB of request lines are buffered at once.
const _: () = assert!(MAX_LINE <= 96 << 20);
/// Connections served at once; further ones get an error line and are closed.
const MAX_CONNECTIONS: usize = 16;
/// The error a connection gets when its first request is not a good `auth`.
pub const AUTH_REQUIRED: &str = "authentication required";

/// Bind the control listener. Always the IPv4 loopback address, never a wildcard or a LAN address:
/// the control channel must not be reachable from another machine.
fn bind_loopback(port: u16) -> std::io::Result<TcpListener> {
    TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port))
}

pub fn start(port: u16, token: String, ctx: egui::Context) -> Receiver<ControlRequest> {
    let (tx, rx) = channel::<ControlRequest>();
    let listener = match bind_loopback(port) {
        Ok(l) => l,
        Err(e) => {
            log::error!("control server failed to bind 127.0.0.1:{port}: {e}");
            return rx;
        }
    };
    let actual = listener.local_addr().map(|a| a.port()).unwrap_or(port);
    log::info!("control server listening on 127.0.0.1:{actual}");
    println!("SOUNDCRAFT_CONTROL_PORT={actual}");
    let token: Arc<str> = token.into();
    let spawned = std::thread::Builder::new().name("soundcraft-control".into()).spawn(move || accept_loop(listener, token, tx, ctx));
    if let Err(e) = spawned {
        log::error!("control thread failed: {e}");
    }
    rx
}

/// A connection's place among [`MAX_CONNECTIONS`], given back when the connection ends (also when
/// its thread can't be started, or panics).
struct Slot(Arc<AtomicUsize>);

impl Drop for Slot {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

fn accept_loop(listener: TcpListener, token: Arc<str>, tx: Sender<ControlRequest>, ctx: egui::Context) {
    let open = Arc::new(AtomicUsize::new(0));
    for mut stream in listener.incoming().flatten() {
        if open.fetch_add(1, Ordering::SeqCst) >= MAX_CONNECTIONS {
            open.fetch_sub(1, Ordering::SeqCst);
            let _ = writeln!(stream, "{}", json!({"ok": false, "error": "too many control connections"}));
            continue;
        }
        let slot = Slot(Arc::clone(&open));
        let tx = tx.clone();
        let ctx = ctx.clone();
        let token = Arc::clone(&token);
        let _ = std::thread::Builder::new().name("soundcraft-control-conn".into()).spawn(move || {
            let _slot = slot;
            serve(stream, &token, tx, ctx);
        });
    }
}

/// Read one `\n`-terminated line of at most `max` bytes into `buf` (newline stripped).
/// `Ok(false)` at end of stream; `Err` when the line is too long or the read fails.
fn read_bounded_line(reader: &mut impl BufRead, buf: &mut Vec<u8>, max: usize) -> Result<bool, String> {
    buf.clear();
    // `take` bounds the bytes buffered for one line: a peer can't make us allocate more than `max`.
    let limit = u64::try_from(max).unwrap_or(u64::MAX).saturating_add(1);
    let n = reader.take(limit).read_until(b'\n', buf).map_err(|e| e.to_string())?;
    if n == 0 {
        return Ok(false);
    }
    if buf.last() == Some(&b'\n') {
        buf.pop();
        if buf.last() == Some(&b'\r') {
            buf.pop();
        }
        return Ok(true);
    }
    if buf.len() > max {
        return Err(format!("request line longer than {max} bytes; closing the connection"));
    }
    // Last line without a newline, then end of stream.
    Ok(true)
}

/// Whether a request authenticates with `token`.
fn authenticates(method: &str, params: &Value, token: &str) -> bool {
    method == "auth" && params.get("token").and_then(Value::as_str).is_some_and(|t| crate::control_token::matches(t, token))
}

fn serve(stream: TcpStream, token: &str, tx: Sender<ControlRequest>, ctx: egui::Context) {
    let Ok(read) = stream.try_clone() else { return };
    let mut out = stream;
    let mut reader = BufReader::new(read);
    let mut buf = Vec::new();
    let mut authenticated = false;
    loop {
        match read_bounded_line(&mut reader, &mut buf, MAX_LINE) {
            Ok(true) => {}
            Ok(false) => break,
            Err(e) => {
                let _ = writeln!(out, "{}", json!({"ok": false, "error": e}));
                break;
            }
        }
        // Bytes that aren't UTF-8 end the connection, as `BufRead::lines` did.
        let Ok(line) = std::str::from_utf8(&buf) else { break };
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(line) {
            Ok(msg) => {
                let id = msg.get("id").cloned().unwrap_or(Value::Null);
                let method = msg.get("method").and_then(Value::as_str).unwrap_or("").to_string();
                let params = msg.get("params").cloned().unwrap_or(json!({}));
                if !authenticated {
                    if !authenticates(&method, &params, token) {
                        let _ = writeln!(out, "{}", json!({"id": id, "ok": false, "error": AUTH_REQUIRED})).and_then(|()| out.flush());
                        break;
                    }
                    authenticated = true;
                    if writeln!(out, "{}", json!({"id": id, "ok": true, "result": {"authenticated": true}})).is_err() {
                        break;
                    }
                    continue;
                }
                let (req, rrx) = ControlRequest::new(method, params);
                if tx.send(req).is_err() {
                    break;
                }
                ctx.request_repaint();
                let mut r = rrx.recv_timeout(Duration::from_secs(90)).unwrap_or_else(|_| json!({"ok": false, "error": "timeout"}));
                if let Some(o) = r.as_object_mut() {
                    o.insert("id".into(), id);
                }
                r
            }
            Err(e) if !authenticated => {
                let _ = writeln!(out, "{}", json!({"ok": false, "error": AUTH_REQUIRED, "detail": format!("bad JSON: {e}")}));
                break;
            }
            Err(e) => json!({"ok": false, "error": format!("bad JSON: {e}")}),
        };
        if writeln!(out, "{reply}").is_err() {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_are_read_up_to_the_limit() {
        let mut buf = Vec::new();
        let mut input: &[u8] = b"aaaaaaaaaa\nbb\r\ncc";
        assert_eq!(read_bounded_line(&mut input, &mut buf, 10), Ok(true));
        assert_eq!(buf, b"aaaaaaaaaa", "a line exactly at the limit is accepted");
        assert_eq!(read_bounded_line(&mut input, &mut buf, 10), Ok(true));
        assert_eq!(buf, b"bb", "CRLF is stripped");
        assert_eq!(read_bounded_line(&mut input, &mut buf, 10), Ok(true));
        assert_eq!(buf, b"cc", "a last line without a newline is still read");
        assert_eq!(read_bounded_line(&mut input, &mut buf, 10), Ok(false), "then end of stream");
    }

    #[test]
    fn an_overlong_line_is_refused_without_reading_the_rest() {
        let mut buf = Vec::new();
        let long = [b'x'; 100];
        let mut input: &[u8] = &long;
        assert!(read_bounded_line(&mut input, &mut buf, 10).is_err());
        assert!(buf.len() <= 11, "at most limit + 1 bytes are buffered, got {}", buf.len());
        assert_eq!(input.len(), 100 - 11, "the rest of the line is left unread");
    }

    #[test]
    fn the_largest_plugin_state_fits_in_a_line() {
        let state = soundcraft_engine::model::b64::MAX_DECODED_BYTES.div_ceil(3) * 4;
        let request = json!({"id": 1, "method": "engine.execute", "params": {"command": "mix.insert_state", "params": {"track": "Audio 1", "slot": 0, "state": ""}}});
        assert!(state + request.to_string().len() <= MAX_LINE);
    }

    const TOKEN: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    /// The control server on a loopback port, with a stand-in UI that echoes each method.
    fn serve_echo() -> std::net::SocketAddr {
        let listener = bind_loopback(0).unwrap();
        let addr = listener.local_addr().unwrap();
        let (tx, rx) = channel::<ControlRequest>();
        std::thread::spawn(move || accept_loop(listener, TOKEN.into(), tx, egui::Context::default()));
        std::thread::spawn(move || {
            for req in rx {
                let _ = req.reply.send(json!({"ok": true, "method": req.method}));
            }
        });
        addr
    }

    #[test]
    fn requests_are_answered_with_their_ids() {
        let addr = serve_echo();
        let mut c = TcpStream::connect(addr).unwrap();
        c.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let auth = json!({"id": 0, "method": "auth", "params": {"token": TOKEN}});
        c.write_all(format!("{auth}\n{{\"id\":3,\"method\":\"ping\"}}\r\n\n{{\"id\":4,\"method\":\"pong\"}}\n").as_bytes()).unwrap();
        let mut r = BufReader::new(c);
        let mut line = String::new();
        r.read_line(&mut line).unwrap();
        assert!(line.contains("\"authenticated\":true"), "{line}");
        for (id, method) in [(3, "ping"), (4, "pong")] {
            let mut line = String::new();
            r.read_line(&mut line).unwrap();
            let v: Value = serde_json::from_str(&line).unwrap();
            assert_eq!((v["id"].as_i64(), v["method"].as_str()), (Some(id), Some(method)), "{line}");
        }
    }

    #[test]
    fn extra_connections_are_refused() {
        let addr = serve_echo();
        // Idle connections are accepted in order and held open.
        let held: Vec<TcpStream> = (0..MAX_CONNECTIONS).map(|_| TcpStream::connect(addr).unwrap()).collect();
        let mut extra = TcpStream::connect(addr).unwrap();
        extra.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let mut line = String::new();
        BufReader::new(&mut extra).read_line(&mut line).unwrap();
        assert!(line.contains("too many control connections"), "{line}");
        drop(held);
    }

    /// Every reply line on a fresh connection that sends `input`, until the server closes it.
    fn exchange(addr: std::net::SocketAddr, input: &str) -> Vec<Value> {
        let mut c = TcpStream::connect(addr).unwrap();
        c.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        c.write_all(input.as_bytes()).unwrap();
        let _ = c.shutdown(std::net::Shutdown::Write);
        let mut text = String::new();
        c.read_to_string(&mut text).unwrap();
        text.lines().map(|l| serde_json::from_str(l).unwrap()).collect()
    }

    #[test]
    fn the_first_request_must_authenticate() {
        let addr = serve_echo();
        let wrong = "f".repeat(64);
        for first in [
            json!({"id": 1, "method": "engine.execute", "params": {"command": "transport.play"}}),
            json!({"id": 1, "method": "engine.execute", "params": {"token": TOKEN}}),
            json!({"id": 1, "method": "auth"}),
            json!({"id": 1, "method": "auth", "params": {"token": wrong}}),
            json!({"id": 1, "method": "auth", "params": {"token": &TOKEN[..63]}}),
            json!({"id": 1, "method": "auth", "params": {"token": format!("{TOKEN}0")}}),
            json!({"id": 1, "method": "auth", "params": {"token": 5}}),
        ] {
            let r = exchange(addr, &format!("{first}\n{}\n", json!({"id": 2, "method": "ui.inspect"})));
            assert_eq!(r.len(), 1, "{first}: only the refusal, then the connection is closed: {r:?}");
            assert_eq!((r[0]["ok"].clone(), r[0]["error"].clone()), (json!(false), json!(AUTH_REQUIRED)), "{first}");
        }
        let r = exchange(addr, "GET / HTTP/1.1\r\n\r\n{\"id\":2,\"method\":\"ui.inspect\"}\n");
        assert_eq!((r.len(), r[0]["error"].clone()), (1, json!(AUTH_REQUIRED)), "junk first is refused too");
        let r = exchange(
            addr,
            &format!("{}\n{}\n", json!({"id": "a", "method": "auth", "params": {"token": TOKEN}}), json!({"id": 2, "method": "ui.inspect"})),
        );
        assert_eq!((r[0]["id"].clone(), r[0]["ok"].clone()), (json!("a"), json!(true)));
        assert_eq!((r[1]["id"].clone(), r[1]["method"].clone()), (json!(2), json!("ui.inspect")));
    }

    #[test]
    fn the_listener_is_bound_to_loopback_only() {
        let listener = bind_loopback(0).unwrap();
        assert_eq!(listener.local_addr().unwrap().ip(), std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));
    }
}
