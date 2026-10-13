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
    /// Inspect consent selectors once using saved identity; no OTP or SMS.
    InspectConsent,
    /// Submit one reviewed decision from private JSON matching saved selectors.
    SubmitConsent {
        #[arg(long)]
        decision: std::path::PathBuf,
    },
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
        /// Create the store if it does not exist, or claim an existing store
        /// that holds no attempt, for the config's number. `accounts new`
        /// already creates and claims its profile's store, so this is only for
        /// a profile made before it did. Without it, a forgotten `--profile`,
        /// a mistyped path or an empty store another command created fails
        /// instead of starting a fresh identity; another number's store is
        /// always refused.
        #[arg(long)]
        create_store: bool,
    },
    /// Ask for a *new* code on the attempt already in progress, because a
    /// delivered code expires long before the attempt does. Keeps the attempt,
    /// its identity, its keys and its history; honours the server's
    /// `retry_after`; refuses any state but a pending challenge.
    ResendCode {
        #[arg(long, value_enum)]
        method: Option<super::Method>,
    },
    /// Submit one OTP from redirected stdin (never a command-line argument).
    VerifyCode,
    /// Submit one account PIN from redirected stdin; never resets the account PIN.
    VerifyPin,
    /// Local only: move a `challenge` refusal of /v2/register to the email
    /// second-factor step (2.26.37.71 launchEmailChallengeFlow). No request.
    ResumeEmailChallenge,
    /// Ask the server to email the one-time code to the address on file
    /// (`/v2/code` with `method=email_otp`, 2.26.37.71 `LX/0GB.A01`).
    RequestEmailCode {
        /// Not sent by the comparison client; rejected when given.
        #[arg(long)]
        email: Option<String>,
    },
    /// One read-safe `/v2/exist` probe of method eligibility for the saved
    /// attempt (email one-time code, old-device confirmation, waits). Prints
    /// the server's answer and changes no state.
    Exist,
    /// Submit the emailed one-time code from redirected stdin (`/v2/register`).
    VerifyEmailCode,
}

/// Keep only the first character of the local part and the domain of an
/// address the server already masked (`a***@example.com`).
fn mask_email(value: &str) -> String {
    match value.split_once('@') {
        Some((local, domain)) => {
            let first: String = local.chars().take(1).collect();
            format!("{first}***@{domain}")
        }
        None => "***".into(),
    }
}

fn invalid(message: &'static str) -> Error {
    Error::invalid("lab_signup", message)
}

fn authorize(action: &Action, allow_live: bool) -> Result<(), Error> {
    if !matches!(
        action,
        Action::Template | Action::Status | Action::Preview { .. } | Action::ResumeEmailChallenge
    ) && !allow_live
    {
        return Err(invalid(
            "requires --allow-live-request; experimental signup can affect an existing account",
        ));
    }
    Ok(())
}

