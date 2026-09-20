//! Ban-appeal submission: POST to ACS GraphQL with the saved appeal token.
//! Requires `experimental-registration-https` for the constrained reqwest client.
use crate::{
    Error, Result, registration::request::LabRegistrationConfig, session::AccountRecord,
    store::ProtocolStore,
};
use std::{sync::Arc, time::Duration};
use zeroize::Zeroizing;

const ACS_GRAPHQL_URL: &str = "https://acs.whatsapp.com/graphql";
const OPERATION_NAME: &str = "SubmitAnAppealForIP";
const MAX_RESPONSE_BYTES: usize = 64 * 1024;

/// Inputs for a ban-appeal submission.
pub struct BanAppealInput {
    /// Appeal token from the saved account credentials or overridden by the operator.
    pub appeal_token: Zeroizing<String>,
    /// Optional appeal reason text (maps to `unban_reason` in the form).
    pub message: Option<String>,
    /// Persistent query doc_id for SubmitAnAppealForIP. Unknown at compile time;
    /// the operator must supply a candidate value from APK analysis.
    pub doc_id: Option<u64>,
}

impl std::fmt::Debug for BanAppealInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BanAppealInput")
            .field("appeal_token", &"<redacted>")
            .field("has_message", &self.message.is_some())
            .field("doc_id", &self.doc_id)
            .finish()
    }
}

/// Redacted summary of the ACS GraphQL response.
pub struct BanAppealResult {
    /// HTTP status code.
    pub http_status: u16,
    /// Redacted response summary (top-level JSON keys and non-secret fields).
    pub summary: serde_json::Value,
}

impl std::fmt::Debug for BanAppealResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BanAppealResult")
            .field("http_status", &self.http_status)
            .field("summary", &self.summary)
            .finish()
    }
}

/// Read the `noise_appeal_token` credential from the account store, if present.
pub async fn read_stored_appeal_token(
    store: &Arc<ProtocolStore>,
) -> Result<Option<Zeroizing<String>>> {
    store
        .run_blocking(|s| {
            s.read(|tx| {
                let Some(account) = AccountRecord::load(tx)? else {
                    return Ok(None);
                };
                let token = account
                    .credentials
                    .iter()
                    .rev()
                    .find(|c| c.name == "noise_appeal_token")
                    .map(|c| Zeroizing::new(String::from_utf8_lossy(&c.value).into_owned()));
                Ok(token)
            })
        })
        .await
}

/// Submit a ban appeal to `acs.whatsapp.com/graphql`.
///
/// Auth uses the `login` credential from the account store as a Bearer token.
/// The server response is summarised (HTTP status, top-level JSON keys) without
/// printing the full body, which may contain credentials.
pub async fn submit_ban_appeal(
    store: &Arc<ProtocolStore>,
    config: &LabRegistrationConfig,
    input: BanAppealInput,
) -> Result<BanAppealResult> {
    // Read the login credential for Authorization.
    let login_token: Option<Zeroizing<Vec<u8>>> = store
        .run_blocking(|s| {
            s.read(|tx| {
                let Some(account) = AccountRecord::load(tx)? else {
                    return Ok(None);
                };
                let token = account
                    .credentials
                    .iter()
                    .rev()
                    .find(|c| c.name == "login")
                    .map(|c| Zeroizing::new(c.value.to_vec()));
                Ok(token)
            })
        })
        .await?;

    // Build a constrained HTTPS client.
    let client = build_client(config)?;

    // Construct the GraphQL body.
    let body: Zeroizing<Vec<u8>> = {
        let mut vars = serde_json::json!({
            "input": {
                "appeal_token": input.appeal_token.as_str(),
            }
        });
        if let Some(msg) = &input.message {
            vars["input"]["unban_reason"] = serde_json::json!(msg);
        }
        let obj = if let Some(doc_id) = input.doc_id {
            serde_json::json!({ "doc_id": doc_id.to_string(), "variables": vars })
        } else {
            serde_json::json!({ "operationName": OPERATION_NAME, "variables": vars })
        };
        Zeroizing::new(
            serde_json::to_vec(&obj)
                .map_err(|e| Error::invalid("ban_appeal", format!("body serialization: {e}")))?,
        )
    };

    let mut req = client
        .post(ACS_GRAPHQL_URL)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::CONTENT_LENGTH, body.len())
        .body(body.to_vec());

    if let Some(token) = login_token {
        let bearer = format!("Bearer {}", String::from_utf8_lossy(&token));
        req = req.header(reqwest::header::AUTHORIZATION, bearer);
    }

    let response = req
        .send()
        .await
        .map_err(|_| Error::Io(std::io::Error::other("ban appeal HTTPS request failed")))?;

    let http_status = response.status().as_u16();
    let mut raw = Zeroizing::new(Vec::<u8>::new());
    let mut stream = response;
    while let Some(chunk) = stream
        .chunk()
        .await
        .map_err(|_| Error::Io(std::io::Error::other("ban appeal response body failed")))?
    {
        if raw.len().saturating_add(chunk.len()) > MAX_RESPONSE_BYTES {
            return Err(Error::ResourceExhausted {
                resource: "ban appeal response bytes",
                limit: MAX_RESPONSE_BYTES as u64,
            });
        }
        raw.extend_from_slice(&chunk);
    }

    let summary = summarize(http_status, &raw);
    Ok(BanAppealResult {
        http_status,
        summary,
    })
}

