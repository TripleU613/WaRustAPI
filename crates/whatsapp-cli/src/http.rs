//! Minimal HTTP control plane for local operation.
//!
//! By default this exposes only secret-free, offline-safe endpoints while the
//! Android primary-device profile is incomplete. Network mutations return
//! structured unsupported responses instead of bypassing evidence gates.
//!
//! `serve --store-routes` additionally serves a few read-only, redacted
//! inspections of one encrypted account store (the same library calls and
//! JSON as the CLI). They require a per-start bearer token that is written to
//! a new private file, are loopback-only, open the store per request (so other
//! CLI commands can use it in between) and never send anything.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use tokio::time::timeout;
use whatsapp_api::capabilities::CapabilityMatrix;
use whatsapp_api::model::ids::LocalMessageId;
use whatsapp_api::registration::wire::{
    COMPARISON_SIGNAL_KEY_TYPE, MAX_RESPONSE_BODY_BYTES, RegistrationEndpoint,
    RegistrationResponseReason, RegistrationResponseStatus, WHATSAPP_2_26_37_70,
};
use whatsapp_api::transport::comparison::{
    COMPARISON_CONNECTION_HEADER, COMPARISON_DICTIONARY_VERSION, COMPARISON_FALLBACK_PORT,
    COMPARISON_PRIMARY_PORTS, COMPARISON_PROTOCOL_VERSION, COMPARISON_PSEUDO_HTTP_CHUNK_BYTES,
    COMPARISON_PSEUDO_HTTP_LINE_BYTES, COMPARISON_PSEUDO_HTTP_REQUEST_HEAD, ComparisonEndpoint,
    ComparisonEndpointMode, ComparisonGateway, ComparisonSocketLayer,
};
use whatsapp_api::transport::selector::{
    COMPARISON_DEFAULT_PQ_MODE, COMPARISON_DEFAULT_PQ_VARIANT, ComparisonHandshakeKind,
    comparison_noise_pattern,
};
use whatsapp_api::{Error, ProtocolProfile};

const MAX_HEADER_BYTES: usize = 16 * 1024;
const MAX_CONNECTIONS: usize = 64;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const OPENAPI_JSON: &str = include_str!("../../../docs/api/openapi.json");

struct ApiState {
    profile: ProtocolProfile,
    capabilities: CapabilityMatrix,
    store: Option<StoreRoutes>,
}

/// Read-only store routes, each the same output as an existing CLI inspection
/// (`receipts`, `prekeys`, `outbox-suspended`, `actions` list,
/// `identity-conflicts`, `outbox-group-preparations`, `group own-lid`): counts,
/// local and wire message/iq IDs, unsalted address hashes (`peer_ticket`,
/// guessable for phone numbers), key fingerprints, fixed reasons and, for
/// refused actions, the server's error text. No message content, key material
/// or raw address. These take no query parameters.
const STORE_ROUTES: [&str; 7] = [
    "/v1/receipts",
    "/v1/prekeys",
    "/v1/outbox/suspended",
    "/v1/actions",
    "/v1/identity-conflicts",
    "/v1/outbox/group-preparations",
    "/v1/diagnostics/own-lid",
];

/// Store routes taking exactly one required query parameter, named as the
/// OpenAPI document names it: `group receipts`, `group key-queries` and
/// `group refresh-status`. Their output is counts, a fixed reason and, for
/// refresh status, the next action ID; no address. The value must already
/// satisfy the library's ID rule (ASCII alphanumerics and `-_.:`, at most 128
/// bytes), so no percent-decoding is done and an encoded value is refused.
const QUERY_ROUTES: [(&str, &str); 3] = [
    ("/v1/groups/receipts", "request_id"),
    ("/v1/groups/key-queries", "request_id"),
    ("/v1/groups/refresh-status", "id"),
];