pub(super) fn config(path: &Path) -> Result<LabRegistrationConfig, Error> {
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

impl DiagnosticTransport {
    fn prepare(
        &self,
        request: EncodedRegistrationRequest,
    ) -> Result<EncodedRegistrationRequest, Error> {
        let request = if let Some(fields) = &self.extra
            && request.url().ends_with("/v2/code")
        {
            request.with_additional_code_fields(fields)?
        } else {
            request
        };
        Ok(match self.body_mode {
            RequestBodyMode::EncEmptyH => request.encrypt_comparison_without_signature()?,
            RequestBodyMode::Enc => request.encrypt_comparison()?,
            RequestBodyMode::ClearForm => request,
        })
    }
}

fn not_sent(error: Error) -> Error {
    match error {
        Error::RequestNotSent(_) => error,
        other => Error::RequestNotSent(Box::new(other)),
    }
}

impl RegistrationTransport for DiagnosticTransport {
    fn send<'a>(&'a self, request: EncodedRegistrationRequest) -> RegistrationTransportFuture<'a> {
        Box::pin(async move {
            // Adding fields or encrypting fails before anything is written, so
            // the failure is "not sent", not an unknown outcome (registration
            // safety audit L4, 2026-10-13): the attempt keeps its prior state.
            let request = self.prepare(request).map_err(not_sent)?;
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
    let create = store_creation(action, &cli.store)?;
    let store = Arc::new(ProtocolStore::open(OpenOptions {
        path: cli.store.clone(),
        account_id: whatsapp_api::model::ids::AccountId::new(cli.account.clone())?,
        key_source: Arc::new(EnvKeySource::new(cli.key_env.clone())),
        create,
    })?);
    if matches!(action, Action::Begin { .. }) {
        begin_store_guard(&store, &config.phone, create)?;
    }
    let factory = Arc::new(if matches!(action, Action::RetryCorrectedToken) {
        StoredComparisonRequestFactory::new_for_token_correction(store.clone(), config)?
    } else if matches!(action, Action::RetryCorrectedRequest) {
        StoredComparisonRequestFactory::new_for_legacy_request_correction(store.clone(), config)?
    } else {
        StoredComparisonRequestFactory::new(store.clone(), config)?
    });
    let flow = Arc::new(ComparisonHttpFlow::new(factory.clone(), transport.clone()));
    let service = RegistrationService::new(store, &profile, flow.clone());
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
                Some(attempt) if cli.json => {
                    // Full record including the transition history, redacted.
                    let mut value = serde_json::to_value(&attempt).map_err(|e| {
                        Error::invalid("lab_signup", format!("attempt is not serialisable: {e}"))
                    })?;
                    redact_attempt(&mut value);
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&value)
                            .map_err(|e| Error::invalid("lab_signup", e.to_string()))?
                    );
                }
                Some(attempt) => println!("{attempt:?}"),
                None => println!("no lab registration attempt"),
            }
            return Ok(());
        }
        Action::Begin { method, .. } => {
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
        Action::ResendCode { method } => {
            let method = match method {
                Some(super::Method::Sms) => ChallengeMethod::Sms,
                Some(super::Method::Voice) => ChallengeMethod::Voice,
                None => match configured_method {
                    DeliveryMethod::Sms => ChallengeMethod::Sms,
                    DeliveryMethod::Voice => ChallengeMethod::Voice,
                },
            };
            service.resend_code(phone, method).await?
        }
        Action::DiagnoseLegacyRefusal => service.diagnose_legacy_refusal_once(phone).await?,
        Action::RetryCorrectedToken => service.retry_corrected_token_once(phone).await?,
        Action::RetryCorrectedRequest => service.diagnose_legacy_refusal_once(phone).await?,
        Action::ResumePreflight => service.resume_preflight_once(phone).await?,
        Action::InspectConsent => {
            let attempt = service
                .current()
                .await?
                .ok_or_else(|| invalid("no saved attempt"))?;
            factory.consent_request(&attempt, None)?;
            service.continue_consent(phone, None).await?
        }
        Action::SubmitConsent { decision } => {
            let file = File::open(decision)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if file.metadata()?.permissions().mode() & 0o077 != 0 {
                    return Err(invalid(
                        "consent decision requires private file permissions",
                    ));
                }
            }
            if !file.metadata()?.is_file() || file.metadata()?.len() > MAX_CONFIG_BYTES {
                return Err(invalid("consent decision must be a bounded regular file"));
            }
            let mut data = Zeroizing::new(Vec::new());
            file.take(MAX_CONFIG_BYTES + 1).read_to_end(&mut data)?;
            if data.len() as u64 > MAX_CONFIG_BYTES {
                return Err(invalid("consent decision too large"));
            }
            let decision: whatsapp_api::registration::consent::ConsentDecision =
                serde_json::from_slice(&data)
                    .map_err(|_| invalid("invalid consent decision JSON"))?;
            decision.validate()?;
            let attempt = service
                .current()
                .await?
                .ok_or_else(|| invalid("no saved attempt"))?;
            factory.consent_request(&attempt, Some(&decision))?;
            service.continue_consent(phone, Some(decision)).await?
        }
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
        Action::Exist => {
            let attempt = service
                .current()
                .await?
                .ok_or_else(|| invalid("no registration attempt"))?;
            exist_allowed(&attempt.state, whatsapp_api::model::time::Timestamp::now())?;
            let (status_code, mut response) = flow.exist_probe(&attempt).await?;
            let email = response.take_secrets().take_email();
            let summary = serde_json::json!({
                "http_status": status_code,
                "status": format!("{:?}", response.status),
                "reason": response.reason.as_ref().map(|r| format!("{r:?}")),
                "email_otp_eligible": response.email_otp_eligible,
                "wa_old_eligible": response.wa_old_eligible,
                "email_on_file": email.as_deref().map(|e| mask_email(e.as_str())),
                "fallback_methods": response.fallback_methods.as_ref().map(|m| format!("{m:?}")),
                "recommended_method": response.recommended_method.as_ref().map(|m| format!("{m:?}")),
                "retry_after_seconds": response.retry_after.map(|d| d.as_secs()),
                "sms_wait_seconds": response.sms_wait.map(|d| d.as_secs()),
                "voice_wait_seconds": response.voice_wait.map(|d| d.as_secs()),
                "email_otp_wait_seconds": response.email_otp_wait.map(|d| d.as_secs()),
                "is_device_trusted": response.is_device_trusted,
            });
            println!(
                "exist: {}",
                serde_json::to_string_pretty(&summary)
                    .map_err(|e| Error::invalid("lab_signup", e.to_string()))?
            );
            return Ok(());
        }
        Action::ResumeEmailChallenge => service.resume_email_challenge().await?,
        Action::RequestEmailCode { email } => {
            let attempt = service
                .current()
                .await?
                .ok_or_else(|| invalid("no registration attempt"))?;
            let RegistrationState::EmailChallengeRequired { handle } = &attempt.state else {
                return Err(invalid(
                    "current state does not permit an email code request; inspect status",
                ));
            };
            service
                .request_email_code(handle.clone(), email.clone())
                .await?
        }
        Action::VerifyCode | Action::VerifyPin | Action::VerifyEmailCode => {
            let attempt = service
                .current()
                .await?
                .ok_or_else(|| invalid("no registration attempt"))?;
            let handle = match (&attempt.state, action) {
                (RegistrationState::ChallengePending { handle }, Action::VerifyCode)
                | (RegistrationState::NeedsUserAction { handle }, Action::VerifyPin)
                | (RegistrationState::EmailCodePending { handle }, Action::VerifyEmailCode) => {
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
            let value = match action {
                Action::VerifyCode => Answer::Code(value),
                Action::VerifyEmailCode => Answer::EmailCode(value),
                _ => Answer::Pin(value),
            };
            service.submit(handle, value).await?
        }
    };
    super::print_step(&step, cli.json);
    Ok(())
}

