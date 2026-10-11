//! The live socket (`/ws`): one per page that needs it. It watches a
//! campaign and hands each server message to a callback. On a close it
//! connects again (1 s, 2 s, … up to 15 s) and watches again; the server
//! then sends the current state, so nothing is lost.

use std::cell::RefCell;
use std::rc::Rc;

use mimir_wire::{ClientMsg, ServerMsg};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{MessageEvent, WebSocket};

use crate::auth;

/// The socket address for this page: `ws(s)://<host>/ws`, with the token.
pub fn socket_url(protocol: &str, host: &str, token: Option<&str>) -> String {
    let scheme = if protocol == "https:" { "wss" } else { "ws" };
    match token {
        Some(t) => format!("{scheme}://{host}/ws?access_token={}", js_encode(t)),
        None => format!("{scheme}://{host}/ws"),
    }
}

fn js_encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// The wait before reconnect attempt `n` (from 0), in ms.
pub fn backoff_ms(n: u32) -> i32 {
    (1000_i32.saturating_mul(1 << n.min(4))).min(15_000)
}

/// A live connection. Dropping it closes the socket and stops reconnects.
pub struct Live {
    inner: Rc<RefCell<Inner>>,
}

struct Inner {
    socket: Option<WebSocket>,
    campaign_id: String,
    on_message: Rc<dyn Fn(ServerMsg)>,
    attempts: u32,
    stopped: bool,
    // Kept alive as long as the socket.
    handlers: Vec<Closure<dyn FnMut(web_sys::Event)>>,
}

impl Live {
    /// Connect and watch `campaign_id`.
    pub fn watch(campaign_id: String, on_message: impl Fn(ServerMsg) + 'static) -> Self {
        let inner = Rc::new(RefCell::new(Inner {
            socket: None,
            campaign_id,
            on_message: Rc::new(on_message),
            attempts: 0,
            stopped: false,
            handlers: Vec::new(),
        }));
        connect(&inner);
        Live { inner }
    }
}

impl Drop for Live {
    fn drop(&mut self) {
        // The page can drop this from inside a handler (a message that
        // leaves the page). Detach the handlers from the socket, close it,
        // and free the handlers on the next tick, not while one runs.
        let Ok(mut inner) = self.inner.try_borrow_mut() else {
            return;
        };
        inner.stopped = true;
        if let Some(s) = inner.socket.take() {
            s.set_onopen(None);
            s.set_onmessage(None);
            s.set_onclose(None);
            let _ = s.close();
        }
        let handlers = std::mem::take(&mut inner.handlers);
        let later = Closure::once_into_js(move || drop(handlers));
        if let Some(w) = web_sys::window() {
            let _ =
                w.set_timeout_with_callback_and_timeout_and_arguments_0(later.unchecked_ref(), 0);
        }
    }
}

fn connect(inner: &Rc<RefCell<Inner>>) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let location = window.location();
    let url = socket_url(
        &location.protocol().unwrap_or_default(),
        &location.host().unwrap_or_default(),
        auth::stored_token().as_deref(),
    );
    let Ok(socket) = WebSocket::new(&url) else {
        return;
    };

    let on_open = {
        let inner = Rc::clone(inner);
        Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
            let mut i = inner.borrow_mut();
            i.attempts = 0;
            let msg = ClientMsg::Watch {
                campaign_id: i.campaign_id.clone(),
            };
            if let (Some(s), Ok(text)) = (&i.socket, serde_json::to_string(&msg)) {
                let _ = s.send_with_str(&text);
            }
        })
    };
    let on_message = {
        let inner = Rc::clone(inner);
        Closure::<dyn FnMut(web_sys::Event)>::new(move |ev: web_sys::Event| {
            let Some(ev) = ev.dyn_ref::<MessageEvent>() else {
                return;
            };
            let Some(text) = ev.data().as_string() else {
                return;
            };
            if let Ok(msg) = serde_json::from_str::<ServerMsg>(&text) {
                let handler = Rc::clone(&inner.borrow().on_message);
                handler(msg);
            }
        })
    };
    let on_close = {
        let inner = Rc::clone(inner);
        Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
            let (stopped, wait) = {
                let mut i = inner.borrow_mut();
                i.socket = None;
                let wait = backoff_ms(i.attempts);
                i.attempts += 1;
                (i.stopped, wait)
            };
            if stopped {
                return;
            }
            let again = Rc::clone(&inner);
            let retry = Closure::once_into_js(move || {
                if !again.borrow().stopped {
                    connect(&again);
                }
            });
            if let Some(w) = web_sys::window() {
                let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(
                    retry.unchecked_ref(),
                    wait,
                );
            }
        })
    };
    socket.set_onopen(Some(on_open.as_ref().unchecked_ref()));
    socket.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    socket.set_onclose(Some(on_close.as_ref().unchecked_ref()));
    let mut i = inner.borrow_mut();
    i.socket = Some(socket);
    i.handlers = vec![on_open, on_message, on_close];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_socket_url_follows_the_page() {
        assert_eq!(
            socket_url("http:", "127.0.0.1:8740", None),
            "ws://127.0.0.1:8740/ws"
        );
        assert_eq!(
            socket_url("https:", "mimir.example", Some("a b+c")),
            "wss://mimir.example/ws?access_token=a%20b%2Bc"
        );
    }

    #[test]
    fn reconnects_back_off_to_15_s() {
        assert_eq!(backoff_ms(0), 1000);
        assert_eq!(backoff_ms(1), 2000);
        assert_eq!(backoff_ms(3), 8000);
        assert_eq!(backoff_ms(4), 15_000);
        assert_eq!(backoff_ms(40), 15_000);
    }
}