/// One parsed store inspection, validated before the store opens. Without
/// experimental-transport the request is still parsed (so malformed input is
/// answered 400) but nothing reads the fields.
#[cfg_attr(not(feature = "experimental-transport"), allow(dead_code))]
enum Inspection {
    Plain(&'static str),
    Versions(LocalMessageId),
    WithId(&'static str, String),
}

/// `None` for a path that is not a store route; `Err` names what is wrong with
/// the request (answered 400 before the store opens).
fn inspection(path: &str, query: Option<&str>) -> Option<Result<Inspection, String>> {
    if let Some(route) = STORE_ROUTES.iter().find(|route| **route == path) {
        // CLI-only options (for example `actions --show`) are not served;
        // refuse rather than silently answering a different question.
        return Some(match query {
            None => Ok(Inspection::Plain(route)),
            Some(_) => Err("this store route takes no query parameters".into()),
        });
    }
    if let Some(&(route, name)) = QUERY_ROUTES.iter().find(|(route, _)| *route == path) {
        let value = query
            .and_then(|query| query.strip_prefix(name))
            .and_then(|rest| rest.strip_prefix('='));
        return Some(match value {
            Some(value) if whatsapp_api::model::ids::ClientRequestId::new(value).is_ok() => {
                Ok(Inspection::WithId(route, value.to_owned()))
            }
            _ => Err(format!(
                "exactly one query parameter {name}=<id> is required (ASCII alphanumerics and -_.: only, at most 128 bytes, not percent-encoded)"
            )),
        });
    }
    let id = versions_id(path)?;
    Some(match (query, id.parse::<LocalMessageId>()) {
        (Some(_), _) => Err("this store route takes no query parameters".into()),
        (None, Ok(id)) => Ok(Inspection::Versions(id)),
        (None, Err(_)) => Err("local_id is not a valid local message ID".into()),
    })
}

/// What `serve --store-routes` needs to open one account store.
pub struct StoreServe {
    /// Existing store only (`create` is forced off).
    pub options: whatsapp_api::store::OpenOptions,
    /// New private file that receives the bearer token; never overwritten.
    pub token_file: std::path::PathBuf,
}

struct StoreRoutes {
    options: whatsapp_api::store::OpenOptions,
    token: zeroize::Zeroizing<String>,
    /// The store file admits one open handle; requests take turns.
    lock: tokio::sync::Mutex<()>,
}

impl StoreRoutes {
    fn authorized(&self, header: Option<&str>) -> bool {
        use sha2::{Digest, Sha256};
        // RFC 7235: the scheme name is case-insensitive.
        let Some(presented) = header.and_then(|value| {
            let (scheme, token) = value.split_once(' ')?;
            scheme.eq_ignore_ascii_case("bearer").then_some(token)
        }) else {
            return false;
        };
        // Equal-length digests compared without an early exit.
        let presented = Sha256::digest(presented.trim().as_bytes());
        let expected = Sha256::digest(self.token.as_bytes());
        presented
            .iter()
            .zip(expected.iter())
            .fold(0u8, |acc, (a, b)| acc | (a ^ b))
            == 0
    }

    fn options(&self) -> whatsapp_api::store::OpenOptions {
        whatsapp_api::store::OpenOptions {
            create: false,
            ..self.options.clone()
        }
    }
}

struct RequestLine {
    method: String,
    path: String,
    authorization: Option<String>,
}

struct Response {
    status: u16,
    reason: &'static str,
    body: Vec<u8>,
    allow: Option<&'static str>,
    authenticate: bool,
}

impl Response {
    fn json(status: u16, reason: &'static str, body: Value) -> Self {
        Self {
            status,
            reason,
            body: serde_json::to_vec(&body).unwrap_or_else(|_| b"{}".to_vec()),
            allow: None,
            authenticate: false,
        }
    }

    fn error(status: u16, reason: &'static str, code: &str, message: &str) -> Self {
        Self::json(
            status,
            reason,
            json!({ "error": { "code": code, "message": message } }),
        )
    }

    fn method_not_allowed(allow: &'static str) -> Self {
        let mut response = Self::json(
            405,
            "Method Not Allowed",
            json!({
                "error": {
                    "code": "method_not_allowed",
                    "message": "the endpoint does not accept this HTTP method"
                }
            }),
        );
        response.allow = Some(allow);
        response
    }

    fn encode(&self, include_body: bool) -> Vec<u8> {
        let mut headers = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n",
            self.status,
            self.reason,
            self.body.len()
        );
        if let Some(allow) = self.allow {
            headers.push_str(&format!("Allow: {allow}\r\n"));
        }
        if self.authenticate {
            headers.push_str("WWW-Authenticate: Bearer\r\n");
        }
        headers.push_str("\r\n");
        let mut encoded = headers.into_bytes();
        if include_body {
            encoded.extend_from_slice(&self.body);
        }
        encoded
    }
}

pub async fn serve(
    bind: SocketAddr,
    allow_non_loopback: bool,
    store: Option<StoreServe>,
) -> Result<(), Error> {
    validate_bind(bind, allow_non_loopback)?;
    if store.is_some() && !bind.ip().is_loopback() {
        return Err(Error::invalid(
            "bind",
            "store routes serve decrypted account data and are loopback-only",
        ));
    }
    let store = match store {
        Some(store) => Some(open_store_routes(store)?),
        None => None,
    };

    let profile = ProtocolProfile::android_reference()?;
    let state = Arc::new(ApiState {
        capabilities: CapabilityMatrix::current(profile.id.clone()),
        profile,
        store: store.as_ref().map(|(routes, _)| StoreRoutes {
            options: routes.options(),
            token: routes.token.clone(),
            lock: tokio::sync::Mutex::new(()),
        }),
    });
    let listener = TcpListener::bind(bind).await?;
    let local_addr = listener.local_addr()?;
    let permits = Arc::new(Semaphore::new(MAX_CONNECTIONS));
    let shutdown = tokio::signal::ctrl_c();
    tokio::pin!(shutdown);
    // SIGTERM and SIGHUP also stop the server cleanly, so the token file guard
    // runs; SIGKILL or a crash can still leave the (then useless) file behind.
    let terminate = terminate_signal();
    tokio::pin!(terminate);

    println!("whatsapp-api HTTP control plane listening on http://{local_addr}");
    println!("profile: {} (offline gates enforced)", state.profile.id);
    if let Some((_, guard)) = &store {
        println!(
            "store routes enabled (read-only): bearer token written to {} (mode 0600, removed on Ctrl-C/SIGTERM/SIGHUP)",
            guard.0.display()
        );
    }

    loop {
        tokio::select! {
            signal = &mut shutdown => {
                signal?;
                println!("shutdown signal received");
                return Ok(());
            }
            () = &mut terminate => {
                println!("termination signal received");
                return Ok(());
            }
            accepted = listener.accept() => {
                let (stream, _peer) = accepted?;
                let Ok(permit) = Arc::clone(&permits).try_acquire_owned() else {
                    tracing::warn!(limit = MAX_CONNECTIONS, "HTTP connection limit reached");
                    drop(stream);
                    continue;
                };
                let state = Arc::clone(&state);
                tokio::spawn(async move {
                    let _permit = permit;
                    if let Err(error) = handle_connection(stream, &state).await {
                        tracing::warn!(error = %error, "HTTP request failed");
                    }
                });
            }
        }
    }
}

#[cfg(unix)]
async fn terminate_signal() {
    use tokio::signal::unix::{SignalKind, signal};
    match (
        signal(SignalKind::terminate()),
        signal(SignalKind::hangup()),
    ) {
        (Ok(mut term), Ok(mut hup)) => {
            tokio::select! {
                _ = term.recv() => {}
                _ = hup.recv() => {}
            }
        }
        _ => std::future::pending().await,
    }
}

#[cfg(not(unix))]
async fn terminate_signal() {
    std::future::pending().await
}

/// Removes the token file when the server stops, so a stale token is not left
/// readable after its server is gone.
#[derive(Debug)]
struct TokenFile(std::path::PathBuf);
impl Drop for TokenFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Check the store opens now (a clear error at startup, not per request), then
/// issue a fresh 256-bit token into a new private file. An existing file is
/// refused rather than overwritten.
fn open_store_routes(store: StoreServe) -> Result<(StoreRoutes, TokenFile), Error> {
    if !cfg!(feature = "experimental-transport") {
        return Err(Error::invalid(
            "store_routes",
            "store routes need a whatsapp-cli build with experimental-transport",
        ));
    }
    let routes = StoreRoutes {
        options: store.options,
        token: zeroize::Zeroizing::new(String::new()),
        lock: tokio::sync::Mutex::new(()),
    };
    drop(whatsapp_api::store::ProtocolStore::open(routes.options())?);
    let (token, guard) = issue_token(&store.token_file)?;
    Ok((StoreRoutes { token, ..routes }, guard))
}

fn issue_token(path: &std::path::Path) -> Result<(zeroize::Zeroizing<String>, TokenFile), Error> {
    use std::io::Write;
    let mut bytes = zeroize::Zeroizing::new([0u8; 32]);
    getrandom::fill(&mut *bytes)
        .map_err(|_| Error::invalid("store_routes", "token entropy unavailable"))?;
    let token = zeroize::Zeroizing::new(hex::encode(*bytes));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            Error::invalid("token_file", "already exists; choose a new path")
        } else {
            Error::from(error)
        }
    })?;
    // Guard first: a failed write must not leave a partial token behind.
    let guard = TokenFile(path.to_owned());
    file.write_all(token.as_bytes())?;
    file.sync_all()?;
    Ok((token, guard))
}

fn validate_bind(bind: SocketAddr, allow_non_loopback: bool) -> Result<(), Error> {
    if bind.ip().is_loopback() || allow_non_loopback {
        return Ok(());
    }
    Err(Error::invalid(
        "bind",
        "non-loopback listeners require --allow-non-loopback because the base routes are unauthenticated",
    ))
}

async fn handle_connection(mut stream: TcpStream, state: &ApiState) -> Result<(), Error> {
    let request = timeout(REQUEST_TIMEOUT, read_request(&mut stream))
        .await
        .map_err(|_| Error::Timeout(REQUEST_TIMEOUT))??;
    let response = match request.as_ref() {
        Some(request) => match store_route(request, state).await {
            Some(response) => response,
            None => route(&request.method, &request.path, state),
        },
        None => Response::json(
            400,
            "Bad Request",
            json!({
                "error": {
                    "code": "bad_request",
                    "message": "a valid HTTP/1.0 or HTTP/1.1 request line is required"
                }
            }),
        ),
    };
    let include_body = request
        .as_ref()
        .is_none_or(|request| request.method != "HEAD");
    timeout(
        REQUEST_TIMEOUT,
        stream.write_all(&response.encode(include_body)),
    )
    .await
    .map_err(|_| Error::Timeout(REQUEST_TIMEOUT))??;
    Ok(())
}

