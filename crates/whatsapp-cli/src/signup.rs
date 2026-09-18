//! Explicit, single-action lab signup; never selected by the ordinary CLI.

use std::{fs::File, io::Read, path::Path, sync::Arc, time::Instant};

use clap::Subcommand;
use whatsapp_api::registration::{
    Answer, ChallengeMethod, ComparisonHttpFlow, ComparisonRequestFactory, RegistrationService,
    RegistrationState,
    factory::StoredComparisonRequestFactory,
    request::{DeliveryMethod, LabRegistrationConfig, MAX_CONFIG_BYTES, RequestBodyMode},
    wire::{
        EncodedRegistrationRequest, ExperimentalHttpsTransport, RegistrationTransport,
        RegistrationTransportFuture,
    },
};
use whatsapp_api::store::{EnvKeySource, OpenOptions, ProtocolStore};
use whatsapp_api::{Error, ProtocolProfile};
use zeroize::Zeroizing;

#[derive(Subcommand)]
pub(super) enum Action {
    /// Print the editable profile template; no file, store or network access.
    Template,
    /// Validate and show a redacted request plan; no network or store access.
    Preview {
        /// Explicitly show the configured phone in local output.
        #[arg(long)]
        show_phone: bool,
    },
    /// Local status only: sends no request.
    Status,
    /// One retry after bad_token; JSON may change only the token, never identity.
    RetryCorrectedToken,
    /// Migrate an opaque legacy refusal using JSON corrections; one send only.
    /// Preserves identity; shares the legacy diagnostic's durable reservation.
    RetryCorrectedRequest,
    /// Recheck a saved preflight stopped by the old missing-recommendation guard.
    ResumePreflight,
    /// One authorized resend of a legacy refusal whose diagnostics were lost.
    /// Known refusals/cooldowns are excluded; a durable latch prevents repeats.
    DiagnoseLegacyRefusal,
    /// One authorized repeat of HTTP 200 / bad_param to collect field hints.
    /// Same saved payload and keys; a durable latch prevents another repeat.
    DiagnoseBadParameter,
    /// One encrypted-envelope experiment after HTTP 200 / bad_param.
    /// Same inner fields and identity; no attestation, no automatic repeats.
    DiagnoseEncryptedParameter,
    /// One ENC plus empty-H experiment; does not fabricate attestation.
    DiagnoseUnsignedEnvelope,
    /// One additive-code-field experiment from a private EndpointFields JSON file.
    /// Uses ENC plus empty H; rejects replacement of saved fields.
    DiagnoseAdditionalFields {
        #[arg(long)]
        fields: std::path::PathBuf,
    },
    /// One explicit post-failure reason/metrics experiment, using ENC plus empty H.
    DiagnoseFlowFields {
        #[arg(long)]
        fields: std::path::PathBuf,
    },
    /// One APK unknown-build-properties User-Agent experiment with additive fields.
    DiagnoseUnknownAgent {
        #[arg(long)]
        fields: std::path::PathBuf,
    },
    /// Request a code or resume an existing attempt; no automatic retries.
    Begin {
        #[arg(long, value_enum)]
        method: Option<super::Method>,
    },
    /// Submit one OTP from redirected stdin (never a command-line argument).
    VerifyCode,
    /// Submit one account PIN from redirected stdin; never resets the account PIN.
    VerifyPin,
}

fn invalid(message: &'static str) -> Error {
    Error::invalid("lab_signup", message)
}

fn authorize(action: &Action, allow_live: bool) -> Result<(), Error> {
    if !matches!(
        action,
        Action::Template | Action::Status | Action::Preview { .. }
    ) && !allow_live
    {
        return Err(invalid(
            "requires --allow-live-request; experimental signup can affect an existing account",
        ));
    }
    Ok(())
}

fn config(path: &Path) -> Result<LabRegistrationConfig, Error> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > MAX_CONFIG_BYTES {
        return Err(invalid(
            "config must be a regular file no larger than 64 KiB",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(invalid(
                "config contains secrets; remove group/other permissions (chmod 600)",
            ));
        }
    }
    let mut data = Zeroizing::new(Vec::new());
    file.take(MAX_CONFIG_BYTES + 1).read_to_end(&mut data)?;
    if data.len() as u64 > MAX_CONFIG_BYTES {
        return Err(invalid("config exceeds 64 KiB"));
    }
    // Do not expose serde's offending value in an operator error.
    let config: LabRegistrationConfig = serde_json::from_slice(&data).map_err(|_| {
        invalid("invalid lab config JSON (unknown fields are rejected; see request.rs)")
    })?;
    config.validate()?;
    Ok(config)
}

