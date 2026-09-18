//! Explicit, single-action lab signup; never selected by the ordinary CLI.

use std::{fs::File, io::Read, path::Path, sync::Arc, time::Instant};

use clap::Subcommand;
use whatsapp_api::registration::{
    Answer, ChallengeMethod, ComparisonHttpFlow, RegistrationService, RegistrationState,
    factory::StoredComparisonRequestFactory,
    request::{DeliveryMethod, LabRegistrationConfig, MAX_CONFIG_BYTES},
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
}

impl RegistrationTransport for DiagnosticTransport {
    fn send<'a>(&'a self, request: EncodedRegistrationRequest) -> RegistrationTransportFuture<'a> {
        Box::pin(async move {
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
    let configured_method = config.method;
    let mut profile = ProtocolProfile::android_reference()?;
    profile.id = config.client.profile_id.clone();
    let transport = Arc::new(DiagnosticTransport {
        inner: ExperimentalHttpsTransport::with_settings(&config.transport, &config.client)?,
        enabled: config.diagnostics,
    });
    let store = Arc::new(ProtocolStore::open(OpenOptions {
        path: cli.store.clone(),
        account_id: whatsapp_api::model::ids::AccountId::new(cli.account.clone())?,
        key_source: Arc::new(EnvKeySource::new(cli.key_env.clone())),
        create: matches!(action, Action::Begin { .. }),
    })?);
    let factory = Arc::new(StoredComparisonRequestFactory::new(store.clone(), config)?);
    let service = RegistrationService::new(
        store,
        &profile,
        Arc::new(ComparisonHttpFlow::new(factory, transport)),
    );
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