async fn read_request(stream: &mut TcpStream) -> Result<Option<RequestLine>, Error> {
    let mut bytes = Vec::with_capacity(1024);
    loop {
        if bytes.len() == MAX_HEADER_BYTES {
            return Ok(None);
        }
        let remaining = MAX_HEADER_BYTES - bytes.len();
        let mut chunk = [0u8; 1024];
        let read_len = remaining.min(chunk.len());
        let count = stream.read(&mut chunk[..read_len]).await?;
        if count == 0 {
            return Ok(None);
        }
        bytes.extend_from_slice(&chunk[..count]);
        if bytes.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
    }

    let Some(line_end) = bytes.windows(2).position(|window| window == b"\r\n") else {
        return Ok(None);
    };
    let Ok(line) = std::str::from_utf8(&bytes[..line_end]) else {
        return Ok(None);
    };
    let mut parts = line.split_ascii_whitespace();
    let (Some(method), Some(path), Some(version), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Ok(None);
    };
    if !matches!(version, "HTTP/1.0" | "HTTP/1.1") || !path.starts_with('/') {
        return Ok(None);
    }
    let Some(head_end) = bytes.windows(4).position(|window| window == b"\r\n\r\n") else {
        return Ok(None);
    };
    // A request with no headers has its blank line right after the request
    // line, so the header block is empty (head_end == line_end).
    let authorization = bytes
        .get(line_end + 2..head_end)
        .and_then(|headers| std::str::from_utf8(headers).ok())
        .and_then(|headers| {
            headers.split("\r\n").find_map(|header| {
                let (name, value) = header.split_once(':')?;
                name.trim()
                    .eq_ignore_ascii_case("authorization")
                    .then(|| value.trim().to_owned())
            })
        });

    Ok(Some(RequestLine {
        method: method.to_owned(),
        path: path.to_owned(),
        authorization,
    }))
}

/// Store routes, or `None` for every other path. Disabled routes answer 404,
/// a missing or wrong token 401, and a store that cannot be opened (for
/// example while `listen` holds it) 503 with a fixed message.
async fn store_route(request: &RequestLine, state: &ApiState) -> Option<Response> {
    let (path, query) = request
        .path
        .split_once('?')
        .map_or((request.path.as_str(), None), |(path, query)| {
            (path, Some(query))
        });
    let parsed = inspection(path, query)?;
    if !matches!(request.method.as_str(), "GET" | "HEAD") {
        return Some(Response::method_not_allowed("GET, HEAD"));
    }
    let inspection = match parsed {
        Ok(inspection) => inspection,
        Err(message) => {
            return Some(Response::error(
                400,
                "Bad Request",
                "invalid_input",
                &message,
            ));
        }
    };
    let Some(store) = &state.store else {
        return Some(Response::error(
            404,
            "Not Found",
            "store_routes_disabled",
            "start serve with --store-routes --token-file to enable store inspections",
        ));
    };
    if !store.authorized(request.authorization.as_deref()) {
        let mut response = Response::error(
            401,
            "Unauthorized",
            "unauthorized",
            "send Authorization: Bearer <token from the serve token file>",
        );
        response.authenticate = true;
        return Some(response);
    }
    let _turn = store.lock.lock().await;
    let options = store.options();
    let result = tokio::task::spawn_blocking(move || read_store(options, inspection)).await;
    Some(match result {
        Ok(Ok(Some(body))) => Response::json(200, "OK", body),
        Ok(Ok(None)) => Response::error(
            404,
            "Not Found",
            "unknown_local_id",
            "no outbox intent has this local_id",
        ),
        Ok(Err(Error::Storage(whatsapp_api::store::StoreError::AlreadyOwned { .. }))) => {
            Response::error(
                503,
                "Service Unavailable",
                "store_unavailable",
                "the account store is in use by another command; retry when it finishes",
            )
        }
        Ok(Err(Error::InvalidInput {
            field: SAVED_STATE_REFUSED,
            reason,
        })) => Response::error(
            422,
            "Unprocessable Content",
            "saved_state_unsupported",
            &reason,
        ),
        Ok(Err(error)) => {
            // Not a busy store: say so instead of suggesting a retry. Details
            // (which may name paths) stay in the server log.
            tracing::warn!(error = %error, "store route failed");
            Response::error(
                500,
                "Internal Server Error",
                "store_read_failed",
                "the account store could not be read; see the serve log",
            )
        }
        Err(_) => Response::error(
            500,
            "Internal Server Error",
            "internal",
            "store inspection stopped unexpectedly",
        ),
    })
}

/// `/v1/outbox/{local_id}/versions` → the raw `local_id` segment.
fn versions_id(path: &str) -> Option<&str> {
    path.strip_prefix("/v1/outbox/")?
        .strip_suffix("/versions")
        .filter(|id| !id.is_empty() && !id.contains('/'))
}

/// A saved record an inspection refuses to interpret, answered 422 with the
/// library's reason. Only inspections whose refusals are fixed strings are
/// converted (see `saved_state`); any other error stays a generic 500.
const SAVED_STATE_REFUSED: &str = "store_routes.saved_state";

/// Mark a refusal as [`SAVED_STATE_REFUSED`]. Callers: `device_key_batches`
/// (a legacy key outcome without a frozen plan, or a damaged row),
/// `group_receipts` (a damaged saved send, target list or facts) and the own
/// LID lookup (its sources disagree or are malformed). All three refuse only
/// with `&'static str` reasons, enforced by their helpers' signatures, and no
/// address. Storage errors pass through unchanged.
#[cfg(feature = "experimental-transport")]
fn saved_state(error: Error) -> Error {
    match error {
        Error::InvalidInput { reason, .. } => Error::invalid(SAVED_STATE_REFUSED, reason),
        other => other,
    }
}

/// `Ok(None)` when a path parameter names nothing (404). A query-parameter
/// lookup that finds nothing answers JSON `null`, as the CLI prints.
#[cfg(feature = "experimental-transport")]
fn read_store(
    options: whatsapp_api::store::OpenOptions,
    inspection: Inspection,
) -> Result<Option<Value>, Error> {
    use whatsapp_api::model::ids::ClientRequestId;
    use whatsapp_api::transport::{
        actions, device_key_batches, group_receipts, group_send, identity_conflicts, outgoing,
        outgoing_group, prekeys, receipts,
    };
    let store = whatsapp_api::store::ProtocolStore::open(options)?;
    let path = match inspection {
        Inspection::Plain(path) => path,
        Inspection::Versions(id) => return versions(&store, id),
        Inspection::WithId(path, id) => {
            let value = match path {
                "/v1/groups/receipts" => serde_json::to_value(
                    group_receipts::summary(&store, &ClientRequestId::new(id)?)
                        .map_err(saved_state)?,
                ),
                "/v1/groups/key-queries" => serde_json::to_value(
                    device_key_batches::summary(&store, &ClientRequestId::new(id)?)
                        .map_err(saved_state)?,
                ),
                "/v1/groups/refresh-status" => {
                    Ok(crate::transport_receive::refresh_status(&store, &id)?)
                }
                _ => return Err(Error::invalid("store_routes", "unknown store route")),
            };
            return value
                .map(Some)
                .map_err(|_| Error::invalid("store_routes", "inspection did not serialize"));
        }
    };
    let value = match path {
        "/v1/receipts" => serde_json::to_value(receipts::outcomes(&store)?),
        "/v1/prekeys" => serde_json::to_value(prekeys::summary(&store)?),
        "/v1/outbox/suspended" => serde_json::to_value(outgoing::identity_suspended(&store)?),
        "/v1/actions" => serde_json::to_value(actions::outcomes(&store)?),
        "/v1/identity-conflicts" => serde_json::to_value(identity_conflicts::list(&store)?),
        "/v1/outbox/group-preparations" => {
            serde_json::to_value(outgoing_group::preparations(&store)?)
        }
        "/v1/diagnostics/own-lid" => Ok(json!({
            "lid_known": group_send::own_lid(&store).map_err(saved_state)?.is_some()
        })),
        _ => return Err(Error::invalid("store_routes", "unknown store route")),
    };
    value
        .map(Some)
        .map_err(|_| Error::invalid("store_routes", "inspection did not serialize"))
}