/// Whether this action may create its store. Only an explicit `begin
/// --create-store` does (registration safety audit, 2026-10-13): a new store
/// means a fresh identity, and must never come from a forgotten profile or a
/// mistyped path. A missing store is otherwise refused before it is opened.
fn store_creation(action: &Action, store: &Path) -> Result<bool, Error> {
    let create = matches!(
        action,
        Action::Begin {
            create_store: true,
            ..
        }
    );
    if !create && !store.exists() {
        return Err(invalid(
            "store does not exist; select the number's profile with --profile, or pass `begin --create-store` for a profile whose store was never created",
        ));
    }
    Ok(create)
}

/// A store that holds no attempt yet must have been created for this number:
/// by `accounts new`, or by an explicit `--create-store`, which claims it. An
/// empty store some other command created, or another number's store selected
/// with a mismatched `--config`, is refused: starting an attempt there is a
/// fresh identity (registration safety audit review, 2026-10-13). A store with
/// an attempt is left to the library, which refuses another number's attempt.
fn begin_store_guard(
    store: &ProtocolStore,
    phone: &whatsapp_api::model::phone::PhoneNumber,
    create: bool,
) -> Result<(), Error> {
    use whatsapp_api::registration::{claim_store, current_state_name, store_owner};
    if current_state_name(store)?.is_some() {
        return Ok(());
    }
    match store_owner(store)? {
        Some(owner) if &owner == phone => Ok(()),
        Some(_) => Err(invalid(
            "this store was created for another number; select that number's profile",
        )),
        None if create => claim_store(store, phone),
        None => Err(invalid(
            "this store holds no attempt and was not created for this number; select the number's profile, or pass `begin --create-store` to claim it",
        )),
    }
}

