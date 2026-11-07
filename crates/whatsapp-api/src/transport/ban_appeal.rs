//! Ban-appeal submission: POST to ACS GraphQL with the saved appeal token.
//! Requires `experimental-registration-https` for the constrained reqwest client.
use crate::{
    Error, Result,
    registration::request::{LabRegistrationConfig, TransportSettings},
    session::AccountRecord,
    store::{ProtocolStore, Table},
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
    /// Redacted response summary (fixed structural field names and counts).
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

fn persist_appeal_token(
    store: &ProtocolStore,
    token: Zeroizing<Vec<u8>>,
    source_step: &str,
    expected_account: &[u8; 32],
) -> Result<()> {
    use crate::{model::time::Timestamp, session::ObservedCredential};

    let now = Timestamp::now();
    store.write(|tx| {
        let mut account = AccountRecord::load_in_write(tx)?.ok_or(Error::NotRegistered)?;
        if super::account_authority::fingerprint(tx.generation(), Some(&account))?
            != *expected_account
        {
            return Err(Error::invalid(
                "ban_appeal",
                "account changed before recovered token persistence",
            ));
        }
        if account.credentials.len() < 64
            && !account
                .credentials
                .iter()
                .any(|c| c.name == "noise_appeal_token")
        {
            account.credentials.push(ObservedCredential {
                name: "noise_appeal_token".into(),
                value: token.clone(),
                observed_at: now,
                source_step: Some(source_step.into()),
            });
            AccountRecord::save(&account, tx)?;
        }
        Ok(())
    })
}

/// Recover the `noise_appeal_token` from preserved failure-node captures if not already saved.
pub fn recover_appeal_token_from_captures(
    store: &ProtocolStore,
) -> Result<Option<Zeroizing<String>>> {
    let found = find_captured_appeal_token(store)?;
    if let Some(candidate) = found {
        persist_appeal_token(
            store,
            candidate.token.clone(),
            "failure-capture",
            &candidate.authority,
        )?;
        Ok(Some(appeal_token_text(&candidate.token)?))
    } else {
        Ok(None)
    }
}

struct CapturedAppealToken {
    token: Zeroizing<Vec<u8>>,
    authority: [u8; 32],
}

// Candidate and account authority share a read snapshot. This binds only the
// recovery operation; historical capture origin remains a separate unknown.
fn find_captured_appeal_token(store: &ProtocolStore) -> Result<Option<CapturedAppealToken>> {
    use crate::binary::unpack_frame_payload;
    store.read(|tx| {
        let limits = crate::profile::Limits::default();
        let mut cursor = None;
        // Page the same read snapshot to exhaustion: unrelated checkpoints must
        // not turn a retained refusal token into a false absence result.
        loop {
            let captures = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
            if captures.is_empty() {
                break;
            }
            cursor = captures.last().map(|(key, _)| key.clone());
            for (key, value) in captures {
                // `inbound:sha256:` is the registration-era capture prefix.
                // `login-result:` is the preserved primary-login result, which is
                // where a transport refusal's `<failure>` — and its appeal token —
                // is kept. A refusal cannot be replayed to fetch the token again, so
                // both sources have to be searched.
                if !key.starts_with(b"inbound:sha256:")
                    && !key.starts_with(b"login-result:")
                    && !key.starts_with(b"inbound:")
                {
                    continue;
                }

                let wire = match serde_json::from_slice::<Vec<u8>>(&value) {
                    Ok(w) => w,
                    Err(_) => continue,
                };

                let payload = match unpack_frame_payload(&wire, &limits) {
                    Ok(p) => p,
                    Err(_) => continue,
                };

                let node = match crate::binary::Decoder::new(
                    &payload,
                    crate::binary::TokenDictionary::comparison_2_26_37_71(),
                    &limits,
                )
                .decode_node()
                {
                    Ok(n) => n,
                    Err(_) => continue,
                };

                if node.description == "failure"
                    && let Some(token) = node.get_attr("appeal_token").and_then(|v| v.as_text())
                {
                    let account = AccountRecord::load(tx)?.ok_or(Error::NotRegistered)?;
                    let authority =
                        super::account_authority::fingerprint(tx.generation(), Some(&account))?;
                    return Ok(Some(CapturedAppealToken {
                        token: Zeroizing::new(token.as_bytes().to_vec()),
                        authority,
                    }));
                }
            }
        }
        Ok(None)
    })
}

// JSON needs text. Never silently substitute token bytes with replacement
// characters; refuse locally without disclosing the stored credential.
fn appeal_token_text(bytes: &[u8]) -> Result<Zeroizing<String>> {
    std::str::from_utf8(bytes)
        .map(|text| Zeroizing::new(text.to_owned()))
        .map_err(|_| Error::invalid("ban_appeal", "stored appeal token is not valid UTF-8"))
}

/// Read the `noise_appeal_token` credential from the account store, if present.
/// Falls back to recovery from preserved failure captures if no credential exists.
pub async fn read_stored_appeal_token(
    store: &Arc<ProtocolStore>,
) -> Result<Option<Zeroizing<String>>> {
    let token = store
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
                    .map(|c| appeal_token_text(&c.value))
                    .transpose()?;
                Ok(token)
            })
        })
        .await?;

    if token.is_some() {
        return Ok(token);
    }

    recover_appeal_token_from_captures(store)
}

