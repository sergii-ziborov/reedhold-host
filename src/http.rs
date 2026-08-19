//! `tiny_http` loop. CORS is open so the Vite site can call localhost.

use crate::routes::dispatch;
use crate::state::State;
use std::sync::Mutex;
use tiny_http::{Header, Request, Response, Server, StatusCode};

/// Listen until the process is killed.
///
/// # Errors
///
/// Returns a display string when the port cannot be bound.
pub fn serve(bind: &str) -> Result<(), String> {
    let server = Server::http(bind).map_err(|error| error.to_string())?;
    eprintln!("reedhold-host listening on http://{bind}");
    let state = Mutex::new(State::default());
    for request in server.incoming_requests() {
        handle(&state, request);
    }
    Ok(())
}

fn handle(state: &Mutex<State>, mut request: Request) {
    let method = request.method().to_string();
    let url = request.url().to_owned();
    let mut raw = String::new();
    let _ = request.as_reader().read_to_string(&mut raw);
    let reply = dispatch(state, &method, &url, &raw);
    let mut response = Response::from_string(reply.body).with_status_code(StatusCode(reply.status));
    if let Ok(header) = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]) {
        response.add_header(header);
    }
    add_cors(&mut response);
    let _ = request.respond(response);
}

fn add_cors(response: &mut Response<std::io::Cursor<Vec<u8>>>) {
    if let Ok(header) = Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]) {
        response.add_header(header);
    }
    if let Ok(header) =
        Header::from_bytes(&b"Access-Control-Allow-Headers"[..], &b"Content-Type"[..])
    {
        response.add_header(header);
    }
    if let Ok(header) =
        Header::from_bytes(&b"Access-Control-Allow-Methods"[..], &b"GET, POST, OPTIONS"[..])
    {
        response.add_header(header);
    }
}