/// `/v1/outbox/{local_id}/versions`; `Ok(None)` for an unknown intent.
#[cfg(feature = "experimental-transport")]
fn versions(
    store: &whatsapp_api::store::ProtocolStore,
    id: LocalMessageId,
) -> Result<Option<Value>, Error> {
    use whatsapp_api::transport::send_versions;
    let known = store.read(|tx| {
        whatsapp_api::outbox::Outbox::new("http-inspection", Default::default()).get(tx, id)
    })?;
    if known.is_none() {
        return Ok(None);
    }
    serde_json::to_value(send_versions::inspect(store, id)?)
        .map(Some)
        .map_err(|_| Error::invalid("store_routes", "inspection did not serialize"))
}

#[cfg(not(feature = "experimental-transport"))]
fn read_store(_: whatsapp_api::store::OpenOptions, _: Inspection) -> Result<Option<Value>, Error> {
    Err(Error::invalid(
        "store_routes",
        "store routes need a whatsapp-cli build with experimental-transport",
    ))
}

fn route(method: &str, raw_path: &str, state: &ApiState) -> Response {
    let path = raw_path.split_once('?').map_or(raw_path, |(path, _)| path);
    let is_read = matches!(method, "GET" | "HEAD");

    match (method, path) {
        (
            _,
            "/"
            | "/healthz"
            | "/openapi.json"
            | "/v1/profile"
            | "/v1/capabilities"
            | "/v1/transport/wire-profile"
            | "/v1/registration/wire-profile",
        ) if !is_read => Response::method_not_allowed("GET, HEAD"),
        (_, "/v1/connect" | "/v1/registration") if method != "POST" => {
            Response::method_not_allowed("POST")
        }
        ("GET" | "HEAD", "/") => {
            let mut routes = vec![
                "GET /healthz".to_owned(),
                "GET /openapi.json".to_owned(),
                "GET /v1/profile".to_owned(),
                "GET /v1/capabilities".to_owned(),
                "GET /v1/transport/wire-profile".to_owned(),
                "GET /v1/registration/wire-profile".to_owned(),
                "POST /v1/connect".to_owned(),
                "POST /v1/registration".to_owned(),
            ];
            // Listed only when enabled; each needs the bearer token.
            if state.store.is_some() {
                routes.extend(STORE_ROUTES.iter().map(|path| format!("GET {path}")));
                routes.extend(
                    QUERY_ROUTES
                        .iter()
                        .map(|(path, name)| format!("GET {path}?{name}={{{name}}}")),
                );
                routes.push("GET /v1/outbox/{local_id}/versions".to_owned());
            }
            Response::json(
                200,
                "OK",
                json!({
                    "service": "whatsapp-api",
                    "version": env!("CARGO_PKG_VERSION"),
                    "routes": routes,
                }),
            )
        }
        ("GET" | "HEAD", "/healthz") => Response::json(
            200,
            "OK",
            json!({
                "status": "ok",
                "service": "whatsapp-api",
                "version": env!("CARGO_PKG_VERSION"),
                "profile_id": state.profile.id,
                "production_grade": state.profile.is_production_grade()
            }),
        ),
        ("GET" | "HEAD", "/openapi.json") => Response::json(
            200,
            "OK",
            serde_json::from_str(OPENAPI_JSON).expect("docs/api/openapi.json is valid JSON"),
        ),
        ("GET" | "HEAD", "/v1/profile") => Response::json(
            200,
            "OK",
            json!({
                "profile": state.profile,
                "production_grade": state.profile.is_production_grade(),
                "missing_for_connect": state.profile.missing_for_connect(),
                "missing_for_registration": state.profile.missing_for_registration()
            }),
        ),
        ("GET" | "HEAD", "/v1/capabilities") => {
            Response::json(200, "OK", json!(state.capabilities))
        }
        ("GET" | "HEAD", "/v1/transport/wire-profile") => Response::json(
            200,
            "OK",
            json!({
                "evidence_scope": "signed newer-build comparison; not active profile",
                "version_name": "2.26.37.70",
                "connection_header": {
                    "hex": COMPARISON_CONNECTION_HEADER
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect::<String>(),
                    "protocol_version": COMPARISON_PROTOCOL_VERSION,
                    "dictionary_version": COMPARISON_DICTIONARY_VERSION
                },
                "ports": {
                    "primary_candidates": COMPARISON_PRIMARY_PORTS,
                    "fallback": COMPARISON_FALLBACK_PORT
                },
                "built_in_socket_layers": [
                    comparison_endpoint_json(ComparisonGateway::Primary, 443, "g.whatsapp.net"),
                    comparison_endpoint_json(ComparisonGateway::Primary, 5222, "g.whatsapp.net"),
                    comparison_endpoint_json(ComparisonGateway::Primary, 80, "g.whatsapp.net"),
                    comparison_endpoint_json(
                        ComparisonGateway::Fallback,
                        443,
                        "g-fallback.whatsapp.net"
                    ),
                    comparison_endpoint_json(
                        ComparisonGateway::Fallback,
                        5222,
                        "g-fallback.whatsapp.net"
                    ),
                    comparison_endpoint_json(
                        ComparisonGateway::Fallback,
                        80,
                        "g-fallback.whatsapp.net"
                    ),
                    comparison_endpoint_json(
                        ComparisonGateway::Edge(1),
                        443,
                        "e1-e16.whatsapp.net"
                    ),
                    comparison_endpoint_json(
                        ComparisonGateway::Edge(1),
                        5222,
                        "e1-e16.whatsapp.net"
                    ),
                    comparison_endpoint_json(
                        ComparisonGateway::Edge(1),
                        80,
                        "e1-e16.whatsapp.net"
                    )
                ],
                "port80_pseudo_http": {
                    "request_head": String::from_utf8_lossy(
                        COMPARISON_PSEUDO_HTTP_REQUEST_HEAD
                    ),
                    "output_chunk_bytes": COMPARISON_PSEUDO_HTTP_CHUNK_BYTES,
                    "input_line_limit_bytes": COMPARISON_PSEUDO_HTTP_LINE_BYTES,
                    "response_transfer_encoding": "chunked"
                },
                "noise_selector": {
                    "compiled_pq_mode": COMPARISON_DEFAULT_PQ_MODE,
                    "compiled_variant": COMPARISON_DEFAULT_PQ_VARIANT,
                    "active_remote_configuration": null,
                    "compiled_default_patterns": {
                        "full": comparison_noise_pattern(
                            COMPARISON_DEFAULT_PQ_MODE,
                            COMPARISON_DEFAULT_PQ_VARIANT,
                            ComparisonHandshakeKind::Full
                        ),
                        "resume": comparison_noise_pattern(
                            COMPARISON_DEFAULT_PQ_MODE,
                            COMPARISON_DEFAULT_PQ_VARIANT,
                            ComparisonHandshakeKind::Resume
                        ),
                        "resume_fallback": comparison_noise_pattern(
                            COMPARISON_DEFAULT_PQ_MODE,
                            COMPARISON_DEFAULT_PQ_VARIANT,
                            ComparisonHandshakeKind::ResumeFallback
                        )
                    }
                },
                "unresolved": [
                    "active remote configuration",
                    "pinned-profile equivalence",
                    "primary-device login payload"
                ]
            }),
        ),
        ("GET" | "HEAD", "/v1/registration/wire-profile") => Response::json(
            200,
            "OK",
            json!({
                "evidence_scope": "signed newer-build comparison; not active profile",
                "wire_profile": WHATSAPP_2_26_37_70,
                "endpoints": RegistrationEndpoint::ALL
                    .iter()
                    .map(|endpoint| json!({
                        "name": endpoint,
                        "path": endpoint.path(),
                        "url": WHATSAPP_2_26_37_70.endpoint_url(*endpoint)
                    }))
                    .collect::<Vec<_>>(),
                "response_schema": {
                    "encoding": "json",
                    "max_body_bytes": MAX_RESPONSE_BODY_BYTES,
                    "statuses": RegistrationResponseStatus::OBSERVED
                        .iter()
                        .filter_map(|status| status.wire_value())
                        .collect::<Vec<_>>(),
                    "reasons": RegistrationResponseReason::OBSERVED
                        .iter()
                        .filter_map(|reason| reason.wire_value())
                        .collect::<Vec<_>>()
                },
                "common_fields": {
                    "groups": [
                        ["cc", "in"],
                        ["lg", "lc", "fdid", "expid"],
                        ["access_session_id", "id", "backup_token"]
                    ],
                    "uuid_encoding": "base64url-no-padding",
                    "binary_encoding": "rfc3986-percent-encoded-pre-encoded-value",
                    "fresh_fallbacks": {
                        "fdid": "persisted-uuid-v4-string",
                        "expid": "persisted-uuid-v4-bytes",
                        "access_session_id": "persisted-uuid-v4-bytes",
                        "id_bytes": 20,
                        "backup_token_bytes": 20
                    }
                },
                "code_request": {
                    "path": RegistrationEndpoint::Code.path(),
                    "required_fields": ["token", "method"],
                    "token_derivation": {
                        "scope": "whatsapp-2.26.37.70-comparison-only",
                        "kdf": "PBKDF2-HMAC-SHA1-128x-512bit",
                        "mac_inputs": [
                            "signing-certificate-der", "md5-classes.dex",
                            "national-number"
                        ],
                        "output": "standard-base64-padded-no-wrap"
                    },
                    "optional_fields": [
                        "context", "clicked_education_link", "manage_call_permission",
                        "call_log_permission", "client_start_message", "advertising_id",
                        "login", "type"
                    ],
                    "boolean_encoding": "true-false-or-omitted",
                    "client_start_message_encoding": "base64url-no-padding"
                },
                "register_request": {
                    "path": RegistrationEndpoint::Register.path(),
                    "required_fields": ["code"],
                    "optional_fields": [
                        "auth_response", "context", "method", "advertising_id",
                        "login", "type"
                    ],
                    "auth_response_encoding": "base64url-no-padding"
                },
                "security_request": {
                    "path": RegistrationEndpoint::Security.path(),
                    "optional_fields": ["code", "reset", "wipe_token", "advertising_id"],
                    "wipe_token_encoding": "rfc3986-percent-encoded-pre-encoded-value"
                },
                "key_bundle": {
                    "encoding": "base64url-no-padding",
                    "key_type_hex": format!("{COMPARISON_SIGNAL_KEY_TYPE:02x}"),
                    "registration_id": "u32-big-endian",
                    "signed_pre_key_id": "u24-big-endian",
                    "fields": [
                        "authkey", "e_ident", "e_keytype", "e_regid",
                        "e_skey_id", "e_skey_val", "e_skey_sig"
                    ]
                }
            }),
        ),
        ("POST", "/v1/connect") => Response::json(
            501,
            "Not Implemented",
            json!({
                "error": {
                    "code": "profile_incomplete",
                    "message": "authenticated WhatsApp transport is not implemented for this profile",
                    "profile_id": state.profile.id,
                    "missing": state.profile.missing_for_connect()
                }
            }),
        ),
        ("POST", "/v1/registration") => Response::json(
            501,
            "Not Implemented",
            json!({
                "error": {
                    "code": "profile_incomplete",
                    "message": "primary-device registration is not implemented for this profile",
                    "profile_id": state.profile.id,
                    "missing": state.profile.missing_for_registration()
                }
            }),
        ),
        _ => Response::json(
            404,
            "Not Found",
            json!({
                "error": {
                    "code": "not_found",
                    "message": "no route matches this request"
                }
            }),
        ),
    }
}