/// Submit a ban appeal to `acs.whatsapp.com/graphql`.
///
/// Auth uses the `login` credential from the account store as a Bearer token.
/// The server response is summarised (HTTP status, fixed structural field names and counts) without
/// printing the full body, which may contain credentials.
pub async fn submit_ban_appeal(
    store: &Arc<ProtocolStore>,
    config: &LabRegistrationConfig,
    input: BanAppealInput,
) -> Result<BanAppealResult> {
    let response_limit = appeal_response_limit(&config.transport)?;
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
        req = req.header(
            reqwest::header::AUTHORIZATION,
            authorization_header(&token)?,
        );
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
        append_response_chunk(&mut raw, &chunk, response_limit)?;
    }

    let summary = summarize(http_status, &raw);
    Ok(BanAppealResult {
        http_status,
        summary,
    })
}

fn appeal_response_limit(settings: &TransportSettings) -> Result<usize> {
    settings.validate()?;
    Ok(settings.max_response_bytes.min(MAX_RESPONSE_BYTES))
}

fn append_response_chunk(raw: &mut Vec<u8>, chunk: &[u8], limit: usize) -> Result<()> {
    if raw.len().saturating_add(chunk.len()) > limit {
        return Err(Error::ResourceExhausted {
            resource: "ban appeal response bytes",
            limit: limit as u64,
        });
    }
    raw.extend_from_slice(chunk);
    Ok(())
}

