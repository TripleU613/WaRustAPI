//! Minimal HTTP control plane for local operation.
//!
//! This intentionally exposes only secret-free, offline-safe endpoints while
//! the Android primary-device profile is incomplete. Network mutations return
//! structured unsupported responses instead of bypassing evidence gates.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use tokio::time::timeout;
use whatsapp_api::capabilities::CapabilityMatrix;
use whatsapp_api::registration::wire::{
    COMPARISON_SIGNAL_KEY_TYPE, MAX_RESPONSE_BODY_BYTES, RegistrationEndpoint,
    RegistrationResponseReason, RegistrationResponseStatus, WHATSAPP_2_26_37_70,
};
use whatsapp_api::transport::comparison::{
    COMPARISON_CONNECTION_HEADER, COMPARISON_DICTIONARY_VERSION, COMPARISON_FALLBACK_PORT,
    COMPARISON_PRIMARY_PORTS, COMPARISON_PROTOCOL_VERSION, ComparisonEndpoint,
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

struct ApiState {
    profile: ProtocolProfile,
    capabilities: CapabilityMatrix,
}

struct RequestLine {
    method: String,
    path: String,
}

struct Response {
    status: u16,
    reason: &'static str,
    body: Vec<u8>,
    allow: Option<&'static str>,
}

impl Response {
    fn json(status: u16, reason: &'static str, body: Value) -> Self {
        Self {
            status,
            reason,
            body: serde_json::to_vec(&body).unwrap_or_else(|_| b"{}".to_vec()),
            allow: None,
        }
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
        headers.push_str("\r\n");
        let mut encoded = headers.into_bytes();
        if include_body {
            encoded.extend_from_slice(&self.body);
        }
        encoded
    }
}

pub async fn serve(bind: SocketAddr, allow_non_loopback: bool) -> Result<(), Error> {
    validate_bind(bind, allow_non_loopback)?;

    let profile = ProtocolProfile::android_reference()?;
    let state = Arc::new(ApiState {
        capabilities: CapabilityMatrix::current(profile.id.clone()),
        profile,
    });
    let listener = TcpListener::bind(bind).await?;
    let local_addr = listener.local_addr()?;
    let permits = Arc::new(Semaphore::new(MAX_CONNECTIONS));
    let shutdown = tokio::signal::ctrl_c();
    tokio::pin!(shutdown);

    println!("whatsapp-api HTTP control plane listening on http://{local_addr}");
    println!("profile: {} (offline gates enforced)", state.profile.id);

    loop {
        tokio::select! {
            signal = &mut shutdown => {
                signal?;
                println!("shutdown signal received");
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

fn validate_bind(bind: SocketAddr, allow_non_loopback: bool) -> Result<(), Error> {
    if bind.ip().is_loopback() || allow_non_loopback {
        return Ok(());
    }
    Err(Error::invalid(
        "bind",
        "non-loopback listeners require --allow-non-loopback because HTTP authentication is not implemented",
    ))
}

async fn handle_connection(mut stream: TcpStream, state: &ApiState) -> Result<(), Error> {
    let request = timeout(REQUEST_TIMEOUT, read_request(&mut stream))
        .await
        .map_err(|_| Error::Timeout(REQUEST_TIMEOUT))??;
    let response = match request.as_ref() {
        Some(request) => route(&request.method, &request.path, state),
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

    Ok(Some(RequestLine {
        method: method.to_owned(),
        path: path.to_owned(),
    }))
}

fn route(method: &str, raw_path: &str, state: &ApiState) -> Response {
    let path = raw_path.split_once('?').map_or(raw_path, |(path, _)| path);
    let is_read = matches!(method, "GET" | "HEAD");

    match (method, path) {
        (
            _,
            "/"
            | "/healthz"
            | "/v1/profile"
            | "/v1/capabilities"
            | "/v1/transport/wire-profile"
            | "/v1/registration/wire-profile",
        ) if !is_read => Response::method_not_allowed("GET, HEAD"),
        (_, "/v1/connect" | "/v1/registration") if method != "POST" => {
            Response::method_not_allowed("POST")
        }
        ("GET" | "HEAD", "/") => Response::json(
            200,
            "OK",
            json!({
                "service": "whatsapp-api",
                "version": env!("CARGO_PKG_VERSION"),
                "routes": [
                    "GET /healthz",
                    "GET /v1/profile",
                    "GET /v1/capabilities",
                    "GET /v1/transport/wire-profile",
                    "GET /v1/registration/wire-profile",
                    "POST /v1/connect",
                    "POST /v1/registration"
                ]
            }),
        ),
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
                    "exact port-80 stream semantics",
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
            ComparisonEndpointMode::Port80Fallback => "port80_fallback",
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
}