/// The read-safe `/v2/exist` probe has no reservation, so it is refused where
/// asking the server anything would talk over the attempt: a request in flight
/// or of unknown outcome, a registered account, or a cooldown or refusal wait
/// the server set that has not elapsed (registration safety audit, 2026-10-13).
fn exist_allowed(
    state: &RegistrationState,
    now: whatsapp_api::model::time::Timestamp,
) -> Result<(), Error> {
    use whatsapp_api::registration::RegistrationState as S;
    match state {
        S::Preparing | S::Verifying { .. } | S::RegistrationOutcomeUnknown { .. } => Err(invalid(
            "a request is in flight or its outcome is unknown; the probe would talk over it",
        )),
        S::Registered => Err(invalid("the account is already registered")),
        S::CoolingDown { until } if now < *until => {
            Err(invalid("the server set a wait that has not elapsed"))
        }
        // A refusal's SMS/voice/email wait is a server-set wait too.
        _ if state.refusal_not_before().is_some_and(|until| now < until) => {
            Err(invalid("the server set a wait that has not elapsed"))
        }
        // So are a consent refusal's waits, which consent inspection, on the
        // same endpoint, already enforces (registration review nit).
        S::Blocked {
            reason:
                whatsapp_api::registration::BlockedReason::ConsentRequired {
                    retry_after,
                    sms_retry_after,
                    voice_retry_after,
                    ..
                },
        } if [retry_after, sms_retry_after, voice_retry_after]
            .into_iter()
            .flatten()
            .any(|until| now < *until) =>
        {
            Err(invalid("the server set a wait that has not elapsed"))
        }
        _ => Ok(()),
    }
}

/// Fields of a saved attempt that must never be printed: the phone number, and
/// the consent context's intermediate login, opaque pending selector and parent
/// consent URL (`RegistrationConsent`, kept only in encrypted storage). Matched
/// at any depth, so a field nested under a blocked reason is covered too.
const REDACTED_ATTEMPT_KEYS: [&str; 4] = [
    "phone",
    "registration_login",
    "pending",
    "parent_consent_url",
];