#[derive(Debug)]
struct DiagnosticTransport {
    inner: ExperimentalHttpsTransport,
    enabled: bool,
    body_mode: RequestBodyMode,
    extra: Option<whatsapp_api::registration::request::EndpointFields>,
}

impl RegistrationTransport for DiagnosticTransport {
    fn send<'a>(&'a self, request: EncodedRegistrationRequest) -> RegistrationTransportFuture<'a> {
        Box::pin(async move {
            let request = if let Some(fields) = &self.extra
                && request.url().ends_with("/v2/code")
            {
                request.with_additional_code_fields(fields)?
            } else {
                request
            };
            let request = match self.body_mode {
                RequestBodyMode::EncEmptyH => request.encrypt_comparison_without_signature()?,
                RequestBodyMode::Enc => request.encrypt_comparison()?,
                RequestBodyMode::ClearForm => request,
            };
            if self.enabled {
                eprintln!("signup request: {}", request.diagnostic_summary());
            }
            let started = Instant::now();
            let result = self.inner.send(request).await;
            if self.enabled {
                match &result {
                    Ok(response) => eprintln!(
                        "signup response ({} ms): {}",
                        started.elapsed().as_millis(),
                        response.diagnostic_summary()
                    ),
                    Err(_) => eprintln!(
                        "signup transport failed ({} ms); no usable response; no retry",
                        started.elapsed().as_millis()
                    ),
                }
            }
            result
        })
    }
}

fn request_body_mode(configured: RequestBodyMode, action: &Action) -> RequestBodyMode {
    match action {
        Action::DiagnoseEncryptedParameter => RequestBodyMode::Enc,
        Action::DiagnoseUnsignedEnvelope
        | Action::DiagnoseAdditionalFields { .. }
        | Action::DiagnoseFlowFields { .. }
        | Action::DiagnoseUnknownAgent { .. } => RequestBodyMode::EncEmptyH,
        _ => configured,
    }
}

fn answer(mut input: impl Read) -> Result<Zeroizing<String>, Error> {
    let mut value = Zeroizing::new(String::new());
    input
        .by_ref()
        .take(33)
        .read_to_string(&mut value)
        .map_err(|_| invalid("cannot read bounded answer"))?;
    if value.len() > 32 {
        return Err(invalid("answer too long"));
    }
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > 16 || !trimmed.bytes().all(|b| b.is_ascii_digit()) {
        return Err(invalid("answer must contain 1..=16 ASCII digits"));
    }
    Ok(Zeroizing::new(trimmed.to_owned()))
}