fn build_client(config: &LabRegistrationConfig) -> Result<reqwest::Client> {
    let roots = rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let tls = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .map_err(|_| {
        Error::invalid(
            "ban_appeal_tls",
            "ring provider has no safe protocol versions",
        )
    })?
    .with_root_certificates(roots)
    .with_no_client_auth();
    let mut builder = reqwest::Client::builder()
        .use_preconfigured_tls(tls)
        .https_only(true)
        .redirect(reqwest::redirect::Policy::none())
        .retry(reqwest::retry::never())
        .no_proxy()
        .referer(false)
        .no_gzip()
        .no_brotli()
        .no_deflate()
        .no_zstd()
        .timeout(Duration::from_millis(config.transport.timeout_ms))
        .connect_timeout(Duration::from_millis(config.transport.connect_timeout_ms));
    if let Some(ua) = &config.client.user_agent {
        builder = builder.user_agent(ua.as_str());
    }
    builder
        .build()
        .map_err(|_| Error::invalid("ban_appeal_tls", "cannot build appeal client"))
}

/// Redacted summary: HTTP status + top-level JSON keys, no values.
fn summarize(http_status: u16, body: &[u8]) -> serde_json::Value {
    let body_bytes = body.len();
    let mut s = serde_json::json!({ "http_status": http_status, "body_bytes": body_bytes });
    if let Ok(parsed) = serde_json::from_slice::<serde_json::Value>(body) {
        let keys: Vec<String> = parsed
            .as_object()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default();
        s["top_level_keys"] = serde_json::json!(keys);
        // Preserve known non-secret structural fields.
        for field in [
            "status",
            "error",
            "error_code",
            "error_message",
            "error_type",
        ] {
            if let Some(v) = parsed.get(field)
                && (v.is_string() || v.is_number() || v.as_bool().is_some())
            {
                s[field] = v.clone();
            }
        }
        // errors array: keep code + type, mask message text.
        if let Some(errors) = parsed.get("errors").and_then(|e| e.as_array()) {
            let redacted: Vec<serde_json::Value> = errors
                .iter()
                .map(|e| {
                    let mut item = serde_json::json!({});
                    for field in ["code", "type", "error_type"] {
                        if let Some(v) = e.get(field)
                            && (v.is_number() || v.is_string())
                        {
                            item[field] = v.clone();
                        }
                    }
                    item
                })
                .collect();
            s["errors"] = serde_json::json!(redacted);
        }
    }
    s
}