/// Replace every present value of [`REDACTED_ATTEMPT_KEYS`] with a marker;
/// `null` stays `null`, so presence remains visible without the value.
fn redact_attempt(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, field) in map.iter_mut() {
                if REDACTED_ATTEMPT_KEYS.contains(&key.as_str()) {
                    if !field.is_null() {
                        *field = serde_json::Value::String("<redacted>".into());
                    }
                } else {
                    redact_attempt(field);
                }
            }
        }
        serde_json::Value::Array(items) => items.iter_mut().for_each(redact_attempt),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The lab wrapper's own failures (adding fields, encrypting) happen
    /// before any write, so they must read as "not sent", which keeps the
    /// attempt's prior state, not as an unknown outcome (audit L4).
    #[tokio::test]
    async fn diagnostic_preparation_failures_are_not_sent() {
        use whatsapp_api::registration::request::EndpointFields;
        use whatsapp_api::registration::wire::{
            RegistrationEndpoint, RegistrationForm, WHATSAPP_2_26_37_70,
        };
        let mut extra = EndpointFields::default();
        // Register-only fields are refused on the code-request patch.
        extra
            .register
            .insert("refused".into(), Zeroizing::new("1".into()));
        let transport = DiagnosticTransport {
            inner: ExperimentalHttpsTransport::new(std::time::Duration::from_secs(30)).unwrap(),
            enabled: false,
            body_mode: RequestBodyMode::ClearForm,
            extra: Some(extra),
        };
        let request = WHATSAPP_2_26_37_70
            .request(RegistrationEndpoint::Code, &RegistrationForm::new())
            .unwrap();
        let error = transport.send(request).await.unwrap_err();
        let Error::RequestNotSent(cause) = error else {
            panic!("expected not sent: {error}");
        };
        assert!(!matches!(*cause, Error::RequestNotSent(_)), "wrapped once");
    }

    /// `status --json` printed the consent context's login, pending selector
    /// and parent consent URL; only the phone was redacted.
    #[test]
    fn status_json_redacts_consent_secrets_at_any_depth() {
        use whatsapp_api::registration::{BlockedReason, RegistrationState, wire};
        let state = RegistrationState::Blocked {
            reason: BlockedReason::ConsentRequired {
                details: wire::RegistrationConsent {
                    login: Some(Zeroizing::new("SECRET-LOGIN".into())),
                    pending: Some(Zeroizing::new("SECRET-PENDING".into())),
                    parent_consent_url: Some(Zeroizing::new("https://SECRET-URL.invalid/".into())),
                    consent_id: Some(7),
                    ..Default::default()
                },
                retry_after: None,
                sms_retry_after: None,
                voice_retry_after: None,
            },
        };
        let mut value = serde_json::json!({
            "phone": "+15550000000",
            "state": serde_json::to_value(&state).unwrap(),
        });
        assert!(
            value.to_string().contains("SECRET-LOGIN"),
            "fixture holds secrets"
        );
        redact_attempt(&mut value);
        let text = value.to_string();
        for secret in ["SECRET", "15550000000"] {
            assert!(!text.contains(secret), "{secret} leaked: {text}");
        }
        assert!(text.contains("\"consent_id\":7"), "non-secret fields stay");
    }

    /// Only `begin --create-store` may create a store; anything else refuses
    /// a missing one before opening it.
    #[test]
    fn only_an_explicit_begin_creates_a_store() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("mistyped.redb");
        let begin = |create_store| Action::Begin {
            method: None,
            create_store,
        };
        let error = store_creation(&begin(false), &missing).unwrap_err();
        assert!(error.to_string().contains("--create-store"), "{error}");
        assert!(store_creation(&Action::ResendCode { method: None }, &missing).is_err());
        assert!(store_creation(&begin(true), &missing).unwrap());
        let present = dir.path().join("present.redb");
        std::fs::write(&present, b"").unwrap();
        assert!(!store_creation(&begin(false), &present).unwrap());
    }

    /// A store with no attempt must have been created for this number; an
    /// explicit `--create-store` claims an unclaimed one; another number's
    /// store is refused.
    #[test]
    fn begin_requires_a_store_created_for_this_number() {
        use whatsapp_api::model::phone::PhoneNumber;
        use whatsapp_api::store::{MasterKey, StaticKeySource};
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("empty.redb"),
            account_id: whatsapp_api::model::ids::AccountId::new("signup-test").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([5; 32]))),
            create: true,
        })
        .unwrap();
        let number = PhoneNumber::new(1, "5550100001").unwrap();
        let other = PhoneNumber::new(1, "5550100002").unwrap();
        // An empty store some other command created: refused without the flag.
        assert!(begin_store_guard(&store, &number, false).is_err());
        // Claimed explicitly, then accepted for that number only.
        begin_store_guard(&store, &number, true).unwrap();
        begin_store_guard(&store, &number, false).unwrap();
        let error = begin_store_guard(&store, &other, true).unwrap_err();
        assert!(error.to_string().contains("another number"), "{error}");
    }

    #[test]
    fn exist_probe_is_refused_mid_request_registered_or_cooling_down() {
        use whatsapp_api::model::time::Timestamp;
        use whatsapp_api::registration::RegistrationState as S;
        let now = Timestamp::now();
        let refused = |not_before| S::Blocked {
            reason: whatsapp_api::registration::BlockedReason::ServerRefused {
                code: None,
                http_status: Some(200),
                reason: None,
                not_before,
            },
        };
        let consent = |sms_retry_after| S::Blocked {
            reason: whatsapp_api::registration::BlockedReason::ConsentRequired {
                details: whatsapp_api::registration::wire::RegistrationConsent::default(),
                retry_after: None,
                sms_retry_after,
                voice_retry_after: None,
            },
        };
        let later = now.saturating_add(std::time::Duration::from_secs(60));
        for state in [
            S::Preparing,
            S::Verifying { step: 1 },
            S::RegistrationOutcomeUnknown { step: 1 },
            S::Registered,
            S::CoolingDown { until: later },
            refused(Some(later)),
            consent(Some(later)),
        ] {
            assert!(exist_allowed(&state, now).is_err(), "{state:?}");
        }
        for state in [
            S::Unregistered,
            S::CoolingDown { until: now },
            refused(None),
            refused(Some(now)),
            consent(None),
            consent(Some(now)),
        ] {
            assert!(exist_allowed(&state, now).is_ok(), "{state:?}");
        }
    }

    #[test]
    fn mutations_require_explicit_authorization() {
        assert!(authorize(&Action::Status, false).is_ok());
        assert!(authorize(&Action::Template, false).is_ok());
        assert!(authorize(&Action::Preview { show_phone: false }, false).is_ok());
        for action in [
            Action::RetryCorrectedToken,
            Action::RetryCorrectedRequest,
            Action::ResumePreflight,
            Action::InspectConsent,
            Action::SubmitConsent {
                decision: "unused.json".into(),
            },
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
                create_store: false,
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
            Action::Begin {
                method: None,
                create_store: false,
            },
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