pub(super) async fn run(
    cli: &super::Cli,
    path: &Path,
    allow_live: bool,
    action: &Action,
) -> Result<(), Error> {
    use std::io::IsTerminal;
    authorize(action, allow_live)?;
    if matches!(action, Action::Template) {
        println!("{}", whatsapp_api::registration::request::TEMPLATE_JSON);
        return Ok(());
    }
    let config = config(path)?;
    if let Action::Preview { show_phone } = action {
        let mut preview = config.preview(*show_phone);
        preview["store"] = serde_json::json!(cli.store);
        preview["master_key_environment_variable"] = serde_json::json!(cli.key_env);
        println!(
            "{}",
            serde_json::to_string_pretty(&preview)
                .map_err(|_| invalid("preview serialization failed"))?
        );
        return Ok(());
    }
    let phone = config.phone.clone();
    if matches!(action, Action::ResumePreflight) && !config.preflight {
        return Err(invalid(
            "resume-preflight requires the saved preflight-enabled profile",
        ));
    }
    let configured_method = config.method;
    let migration_metrics = if matches!(action, Action::RetryCorrectedRequest) {
        config.fields.code_bytes.get("client_metrics").cloned()
    } else {
        None
    };
    let extra = if let Action::DiagnoseAdditionalFields { fields }
    | Action::DiagnoseFlowFields { fields }
    | Action::DiagnoseUnknownAgent { fields } = action
    {
        let file = File::open(fields)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if file.metadata()?.permissions().mode() & 0o077 != 0 {
                return Err(invalid(
                    "field patch must have private permissions (chmod 600)",
                ));
            }
        }
        if !file.metadata()?.is_file() || file.metadata()?.len() > MAX_CONFIG_BYTES {
            return Err(invalid(
                "field patch must be a regular file within config budget",
            ));
        }
        let mut data = Zeroizing::new(Vec::new());
        file.take(MAX_CONFIG_BYTES + 1).read_to_end(&mut data)?;
        if data.len() as u64 > MAX_CONFIG_BYTES {
            return Err(invalid("field patch too large"));
        }
        Some(serde_json::from_slice(&data).map_err(|_| invalid("invalid field patch JSON"))?)
    } else {
        None
    };
    let mut profile = ProtocolProfile::android_reference()?;
    profile.id = config.client.profile_id.clone();
    let mut transport_client = config.client.clone();
    if matches!(action, Action::DiagnoseUnknownAgent { .. }) {
        transport_client.user_agent = Some(Zeroizing::new(
            whatsapp_api::registration::request::UNKNOWN_DEVICE_USER_AGENT.into(),
        ));
        eprintln!("signup experiment: APK unknown-build-properties User-Agent fallback");
    }
    let transport = Arc::new(DiagnosticTransport {
        inner: ExperimentalHttpsTransport::with_settings(&config.transport, &transport_client)?,
        enabled: config.diagnostics,
        body_mode: request_body_mode(config.transport.body_mode, action),
        extra,
    });
    let store = Arc::new(ProtocolStore::open(OpenOptions {
        path: cli.store.clone(),
        account_id: whatsapp_api::model::ids::AccountId::new(cli.account.clone())?,
        key_source: Arc::new(EnvKeySource::new(cli.key_env.clone())),
        create: matches!(action, Action::Begin { .. }),
    })?);
    let factory = Arc::new(if matches!(action, Action::RetryCorrectedToken) {
        StoredComparisonRequestFactory::new_for_token_correction(store.clone(), config)?
    } else if matches!(action, Action::RetryCorrectedRequest) {
        StoredComparisonRequestFactory::new_for_legacy_request_correction(store.clone(), config)?
    } else {
        StoredComparisonRequestFactory::new(store.clone(), config)?
    });
    let service = RegistrationService::new(
        store,
        &profile,
        Arc::new(ComparisonHttpFlow::new(factory.clone(), transport.clone())),
    );
    if let Some(metrics) = migration_metrics {
        let attempt = service
            .current()
            .await?
            .ok_or_else(|| invalid("no saved attempt"))?;
        let next_count = attempt
            .code_preparation_count()
            .ok_or_else(|| invalid("incomplete preparation history"))?
            + 1;
        let metrics: serde_json::Value =
            serde_json::from_slice(&metrics).map_err(|_| invalid("metrics must be JSON"))?;
        if metrics.get("attempts").and_then(|value| value.as_u64()) != Some(next_count as u64) {
            return Err(invalid(
                "metrics attempts must match saved preparations plus this attempt",
            ));
        }
    }
    if let Some(extra) = &transport.extra {
        let attempt = service
            .current()
            .await?
            .ok_or_else(|| invalid("no saved attempt for field patch"))?;
        // Validate collisions and complete request size before reserving any send.
        factory
            .code_request(&attempt)?
            .with_additional_code_fields(extra)?
            .encrypt_comparison_without_signature()?;
        if matches!(
            action,
            Action::DiagnoseFlowFields { .. } | Action::DiagnoseUnknownAgent { .. }
        ) {
            let next_count = attempt.code_preparation_count().ok_or_else(|| {
                invalid("attempt history incomplete; cannot infer metrics counter")
            })? + 1;
            let metrics = extra
                .code_bytes
                .get("client_metrics")
                .ok_or_else(|| invalid("flow experiment requires client_metrics"))?;
            let metrics: serde_json::Value =
                serde_json::from_slice(metrics).map_err(|_| invalid("metrics must be JSON"))?;
            if metrics.get("attempts").and_then(|value| value.as_u64()) != Some(next_count as u64) {
                return Err(invalid(
                    "metrics attempts must match saved preparations plus this attempt",
                ));
            }
            eprintln!(
                "signup flow experiment: preparation count {next_count}; preserves account identity"
            );
        }
    }
    let step = match action {
        Action::Template | Action::Preview { .. } => {
            return Err(invalid("preview must not open the store"));
        }
        Action::Status => {
            match service.current().await? {
                Some(attempt) => println!("{attempt:?}"),
                None => println!("no lab registration attempt"),
            }
            return Ok(());
        }
        Action::Begin { method } => {
            let method = match method {
                Some(super::Method::Sms) => ChallengeMethod::Sms,
                Some(super::Method::Voice) => ChallengeMethod::Voice,
                None => match configured_method {
                    DeliveryMethod::Sms => ChallengeMethod::Sms,
                    DeliveryMethod::Voice => ChallengeMethod::Voice,
                },
            };
            service.begin(phone, method).await?
        }
        Action::DiagnoseLegacyRefusal => service.diagnose_legacy_refusal_once(phone).await?,
        Action::RetryCorrectedToken => service.retry_corrected_token_once(phone).await?,
        Action::RetryCorrectedRequest => service.diagnose_legacy_refusal_once(phone).await?,
        Action::ResumePreflight => service.resume_preflight_once(phone).await?,
        Action::DiagnoseBadParameter => service.diagnose_bad_parameter_once(phone).await?,
        Action::DiagnoseUnsignedEnvelope => service.diagnose_unsigned_envelope_once(phone).await?,
        Action::DiagnoseAdditionalFields { .. } => {
            service.diagnose_additional_fields_once(phone).await?
        }
        Action::DiagnoseFlowFields { .. } => service.diagnose_flow_fields_once(phone).await?,
        Action::DiagnoseUnknownAgent { .. } => service.diagnose_unknown_agent_once(phone).await?,
        Action::DiagnoseEncryptedParameter => {
            service.diagnose_encrypted_parameter_once(phone).await?
        }
        Action::VerifyCode | Action::VerifyPin => {
            let attempt = service
                .current()
                .await?
                .ok_or_else(|| invalid("no registration attempt"))?;
            let handle = match (&attempt.state, action) {
                (RegistrationState::ChallengePending { handle }, Action::VerifyCode)
                | (RegistrationState::NeedsUserAction { handle }, Action::VerifyPin) => {
                    handle.clone()
                }
                _ => {
                    return Err(invalid(
                        "current state does not permit this answer; inspect status and do not retry blindly",
                    ));
                }
            };
            if std::io::stdin().is_terminal() {
                return Err(invalid(
                    "redirect the answer on stdin; interactive echo is disabled by refusing terminal input",
                ));
            }
            let value = answer(std::io::stdin().lock())?;
            let value = if matches!(action, Action::VerifyCode) {
                Answer::Code(value)
            } else {
                Answer::Pin(value)
            };
            service.submit(handle, value).await?
        }
    };
    super::print_step(&step, cli.json);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutations_require_explicit_authorization() {
        assert!(authorize(&Action::Status, false).is_ok());
        assert!(authorize(&Action::Template, false).is_ok());
        assert!(authorize(&Action::Preview { show_phone: false }, false).is_ok());
        for action in [
            Action::RetryCorrectedToken,
            Action::RetryCorrectedRequest,
            Action::ResumePreflight,
            Action::DiagnoseLegacyRefusal,
            Action::DiagnoseBadParameter,
            Action::DiagnoseEncryptedParameter,
            Action::DiagnoseUnsignedEnvelope,
            Action::DiagnoseAdditionalFields {
                fields: "unused.json".into(),
            },
            Action::DiagnoseFlowFields {
                fields: "unused.json".into(),
            },
            Action::DiagnoseUnknownAgent {
                fields: "unused.json".into(),
            },
            Action::Begin {
                method: Some(super::super::Method::Sms),
            },
            Action::VerifyCode,
            Action::VerifyPin,
        ] {
            assert!(authorize(&action, false).is_err());
            assert!(authorize(&action, true).is_ok());
        }
    }

    #[test]
    fn bounded_answers_reject_empty_non_numeric_and_oversized_input() {
        assert_eq!(answer(" 012345\n".as_bytes()).unwrap().as_str(), "012345");
        for input in [
            "",
            "\n",
            "secret-not-digits",
            "12345678901234567",
            &"9".repeat(1000),
        ] {
            assert!(answer(input.as_bytes()).is_err());
        }
    }
}
#[test]
fn ordinary_actions_use_json_body_mode_and_diagnostics_keep_explicit_modes() {
    for mode in [
        RequestBodyMode::ClearForm,
        RequestBodyMode::Enc,
        RequestBodyMode::EncEmptyH,
    ] {
        for action in [
            Action::Begin { method: None },
            Action::VerifyCode,
            Action::VerifyPin,
        ] {
            assert_eq!(request_body_mode(mode, &action), mode);
        }
        assert_eq!(
            request_body_mode(mode, &Action::DiagnoseEncryptedParameter),
            RequestBodyMode::Enc
        );
        assert_eq!(
            request_body_mode(mode, &Action::DiagnoseUnsignedEnvelope),
            RequestBodyMode::EncEmptyH
        );
    }
}