fn comparison_endpoint_json(gateway: ComparisonGateway, port: u16, host: &str) -> Value {
    let endpoint = ComparisonEndpoint::built_in(gateway, port)
        .expect("hard-coded comparison endpoint must be valid");
    json!({
        "host": host,
        "port": endpoint.port,
        "layer": match endpoint.layer {
            ComparisonSocketLayer::RawTcp => "raw_tcp",
            ComparisonSocketLayer::Tls => "tls",
        },
        "mode": match endpoint.mode {
            ComparisonEndpointMode::Direct => "direct",
            ComparisonEndpointMode::PseudoHttpChunked => "pseudo_http_chunked",
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> ApiState {
        let profile = ProtocolProfile::android_reference().unwrap();
        ApiState {
            capabilities: CapabilityMatrix::current(profile.id.clone()),
            profile,
            store: None,
        }
    }

    fn body(response: &Response) -> Value {
        serde_json::from_slice(&response.body).unwrap()
    }

    #[test]
    fn health_is_available_without_a_store() {
        let response = route("GET", "/healthz", &state());
        assert_eq!(response.status, 200);
        assert_eq!(body(&response)["status"], "ok");
        assert_eq!(body(&response)["production_grade"], false);
    }

    #[test]
    fn profile_reports_missing_evidence() {
        let response = route("GET", "/v1/profile", &state());
        let body = body(&response);
        assert_eq!(response.status, 200);
        assert!(
            body["missing_for_connect"]
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item == "login.client_payload")
        );
    }

    #[test]
    fn network_mutations_fail_honestly() {
        let state = state();
        let connect = route("POST", "/v1/connect", &state);
        let registration = route("POST", "/v1/registration", &state);
        assert_eq!(connect.status, 501);
        assert_eq!(registration.status, 501);
        assert_eq!(body(&connect)["error"]["code"], "profile_incomplete");
    }

    #[test]
    fn exposes_version_scoped_registration_wire_evidence() {
        let response = route("GET", "/v1/registration/wire-profile", &state());
        let body = body(&response);
        assert_eq!(response.status, 200);
        assert_eq!(body["wire_profile"]["version_name"], "2.26.37.70");
        assert_eq!(body["endpoints"].as_array().unwrap().len(), 17);
        assert_eq!(body["response_schema"]["encoding"], "json");
        assert_eq!(body["response_schema"]["statuses"][1], "sent");
        assert_eq!(
            body["response_schema"]["reasons"].as_array().unwrap().len(),
            37
        );
        assert_eq!(body["key_bundle"]["key_type_hex"], "05");
        assert_eq!(body["key_bundle"]["fields"].as_array().unwrap().len(), 7);
        assert_eq!(body["common_fields"]["groups"][0][0], "cc");
        assert_eq!(body["common_fields"]["groups"][2][2], "backup_token");
        assert_eq!(body["common_fields"]["fresh_fallbacks"]["id_bytes"], 20);
        assert_eq!(
            body["code_request"]["token_derivation"]["scope"],
            "whatsapp-2.26.37.70-comparison-only"
        );
        assert_eq!(body["code_request"]["path"], "/v2/code");
        assert_eq!(body["code_request"]["required_fields"][1], "method");
        assert_eq!(body["register_request"]["path"], "/v2/register");
        assert_eq!(body["register_request"]["required_fields"][0], "code");
        assert_eq!(body["security_request"]["path"], "/v2/security");
        assert_eq!(body["security_request"]["optional_fields"][2], "wipe_token");
        assert_eq!(
            body["endpoints"][2]["url"],
            "https://v.whatsapp.net/v2/code"
        );
    }

    #[test]
    fn exposes_version_scoped_transport_wire_evidence() {
        let response = route("GET", "/v1/transport/wire-profile", &state());
        let body = body(&response);
        assert_eq!(response.status, 200);
        assert_eq!(body["version_name"], "2.26.37.70");
        assert_eq!(body["connection_header"]["hex"], "57410603");
        assert_eq!(body["ports"]["primary_candidates"][1], 5222);
        assert_eq!(body["built_in_socket_layers"][0]["layer"], "raw_tcp");
        assert_eq!(body["built_in_socket_layers"][6]["layer"], "tls");
        assert_eq!(
            body["built_in_socket_layers"][8]["mode"],
            "pseudo_http_chunked"
        );
        assert_eq!(body["port80_pseudo_http"]["output_chunk_bytes"], 8192);
        assert!(
            body["port80_pseudo_http"]["request_head"]
                .as_str()
                .unwrap()
                .starts_with("POST /chat HTTP/1.1\r\n")
        );
        assert_eq!(body["noise_selector"]["compiled_pq_mode"], "none");
        assert_eq!(
            body["noise_selector"]["compiled_default_patterns"]["full"],
            "Noise_XX_25519_AESGCM_SHA256"
        );
        assert!(body["noise_selector"]["active_remote_configuration"].is_null());
    }

    #[test]
    fn methods_and_unknown_routes_are_bounded() {
        let state = state();
        let wrong_method = route("POST", "/healthz", &state);
        let missing = route("GET", "/not-here?ignored=true", &state);
        assert_eq!(wrong_method.status, 405);
        assert_eq!(wrong_method.allow, Some("GET, HEAD"));
        assert_eq!(missing.status, 404);
    }

    #[test]
    fn head_omits_body_but_preserves_content_length() {
        let response = route("HEAD", "/healthz", &state());
        let encoded = response.encode(false);
        assert!(encoded.ends_with(b"\r\n\r\n"));
        assert!(
            String::from_utf8(encoded)
                .unwrap()
                .contains("Content-Length:")
        );
    }

    #[test]
    fn non_loopback_bind_requires_explicit_opt_in() {
        let public = "0.0.0.0:8787".parse().unwrap();
        assert!(validate_bind(public, false).is_err());
        assert!(validate_bind(public, true).is_ok());
        assert!(validate_bind("127.0.0.1:8787".parse().unwrap(), false).is_ok());
    }

    #[test]
    fn openapi_document_is_served() {
        let response = route("GET", "/openapi.json", &state());
        assert_eq!(response.status, 200);
        let body = body(&response);
        assert_eq!(body["openapi"], "3.1.0");
        assert!(body["paths"].is_object());
        assert!(!body["paths"].as_object().unwrap().is_empty());
    }

    fn request(method: &str, path: &str, authorization: Option<&str>) -> RequestLine {
        RequestLine {
            method: method.into(),
            path: path.into(),
            authorization: authorization.map(str::to_owned),
        }
    }

    #[tokio::test]
    async fn store_routes_are_off_unless_enabled_and_never_touch_other_paths() {
        let state = state();
        let response = store_route(&request("GET", "/v1/receipts", None), &state)
            .await
            .unwrap();
        assert_eq!(response.status, 404);
        assert_eq!(body(&response)["error"]["code"], "store_routes_disabled");
        let query = store_route(
            &request("GET", "/v1/groups/receipts?request_id=a", None),
            &state,
        )
        .await
        .unwrap();
        assert_eq!(query.status, 404);
        assert_eq!(body(&query)["error"]["code"], "store_routes_disabled");
        assert!(
            store_route(&request("GET", "/v1/profile", None), &state)
                .await
                .is_none()
        );
    }

    #[cfg(feature = "experimental-transport")]
    #[tokio::test]
    async fn store_routes_require_the_token_and_serve_redacted_inspections() {
        use whatsapp_api::store::{MasterKey, OpenOptions, ProtocolStore, StaticKeySource};
        let dir = tempfile::tempdir().unwrap();
        let options = OpenOptions {
            path: dir.path().join("served.redb"),
            account_id: whatsapp_api::model::ids::AccountId::new("served").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([3; 32]))),
            create: true,
        };
        let known = {
            use whatsapp_api::model::{
                ids::{ChatId, ClientRequestId},
                jid::Jid,
                message::MessageContent,
                send::SendRequest,
            };
            let store = ProtocolStore::open(options.clone()).unwrap();
            let request = SendRequest {
                client_request_id: ClientRequestId::new("served-intent").unwrap(),
                chat: ChatId::from_jid(&Jid::parse("15550000000@s.whatsapp.net").unwrap()).unwrap(),
                content: MessageContent::Text {
                    body: "never sent".into(),
                    quoted: None,
                    style: Default::default(),
                },
            };
            store
                .write(|tx| {
                    whatsapp_api::outbox::Outbox::new("http-test", Default::default())
                        .enqueue(tx, &request)
                })
                .unwrap()
                .0
                .local_id
        };
        let mut state = state();
        state.store = Some(StoreRoutes {
            options,
            token: zeroize::Zeroizing::new("secret-token".into()),
            lock: tokio::sync::Mutex::new(()),
        });
        for authorization in [
            None,
            Some("Bearer wrong"),
            Some("secret-token"),
            Some("Basic secret-token"),
        ] {
            let response = store_route(&request("GET", "/v1/receipts", authorization), &state)
                .await
                .unwrap();
            assert_eq!(response.status, 401, "{authorization:?}");
            assert!(response.authenticate);
            assert!(!String::from_utf8_lossy(&response.body).contains("secret-token"));
        }
        // The scheme name is case-insensitive (RFC 7235).
        let lower = store_route(
            &request("GET", "/v1/receipts", Some("bearer secret-token")),
            &state,
        )
        .await
        .unwrap();
        assert_eq!(lower.status, 200);
        let auth = Some("Bearer secret-token");
        let post = store_route(&request("POST", "/v1/prekeys", auth), &state)
            .await
            .unwrap();
        assert_eq!(post.status, 405);
        let receipts = store_route(&request("GET", "/v1/receipts", auth), &state)
            .await
            .unwrap();
        assert_eq!((receipts.status, body(&receipts)), (200, json!([])));
        let query = store_route(&request("GET", "/v1/actions?show=id", auth), &state)
            .await
            .unwrap();
        assert_eq!(query.status, 400);
        let prekeys = store_route(&request("GET", "/v1/prekeys", auth), &state)
            .await
            .unwrap();
        assert_eq!(prekeys.status, 200);
        for field in ["unsent", "uncertain", "uploaded", "batches", "kem_local"] {
            assert!(body(&prekeys).get(field).is_some(), "{field}");
        }
        for path in [
            "/v1/outbox/suspended",
            "/v1/actions",
            "/v1/identity-conflicts",
            "/v1/outbox/group-preparations",
        ] {
            let listed = store_route(&request("GET", path, auth), &state)
                .await
                .unwrap();
            assert_eq!((listed.status, body(&listed)), (200, json!([])), "{path}");
        }
        // Path parameter: a known intent answers its redacted snapshot (an
        // unprepared intent has none), malformed is 400, unknown is 404.
        let versions = store_route(
            &request("GET", &format!("/v1/outbox/{known}/versions"), auth),
            &state,
        )
        .await
        .unwrap();
        assert_eq!(versions.status, 200);
        assert_eq!(body(&versions)["captured"], false);
        assert!(!String::from_utf8_lossy(&versions.body).contains("never sent"));
        let malformed = store_route(
            &request("GET", "/v1/outbox/not-an-id/versions", auth),
            &state,
        )
        .await
        .unwrap();
        assert_eq!(malformed.status, 400);
        let unknown_id = LocalMessageId([7; 16]).to_string();
        let unknown = store_route(
            &request("GET", &format!("/v1/outbox/{unknown_id}/versions"), auth),
            &state,
        )
        .await
        .unwrap();
        assert_eq!(unknown.status, 404);
        assert_eq!(body(&unknown)["error"]["code"], "unknown_local_id");
        let nested = store_route(
            &request("GET", &format!("/v1/outbox/{unknown_id}/x/versions"), auth),
            &state,
        )
        .await;
        assert!(nested.is_none(), "only one path segment is a local_id");
        // Counts-only group inspections and own-LID availability. Nothing saved
        // under an ID answers JSON null, as the CLI prints.
        let own_lid = store_route(&request("GET", "/v1/diagnostics/own-lid", auth), &state)
            .await
            .unwrap();
        assert_eq!(
            (own_lid.status, body(&own_lid)),
            (200, json!({"lid_known": false}))
        );
        for path in [
            "/v1/groups/receipts?request_id=nothing-saved",
            "/v1/groups/key-queries?request_id=nothing-saved",
            "/v1/groups/refresh-status?id=NOTHING",
        ] {
            let empty = store_route(&request("GET", path, auth), &state)
                .await
                .unwrap();
            assert_eq!((empty.status, body(&empty)), (200, Value::Null), "{path}");
        }
        // Exactly one parameter with the documented name and an ID-shaped
        // value; anything else is refused before the store opens.
        for path in [
            "/v1/groups/receipts",
            "/v1/groups/receipts?",
            "/v1/groups/receipts?id=wrong-name",
            "/v1/groups/receipts?request_id=",
            "/v1/groups/receipts?request_id=a&request_id=b",
            "/v1/groups/receipts?request_id=a%20b",
            "/v1/groups/key-queries?request_idx=a",
            "/v1/groups/refresh-status?request_id=a",
            "/v1/diagnostics/own-lid?verbose=1",
            "/v1/outbox/not-an-id/versions?x=1",
        ] {
            let refused = store_route(&request("GET", path, auth), &state)
                .await
                .unwrap();
            assert_eq!(refused.status, 400, "{path}");
            assert_eq!(body(&refused)["error"]["code"], "invalid_input", "{path}");
        }
        // The library ID rule's 128-byte bound is the boundary.
        let longest = format!("/v1/groups/receipts?request_id={}", "a".repeat(128));
        let longest = store_route(&request("GET", &longest, auth), &state)
            .await
            .unwrap();
        assert_eq!((longest.status, body(&longest)), (200, Value::Null));
        let too_long = format!("/v1/groups/receipts?request_id={}", "a".repeat(129));
        let too_long = store_route(&request("GET", &too_long, auth), &state)
            .await
            .unwrap();
        assert_eq!(too_long.status, 400);
        // A query on the path-parameter route is refused even with a valid ID.
        let versions_query = store_route(
            &request("GET", &format!("/v1/outbox/{known}/versions?x=1"), auth),
            &state,
        )
        .await
        .unwrap();
        assert_eq!(versions_query.status, 400);
        let post = store_route(
            &request("POST", "/v1/groups/receipts?request_id=a", auth),
            &state,
        )
        .await
        .unwrap();
        assert_eq!(post.status, 405);
        // Order: a malformed request is 400 before authentication is checked;
        // a well-formed one without the token is 401.
        let malformed_unauthenticated =
            store_route(&request("GET", "/v1/groups/receipts?id=a", None), &state)
                .await
                .unwrap();
        assert_eq!(malformed_unauthenticated.status, 400);
        let unauthenticated = store_route(
            &request("GET", "/v1/groups/receipts?request_id=a", None),
            &state,
        )
        .await
        .unwrap();
        assert_eq!(unauthenticated.status, 401);
        {
            let store = ProtocolStore::open(state.store.as_ref().unwrap().options()).unwrap();
            store
                .write(|tx| {
                    tx.put_json(
                        whatsapp_api::store::Table::Checkpoints,
                        b"group-refresh-result:ACTION-1",
                        1,
                        &json!({
                            "complete": 2,
                            "truncated": [{"jid": "120363000000000009@g.us", "size": 300}],
                            "forbidden": 1,
                            "not_found": 0,
                        }),
                    )?;
                    // A legacy key outcome without a frozen plan.
                    tx.put_json(
                        whatsapp_api::store::Table::Checkpoints,
                        b"message:legacy-keys:keys",
                        1,
                        &json!({"status": "answered"}),
                    )?;
                    // A saved group send: counts only, never the group.
                    tx.put_json(
                        whatsapp_api::store::Table::Checkpoints,
                        b"message:group-send:send",
                        1,
                        &json!({
                            "chat": "120363000000000009@g.us",
                            "wire_id": "3EB0FIXTURE",
                            "status": "server_acknowledged",
                        }),
                    )?;
                    // A damaged one: no saved wire ID.
                    tx.put_json(
                        whatsapp_api::store::Table::Checkpoints,
                        b"message:damaged-send:send",
                        1,
                        &json!({"chat": "120363000000000009@g.us"}),
                    )?;
                    // Own-LID sources that disagree.
                    tx.put_json(
                        whatsapp_api::store::Table::Checkpoints,
                        b"own-lid",
                        1,
                        &whatsapp_api::model::jid::Jid::parse("111111@lid").unwrap(),
                    )?;
                    tx.put_json(
                        whatsapp_api::store::Table::Checkpoints,
                        b"login-identity",
                        1,
                        &json!({"lid": "222222@lid"}),
                    )
                })
                .unwrap();
        }
        let conflict = store_route(&request("GET", "/v1/diagnostics/own-lid", auth), &state)
            .await
            .unwrap();
        assert_eq!(conflict.status, 422);
        assert_eq!(
            body(&conflict)["error"]["message"],
            "saved own-lid and login-identity.lid disagree"
        );
        let text = String::from_utf8_lossy(&conflict.body);
        assert!(!text.contains("111111") && !text.contains("222222"));
        {
            let store = ProtocolStore::open(state.store.as_ref().unwrap().options()).unwrap();
            store
                .write(|tx| {
                    tx.put_json(
                        whatsapp_api::store::Table::Checkpoints,
                        b"login-identity",
                        1,
                        &json!({"lid": "111111@lid"}),
                    )
                })
                .unwrap();
        }
        let agreed = store_route(&request("GET", "/v1/diagnostics/own-lid", auth), &state)
            .await
            .unwrap();
        assert_eq!(
            (agreed.status, body(&agreed)),
            (200, json!({"lid_known": true}))
        );
        let refresh = store_route(
            &request("GET", "/v1/groups/refresh-status?id=ACTION-1", auth),
            &state,
        )
        .await
        .unwrap();
        assert_eq!(refresh.status, 200);
        assert_eq!(body(&refresh)["complete"], 2);
        assert_eq!(body(&refresh)["truncated"], 1);
        assert_eq!(
            body(&refresh)["pending"],
            1,
            "legacy report: pending recovered"
        );
        assert!(!String::from_utf8_lossy(&refresh.body).contains("120363"));
        let legacy = store_route(
            &request("GET", "/v1/groups/key-queries?request_id=legacy-keys", auth),
            &state,
        )
        .await
        .unwrap();
        assert_eq!(legacy.status, 422);
        assert_eq!(body(&legacy)["error"]["code"], "saved_state_unsupported");
        assert_eq!(
            body(&legacy)["error"]["message"],
            "legacy key outcome has no frozen plan; counts unknown, original preserved"
        );
        let sent = store_route(
            &request("GET", "/v1/groups/receipts?request_id=group-send", auth),
            &state,
        )
        .await
        .unwrap();
        assert_eq!(sent.status, 200);
        assert_eq!(body(&sent)["send_status"], "server_acknowledged");
        assert_eq!(body(&sent)["receipt_devices"], 0);
        assert!(!String::from_utf8_lossy(&sent.body).contains("120363"));
        let damaged = store_route(
            &request("GET", "/v1/groups/receipts?request_id=damaged-send", auth),
            &state,
        )
        .await
        .unwrap();
        assert_eq!(damaged.status, 422);
        assert_eq!(
            body(&damaged)["error"]["message"],
            "missing saved group receipt wire id"
        );
        // While another handle holds the store (as `listen` would), the route
        // answers 503 with a fixed message instead of waiting or leaking paths.
        let held = ProtocolStore::open(state.store.as_ref().unwrap().options()).unwrap();
        let busy = store_route(&request("GET", "/v1/receipts", auth), &state)
            .await
            .unwrap();
        assert_eq!(busy.status, 503);
        assert_eq!(body(&busy)["error"]["code"], "store_unavailable");
        assert!(!String::from_utf8_lossy(&busy.body).contains("served.redb"));
        // A damaged row is not "busy": 500 with a fixed message, no retry hint.
        held.write(|tx| {
            tx.put_json(
                whatsapp_api::store::Table::Checkpoints,
                b"receipt-outbox:damaged:read",
                1,
                &json!({"status": "pending"}),
            )
        })
        .unwrap();
        drop(held);
        let damaged = store_route(&request("GET", "/v1/receipts", auth), &state)
            .await
            .unwrap();
        assert_eq!(damaged.status, 500);
        assert_eq!(body(&damaged)["error"]["code"], "store_read_failed");
    }

    #[test]
    fn token_files_are_new_private_and_never_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("token");
        let (token, guard) = issue_token(&path).unwrap();
        assert_eq!(token.len(), 64);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), *token);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }
        let error = issue_token(&path).unwrap_err().to_string();
        assert!(error.contains("already exists"));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), *token);
        drop(guard);
        assert!(!path.exists());
    }

    #[tokio::test]
    async fn store_routes_refuse_non_loopback_binds_before_opening_anything() {
        let dir = tempfile::tempdir().unwrap();
        let store = StoreServe {
            options: whatsapp_api::store::OpenOptions {
                path: dir.path().join("never-opened.redb"),
                account_id: whatsapp_api::model::ids::AccountId::new("served").unwrap(),
                key_source: Arc::new(whatsapp_api::store::EnvKeySource::new(
                    "WHATSAPP_TEST_UNSET_KEY",
                )),
                create: false,
            },
            token_file: dir.path().join("token"),
        };
        let error = serve("0.0.0.0:0".parse().unwrap(), true, Some(store))
            .await
            .unwrap_err()
            .to_string();
        assert!(error.contains("loopback-only"));
        assert!(!dir.path().join("token").exists());
        assert!(!dir.path().join("never-opened.redb").exists());
    }

    #[tokio::test]
    async fn a_request_without_headers_is_answered_not_panicked() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let client = tokio::spawn(async move {
            let mut stream = TcpStream::connect(addr).await.unwrap();
            stream
                .write_all(b"GET /healthz HTTP/1.0\r\n\r\n")
                .await
                .unwrap();
            let mut answer = Vec::new();
            stream.read_to_end(&mut answer).await.unwrap();
            answer
        });
        let (server, _) = listener.accept().await.unwrap();
        handle_connection(server, &state()).await.unwrap();
        let answer = String::from_utf8(client.await.unwrap()).unwrap();
        assert!(answer.starts_with("HTTP/1.1 200 OK"), "{answer}");
    }

    #[tokio::test]
    async fn authorization_header_is_parsed_case_insensitively() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let client = tokio::spawn(async move {
            let mut stream = TcpStream::connect(addr).await.unwrap();
            stream
                .write_all(
                    b"GET /v1/receipts HTTP/1.1\r\nHost: x\r\nauthorization:  Bearer abc \r\n\r\n",
                )
                .await
                .unwrap();
            stream
        });
        let (mut server, _) = listener.accept().await.unwrap();
        let parsed = read_request(&mut server).await.unwrap().unwrap();
        assert_eq!(parsed.authorization.as_deref(), Some("Bearer abc"));
        drop(client.await.unwrap());
    }

    #[test]
    fn served_routes_are_declared_http_in_openapi() {
        let doc: Value =
            serde_json::from_str(OPENAPI_JSON).expect("docs/api/openapi.json is valid JSON");
        let declared_http_paths = doc["paths"]
            .as_object()
            .unwrap()
            .iter()
            .flat_map(|(path, path_item)| {
                path_item
                    .as_object()
                    .unwrap()
                    .iter()
                    .filter_map(move |(method, op)| {
                        let x_transport_check = op
                            .get("x-transport")
                            .and_then(|t| t.as_array())
                            .map(|arr| arr.iter().any(|t| t == "http"))
                            .unwrap_or_default();
                        if x_transport_check {
                            Some((method.to_uppercase(), path.clone()))
                        } else {
                            None
                        }
                    })
            })
            .collect::<Vec<_>>();

        let routes = [
            ("GET", "/"),
            ("HEAD", "/"),
            ("GET", "/healthz"),
            ("HEAD", "/healthz"),
            ("GET", "/openapi.json"),
            ("GET", "/v1/profile"),
            ("HEAD", "/v1/profile"),
            ("GET", "/v1/capabilities"),
            ("HEAD", "/v1/capabilities"),
            ("GET", "/v1/transport/wire-profile"),
            ("GET", "/v1/registration/wire-profile"),
            ("POST", "/v1/connect"),
            ("POST", "/v1/registration"),
            ("GET", "/v1/outbox/{local_id}/versions"),
        ]
        .into_iter()
        .chain(STORE_ROUTES.iter().map(|path| ("GET", *path)))
        .chain(QUERY_ROUTES.iter().map(|(path, _)| ("GET", *path)))
        .collect::<Vec<_>>();

        for (method, path) in &routes {
            let found = declared_http_paths
                .iter()
                .any(|(m, p)| m == *method && p == *path);
            assert!(
                found,
                "route {method} {path} not declared with x-transport [\"http\"]"
            );
        }

        // Each parameterized route takes exactly the required query parameter
        // OpenAPI names for it.
        for (path, name) in QUERY_ROUTES {
            let required: Vec<&str> = doc["paths"][path]["get"]["parameters"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|p| p["in"] == "query" && p["required"] == true)
                .filter_map(|p| p["name"].as_str())
                .collect();
            assert_eq!(required, vec![name], "{path}");
        }

        for (method, path) in &declared_http_paths {
            let found = routes
                .iter()
                .any(|(m, p)| *m == method.as_str() && p == path);
            assert!(
                found,
                "OpenAPI declares {method} {path} with x-transport [\"http\"] but route is not served"
            );
        }
    }
}