// Preserve opaque saved credential bytes. Lossy UTF-8 decoding changes the
// authentication material; HTTP header validation must refuse invalid bytes.
fn authorization_header(token: &[u8]) -> Result<reqwest::header::HeaderValue> {
    let mut bytes = Zeroizing::new(Vec::with_capacity(7 + token.len()));
    bytes.extend_from_slice(b"Bearer ");
    bytes.extend_from_slice(token);
    let mut header = reqwest::header::HeaderValue::from_bytes(&bytes).map_err(|_| {
        Error::invalid(
            "ban_appeal_auth",
            "saved login credential is not a valid HTTP header",
        )
    })?;
    header.set_sensitive(true);
    Ok(header)
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

/// Redacted summary: local status/size plus fixed structural names and counts.
/// Server-selected text (including key names) can echo credentials or identifiers.
fn summarize(http_status: u16, body: &[u8]) -> serde_json::Value {
    let mut summary = serde_json::json!({ "http_status": http_status, "body_bytes": body.len() });
    if let Ok(serde_json::Value::Object(parsed)) = serde_json::from_slice(body) {
        const FIELDS: &[&str] = &[
            "data",
            "errors",
            "status",
            "error",
            "error_code",
            "error_message",
            "error_type",
        ];
        let keys: Vec<&str> = FIELDS
            .iter()
            .copied()
            .filter(|key| parsed.contains_key(*key))
            .collect();
        summary["top_level_keys"] = serde_json::json!(keys);
        summary["top_level_field_count"] = serde_json::json!(parsed.len());
        if let Some(errors) = parsed.get("errors").and_then(serde_json::Value::as_array) {
            summary["graphql_error_count"] = serde_json::json!(errors.len());
        }
    }
    summary
}

#[cfg(all(test, feature = "experimental-transport"))]
mod tests {
    use super::*;
    use crate::{
        binary::Node,
        model::AccountId,
        session::AccountRecord,
        store::{MasterKey, OpenOptions, StaticKeySource},
    };

    #[test]
    fn appeal_transport_bounds_validate_and_limit_streamed_chunks() {
        let mut settings = TransportSettings::default();
        assert_eq!(
            appeal_response_limit(&settings).unwrap(),
            MAX_RESPONSE_BYTES
        );
        settings.max_response_bytes = 3;
        let limit = appeal_response_limit(&settings).unwrap();
        let mut raw = Vec::new();
        append_response_chunk(&mut raw, b"ab", limit).unwrap();
        append_response_chunk(&mut raw, b"c", limit).unwrap();
        assert_eq!(raw, b"abc");
        assert!(matches!(
            append_response_chunk(&mut raw, b"d", limit),
            Err(Error::ResourceExhausted { limit: 3, .. })
        ));
        assert_eq!(raw, b"abc");
        settings.timeout_ms = 0;
        assert!(appeal_response_limit(&settings).is_err());
        settings = TransportSettings::default();
        settings.connect_timeout_ms = settings.timeout_ms + 1;
        assert!(appeal_response_limit(&settings).is_err());
        settings = TransportSettings::default();
        settings.max_response_bytes = 0;
        assert!(appeal_response_limit(&settings).is_err());
    }

    #[test]
    fn appeal_authorization_preserves_bytes_and_redacts_debug() {
        for token in [
            b"private-login-sentinel".as_slice(),
            b"opaque-\xff-credential".as_slice(),
        ] {
            let header = authorization_header(token).unwrap();
            let mut expected = b"Bearer ".to_vec();
            expected.extend_from_slice(token);
            assert_eq!(header.as_bytes(), expected);
            assert!(header.is_sensitive());
            assert_eq!(format!("{header:?}"), "Sensitive");
        }
        for token in [
            b"private\r\nInjected: value".as_slice(),
            b"private\0credential".as_slice(),
        ] {
            let error = authorization_header(token).unwrap_err();
            assert!(matches!(error, Error::InvalidInput { .. }));
            assert!(!error.to_string().contains("private"));
        }
    }

    #[test]
    fn appeal_response_summary_never_copies_server_text_or_values() {
        let body = serde_json::to_vec(&serde_json::json!({
            "data":{"token":"private-data-sentinel"},
            "errors":[{"message":"private-message-sentinel","code":999888777,"type":"private-type-sentinel"}],
            "status":"private-status-sentinel","error":"private-error-sentinel",
            "error_code":"private-code-sentinel","error_message":"private-echoed-credential",
            "error_type":"private-error-type-sentinel","private-server-key":"private-value-sentinel"
        })).unwrap();
        let summary = summarize(403, &body);
        assert_eq!(summary["http_status"], 403);
        assert_eq!(summary["body_bytes"], body.len());
        assert_eq!(summary["top_level_field_count"], 8);
        assert_eq!(summary["graphql_error_count"], 1);
        assert!(
            summary["top_level_keys"]
                .as_array()
                .unwrap()
                .iter()
                .any(|key| key == "errors")
        );
        let result = BanAppealResult {
            http_status: 403,
            summary,
        };
        for rendered in [result.summary.to_string(), format!("{result:?}")] {
            assert!(!rendered.contains("private-"));
            assert!(!rendered.contains("999888777"));
        }
        for body in [
            b"private-invalid-json".as_slice(),
            b"[\"private-array-value\"]".as_slice(),
        ] {
            assert_eq!(
                summarize(500, body),
                serde_json::json!({"http_status":500,"body_bytes":body.len()})
            );
        }
    }
    #[test]
    fn recover_appeal_token_from_preserved_failure_capture() {
        let temp = tempfile::TempDir::new().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: temp.path().join("test.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9u8; 32]))),
            create: true,
        })
        .unwrap();

        store.write(|tx| AccountRecord::default().save(tx)).unwrap();

        let failure_node = Node::new("failure")
            .attr("reason", "403")
            .attr("location", "cco")
            .attr("appeal_token", "test-appeal-token");

        crate::transport::inbox::preserve(&store, &failure_node).unwrap();

        let token = recover_appeal_token_from_captures(&store).unwrap();
        assert_eq!(
            token.as_ref().map(|t| t.as_str()),
            Some("test-appeal-token")
        );

        let saved_token = store
            .read(|tx| {
                let account = AccountRecord::load(tx)?;
                Ok(account.and_then(|a| {
                    a.credentials
                        .iter()
                        .find(|c| c.name == "noise_appeal_token")
                        .cloned()
                }))
            })
            .unwrap();

        assert!(saved_token.is_some());
        let cred = saved_token.unwrap();
        assert_eq!(String::from_utf8_lossy(&cred.value), "test-appeal-token");
        assert_eq!(cred.source_step, Some("failure-capture".into()));
    }

    #[tokio::test]
    async fn stored_appeal_token_text_is_exact_or_refused_without_mutation() {
        for (bytes, valid) in [
            (b"private-ascii-token".as_slice(), true),
            ("private-שלום-token".as_bytes(), true),
            (b"private-invalid-\xff-token".as_slice(), false),
        ] {
            let temp = tempfile::TempDir::new().unwrap();
            let path = temp.path().join("token-text.redb");
            let open = |create| {
                Arc::new(
                    ProtocolStore::open(OpenOptions {
                        path: path.clone(),
                        account_id: AccountId::new("test").unwrap(),
                        key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                            [9u8; 32],
                        ))),
                        create,
                    })
                    .unwrap(),
                )
            };
            let store = open(true);
            store
                .write(|tx| {
                    let mut account = AccountRecord::default();
                    account
                        .credentials
                        .push(crate::session::ObservedCredential {
                            name: "noise_appeal_token".into(),
                            value: Zeroizing::new(bytes.to_vec()),
                            observed_at: crate::model::time::Timestamp(1),
                            source_step: Some("fixture".into()),
                        });
                    account.save(tx)
                })
                .unwrap();
            let before = store.read(|tx| tx.scan(Table::Account, 1, 100)).unwrap();
            let head = store.journal_head().unwrap();
            let result = read_stored_appeal_token(&store).await;
            if valid {
                assert_eq!(result.unwrap().unwrap().as_bytes(), bytes);
            } else {
                let error =
                    result.expect_err("invalid token bytes became replacement-character text");
                assert!(matches!(
                    error,
                    Error::InvalidInput {
                        field: "ban_appeal",
                        ..
                    }
                ));
                assert!(!error.to_string().contains("private-invalid"));
            }
            assert_eq!(
                store.read(|tx| tx.scan(Table::Account, 1, 100)).unwrap(),
                before
            );
            assert!(
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap()
                    .is_empty()
            );
            assert_eq!(store.journal_head().unwrap(), head);
            drop(store);
            let store = open(false);
            assert_eq!(
                store.read(|tx| tx.scan(Table::Account, 1, 100)).unwrap(),
                before
            );
        }
    }

    #[test]
    fn recovery_finds_failure_beyond_twenty_thousand_rows_after_reopen() {
        let temp = tempfile::TempDir::new().unwrap();
        let path = temp.path().join("paged-recovery.redb");
        let open = |create| {
            ProtocolStore::open(OpenOptions {
                path: path.clone(),
                account_id: AccountId::new("test").unwrap(),
                key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9u8; 32]))),
                create,
            })
            .unwrap()
        };
        let store = open(true);
        store
            .write(|tx| {
                AccountRecord::default().save(tx)?;
                for index in 0..20_005 {
                    tx.put_json(
                        Table::Checkpoints,
                        format!("before:{index:05}").as_bytes(),
                        1,
                        &false,
                    )?;
                }
                Ok(())
            })
            .unwrap();
        crate::transport::inbox::preserve(
            &store,
            &Node::new("failure")
                .attr("reason", "403")
                .attr("appeal_token", "paged-private-token"),
        )
        .unwrap();
        // Decoy eligible captures must not prevent traversal to the real failure.
        store
            .write(|tx| {
                tx.put(Table::Checkpoints, b"inbound:000-invalid-json", 1, b"{")?;
                tx.put_json(
                    Table::Checkpoints,
                    b"inbound:001-invalid-wire",
                    1,
                    &vec![255u8],
                )
            })
            .unwrap();
        let before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 30_000))
            .unwrap();
        drop(store);
        let store = open(false);
        let token = recover_appeal_token_from_captures(&store).unwrap();
        assert_eq!(
            token.as_deref().map(|v| v.as_str()),
            Some("paged-private-token")
        );
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 30_000))
                .unwrap(),
            before
        );
        let account = store.read(|tx| tx.scan(Table::Account, 1, 10)).unwrap();
        drop(store);
        let store = open(false);
        assert_eq!(
            store.read(|tx| tx.scan(Table::Account, 1, 10)).unwrap(),
            account
        );
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 30_000))
                .unwrap(),
            before
        );
    }

    #[test]
    fn recovered_appeal_token_rechecks_account_authority_before_persistence() {
        for case in ["jid", "registration", "missing", "observation", "same"] {
            let temp = tempfile::TempDir::new().unwrap();
            let path = temp.path().join("recovery-owner.redb");
            let open = |create| {
                ProtocolStore::open(OpenOptions {
                    path: path.clone(),
                    account_id: AccountId::new("test").unwrap(),
                    key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9u8; 32]))),
                    create,
                })
                .unwrap()
            };
            let store = open(true);
            store.write(|tx| AccountRecord::default().save(tx)).unwrap();
            crate::transport::inbox::preserve(
                &store,
                &Node::new("failure")
                    .attr("reason", "403")
                    .attr("appeal_token", "private-recovery-token"),
            )
            .unwrap();
            let candidate = find_captured_appeal_token(&store).unwrap().unwrap();
            store
                .write(|tx| {
                    let mut account = AccountRecord::load_in_write(tx)?.unwrap();
                    match case {
                        "jid" => {
                            account.own_jid =
                                Some(crate::model::jid::Jid::parse("15555550100@s.whatsapp.net")?)
                        }
                        "registration" => account.signal_registration_id = Some(123),
                        "missing" => {
                            tx.delete(Table::Account, b"account")?;
                            return Ok(());
                        }
                        "observation" => {
                            account.last_authenticated_at = Some(crate::model::time::Timestamp(123))
                        }
                        _ => {}
                    }
                    account.save(tx)
                })
                .unwrap();
            let account = store.read(|tx| tx.scan(Table::Account, 1, 100)).unwrap();
            let captures = store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap();
            let head = store.journal_head().unwrap();
            let result = persist_appeal_token(
                &store,
                candidate.token,
                "failure-capture",
                &candidate.authority,
            );
            if matches!(case, "same" | "observation") {
                result.unwrap();
                assert!(
                    store
                        .read(AccountRecord::load)
                        .unwrap()
                        .unwrap()
                        .credentials
                        .iter()
                        .any(|c| c.name == "noise_appeal_token")
                );
            } else {
                let error = result.unwrap_err();
                assert!(!error.to_string().contains("private-recovery-token"));
                assert_eq!(
                    store.read(|tx| tx.scan(Table::Account, 1, 100)).unwrap(),
                    account
                );
                assert_eq!(store.journal_head().unwrap(), head);
            }
            let saved = store.read(|tx| tx.scan(Table::Account, 1, 100)).unwrap();
            drop(store);
            let store = open(false);
            assert_eq!(
                store.read(|tx| tx.scan(Table::Account, 1, 100)).unwrap(),
                saved
            );
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                    .unwrap(),
                captures
            );
        }
    }

    #[test]
    fn captured_appeal_token_does_not_create_a_missing_account() {
        let temp = tempfile::TempDir::new().unwrap();
        let path = temp.path().join("missing-account.redb");
        let open = |create| {
            ProtocolStore::open(OpenOptions {
                path: path.clone(),
                account_id: AccountId::new("test").unwrap(),
                key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9u8; 32]))),
                create,
            })
            .unwrap()
        };
        let store = open(true);
        crate::transport::inbox::preserve(
            &store,
            &Node::new("failure")
                .attr("reason", "403")
                .attr("appeal_token", "private-recovery-token"),
        )
        .unwrap();
        let captures = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
            .unwrap();
        let head = store.journal_head().unwrap();
        let result = recover_appeal_token_from_captures(&store);
        assert!(
            matches!(result, Err(Error::NotRegistered)),
            "capture invented an account"
        );
        assert!(store.read(AccountRecord::load).unwrap().is_none());
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap(),
            captures
        );
        assert_eq!(store.journal_head().unwrap(), head);
        drop(store);
        let store = open(false);
        assert!(store.read(AccountRecord::load).unwrap().is_none());
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap(),
            captures
        );
    }

    #[test]
    fn recover_appeal_token_returns_none_for_empty_store() {
        let temp = tempfile::TempDir::new().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: temp.path().join("test.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9u8; 32]))),
            create: true,
        })
        .unwrap();

        let token = recover_appeal_token_from_captures(&store).unwrap();
        assert_eq!(token, None);
    }
}
