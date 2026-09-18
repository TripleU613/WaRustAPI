//! Operator tool and standalone proof harness for `whatsapp-api` (docs/design/README.md
//! section 20).
//!
//! Secrets never travel on the command line: the master key is read from an
//! environment variable and challenge answers are read from stdin.

// A command-line tool talks to its operator through stdout/stderr.
#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::io::{IsTerminal, Read};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use clap::{Parser, Subcommand, ValueEnum};
use whatsapp_api::model::ids::{AccountId, ChatId, ClientRequestId, EventCursor, LocalMessageId};
use whatsapp_api::model::jid::Jid;
use whatsapp_api::model::message::MessageContent;
use whatsapp_api::model::phone::PhoneNumber;
use whatsapp_api::model::send::SendRequest;
use whatsapp_api::registration::{ChallengeMethod, RegistrationStep, RequiredAction};
use whatsapp_api::session::AccountRecord;
use whatsapp_api::store::{ConsumerId, EnvKeySource, MasterKey, Table};
use whatsapp_api::{Error, ProtocolProfile, WhatsAppClient};
use zeroize::Zeroizing;

mod http;
#[cfg(feature = "experimental-transport")]
mod media;
#[cfg(feature = "experimental-transport")]
mod outbox_send;
#[cfg(feature = "experimental-signup")]
mod signup;
#[cfg(feature = "experimental-transport")]
mod transport;
#[cfg(feature = "experimental-transport")]
mod transport_receive;
#[cfg(feature = "experimental-transport")]
mod transport_send;

#[derive(Parser)]
#[command(
    name = "whatsapp-cli",
    version,
    about = "whatsapp-api operator tool (experimental, offline gates only)"
)]
struct Cli {
    /// Path of the account protocol database.
    #[arg(
        long,
        global = true,
        default_value = "protocol.redb",
        env = "WHATSAPP_API_STORE"
    )]
    store: PathBuf,
    /// Local account identifier.
    #[arg(
        long,
        global = true,
        default_value = "default",
        env = "WHATSAPP_API_ACCOUNT"
    )]
    account: String,
    /// Name of the environment variable holding the hex master key.
    #[arg(long, global = true, default_value = "WHATSAPP_API_MASTER_KEY")]
    key_env: String,
    /// Machine-readable JSON output where available.
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Send one private-JSON text, image or voice note through the comparison outbox.
    #[cfg(feature = "experimental-transport")]
    OutboxSend {
        #[arg(long)]
        config: PathBuf,
        #[arg(long)]
        message: PathBuf,
        #[arg(long)]
        allow_live_request: bool,
    },
    /// List ordinary incoming attachments by opaque selector, with content redacted.
    #[cfg(feature = "experimental-transport")]
    MediaList,
    /// Download/authenticate an ordinary image or voice note and open it locally.
    #[cfg(feature = "experimental-transport")]
    MediaOpen {
        #[arg(long)]
        config: PathBuf,
        #[arg(long)]
        id: String,
        #[arg(long)]
        allow_live_request: bool,
    },
    /// Maintain an authenticated comparison connection and process incoming messages.
    #[cfg(feature = "experimental-transport")]
    Listen {
        #[arg(long)]
        config: PathBuf,
        #[arg(long)]
        allow_live_request: bool,
        /// Stop after this many seconds; zero waits for Ctrl-C.
        #[arg(long, default_value_t = 0)]
        seconds: u64,
    },
    /// Display one downloaded view-once picture locally for 30 seconds, consuming it.
    #[cfg(feature = "experimental-transport")]
    LabOpenViewOnce,
    /// Download one authenticated view-once image as private ciphertext; do not open it.
    #[cfg(feature = "experimental-transport")]
    LabDownloadViewOnce {
        #[arg(long)]
        config: PathBuf,
        #[arg(long)]
        allow_live_request: bool,
    },
    /// Decrypt captured direct replies locally, without network I/O.
    #[cfg(feature = "experimental-transport")]
    LabInbox {
        /// Inspect saved plaintext protobuf structure without advancing sessions.
        #[arg(long)]
        inspect: bool,
    },
    /// One comparison-server connection; optional bounded primary login.
    #[cfg(feature = "experimental-transport")]
    LabTransport {
        /// Existing private registration JSON, used to bind the saved account.
        #[arg(long)]
        config: PathBuf,
        /// Authorize a single connection; there are no automatic retries.
        #[arg(long)]
        allow_live_request: bool,
        /// Send one primary login after certificate verification.
        #[arg(long)]
        login: bool,
        /// Private JSON: to, request_id, and text, image (PNG), or voice_note (Ogg/Opus).
        #[arg(long, requires = "login")]
        message: Option<PathBuf>,
        /// Preserve incoming stanzas for up to 30 seconds without sending a chat.
        #[arg(long, requires = "login", conflicts_with = "message")]
        receive: bool,
        /// Obtain and privately preserve one current media-upload route response.
        #[arg(long, requires = "login", conflicts_with_all = ["message", "receive"])]
        media_route: bool,
    },
    /// Explicit comparison-build signup experiment; may affect an existing account.
    #[cfg(feature = "experimental-signup")]
    LabSignup {
        /// Private JSON LabRegistrationConfig file; keep under ignored lab/.
        #[arg(long, default_value = "lab/requests.json")]
        config: PathBuf,
        /// Authorize this single invocation to contact WhatsApp.
        #[arg(long)]
        allow_live_request: bool,
        #[command(subcommand)]
        action: signup::Action,
    },
    /// Print a freshly generated master key (hex) to stdout. Store it in a
    /// secret manager and export it through the key environment variable.
    Keygen,
    /// Inspect the protocol profile and its evidence labels.
    Profile,
    /// Print the capability matrix.
    Capabilities,
    /// Serve the local HTTP control plane.
    Serve {
        /// Address to listen on. Non-loopback addresses require an explicit
        /// opt-in because the control plane does not implement authentication.
        #[arg(long, default_value = "127.0.0.1:8787", env = "WHATSAPP_API_BIND")]
        bind: SocketAddr,
        /// Permit listening on a non-loopback address.
        #[arg(long)]
        allow_non_loopback: bool,
    },
    /// Store operations.
    Store {
        #[command(subcommand)]
        cmd: StoreCmd,
    },
    /// Primary registration steps.
    Registration {
        #[command(subcommand)]
        cmd: RegCmd,
    },
    /// Attempt an authenticated connection (fails with a typed reason).
    Connect {
        /// Timeout in seconds.
        #[arg(long, default_value_t = 30)]
        timeout: u64,
    },
    /// Durably enqueue a direct text message and print its ticket.
    Send {
        /// Destination JID, e.g. 15551234567@s.whatsapp.net.
        #[arg(long)]
        to: String,
        /// Message text.
        #[arg(long)]
        text: String,
        /// Caller idempotency key.
        #[arg(long)]
        request_id: String,
    },
    /// Show the status of a queued message.
    Status {
        /// Local message id (32 hex chars) from the ticket.
        #[arg(long)]
        local_id: String,
    },
    /// Read durable events for a consumer.
    Events {
        /// Print event kinds only, without message content or identifiers.
        #[arg(long)]
        summary: bool,
        /// Consumer name.
        #[arg(long, default_value = "cli")]
        consumer: String,
        /// Maximum events.
        #[arg(long, default_value_t = 50)]
        limit: u32,
        /// Acknowledge the batch after printing it.
        #[arg(long)]
        ack: bool,
    },
}

#[derive(Subcommand)]
enum StoreCmd {
    /// Create the store (idempotent) and verify the master key opens it.
    Init,
    /// Print table counts, journal head, generation and registration status.
    Inspect,
    /// Rotate the master key: re-wrap the data keys under the key found in
    /// the given environment variable. The old key stops working immediately.
    RotateKey {
        /// Environment variable holding the new hex master key.
        #[arg(long)]
        new_key_env: String,
    },
}

#[derive(Subcommand)]
enum RegCmd {
    /// Start or resume a registration attempt.
    Begin {
        /// Country calling code (explicit; never guessed).
        #[arg(long)]
        country: u16,
        /// Phone number in E.164 form (`+<cc><national>`).
        #[arg(long)]
        phone: String,
        /// Challenge method.
        #[arg(long, value_enum, default_value_t = Method::Sms)]
        method: Method,
    },
    /// Show the current attempt.
    Status,
    /// Reconcile an attempt whose request outcome was lost.
    Reconcile,
    /// Discard a completed or blocked attempt (local only).
    Reset,
}

#[derive(Clone, Copy, ValueEnum)]
enum Method {
    Sms,
    Voice,
}

async fn open_client(cli: &Cli, create: bool) -> Result<WhatsAppClient, Error> {
    WhatsAppClient::builder()
        .store_path(cli.store.clone())
        .account_id(AccountId::new(cli.account.clone())?)
        .key_source(Arc::new(EnvKeySource::new(cli.key_env.clone())))
        .create_store(create)
        .build()
        .await
}

fn print_step(step: &RegistrationStep, json: bool) {
    if json {
        println!("{}", serde_json::json!({ "step": format!("{step:?}") }));
        return;
    }
    match step {
        RegistrationStep::Completed => println!("registration: completed"),
        RegistrationStep::Action(RequiredAction::EnterCode(h)) => {
            println!(
                "registration: enter the code delivered via {:?}",
                h.method()
            )
        }
        RegistrationStep::Action(RequiredAction::EnterPin(_)) => {
            println!("registration: enter the account PIN")
        }
        RegistrationStep::Action(RequiredAction::WaitForCooldown { until }) => {
            println!(
                "registration: cooling down until unix {}",
                until.as_unix_seconds()
            )
        }
        RegistrationStep::Blocked(reason) => println!("registration: BLOCKED — {reason:?}"),
        RegistrationStep::OutcomeUnknown => {
            println!("registration: outcome unknown; reconcile before retrying")
        }
        other => println!("registration: {other:?}"),
    }
}

async fn run(cli: Cli) -> Result<(), Error> {
    match &cli.cmd {
        #[cfg(feature = "experimental-transport")]
        Cmd::OutboxSend {
            config,
            message,
            allow_live_request,
        } => outbox_send::run(&cli, config, message, *allow_live_request).await,
        #[cfg(feature = "experimental-transport")]
        Cmd::MediaList => media::list(&cli),
        #[cfg(feature = "experimental-transport")]
        Cmd::MediaOpen {
            config,
            id,
            allow_live_request,
        } => media::open(&cli, config, id, *allow_live_request).await,
        #[cfg(feature = "experimental-transport")]
        Cmd::Listen {
            config,
            allow_live_request,
            seconds,
        } => transport_receive::listen(&cli, config, *allow_live_request, *seconds).await,
        #[cfg(feature = "experimental-transport")]
        Cmd::LabOpenViewOnce => transport_receive::open_view_once(&cli),
        #[cfg(feature = "experimental-transport")]
        Cmd::LabDownloadViewOnce {
            config,
            allow_live_request,
        } => transport_receive::download_view_once(&cli, config, *allow_live_request).await,
        #[cfg(feature = "experimental-transport")]
        Cmd::LabInbox { inspect } => transport_receive::run(&cli, *inspect).await,
        #[cfg(feature = "experimental-transport")]
        Cmd::LabTransport {
            config,
            allow_live_request,
            login,
            message,
            receive,
            media_route,
        } => {
            transport::run(
                &cli,
                config,
                *allow_live_request,
                *login,
                message.as_deref(),
                *receive,
                *media_route,
            )
            .await
        }
        #[cfg(feature = "experimental-signup")]
        Cmd::LabSignup {
            config,
            allow_live_request,
            action,
        } => signup::run(&cli, config, *allow_live_request, action).await,
        Cmd::Keygen => {
            let key = MasterKey::generate()?;
            if std::io::stdout().is_terminal() {
                eprintln!(
                    "warning: printing a secret to a terminal; clear your scrollback afterwards"
                );
            }
            // Expose the hex once; the value is zeroized when dropped.
            let hex = master_key_bytes(&key);
            println!("{}", String::from_utf8_lossy(&hex));
            Ok(())
        }
        Cmd::Profile => {
            let p = ProtocolProfile::android_reference()?;
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&p)
                        .map_err(|e| Error::invalid("profile", e.to_string()))?
                );
            } else {
                println!("profile: {}", p.id);
                println!(
                    "reference: {} {} (code {})",
                    p.reference.package, p.reference.version_name, p.reference.version_code
                );
                println!("apk sha256: {}", p.reference.apk_sha256);
                println!("production grade: {}", p.is_production_grade());
                println!("missing for connect: {:?}", p.missing_for_connect());
                println!(
                    "missing for registration: {:?}",
                    p.missing_for_registration()
                );
                println!(
                    "framing prefix: {:?} [{:?}]",
                    p.framing.length_prefix_bytes.value, p.framing.length_prefix_bytes.label
                );
                println!(
                    "noise pattern: {:?} [{:?}]",
                    p.noise.pattern.value, p.noise.pattern.label
                );
                println!(
                    "token dictionary: v{} [{:?}] {}",
                    p.tokens.version, p.tokens.label, p.tokens.sha256
                );
            }
            Ok(())
        }
        Cmd::Capabilities => {
            let p = ProtocolProfile::android_reference()?;
            let m = whatsapp_api::capabilities::CapabilityMatrix::current(p.id);
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&m)
                        .map_err(|e| Error::invalid("capabilities", e.to_string()))?
                );
            } else {
                println!(
                    "{:<20} {:<13} {:<14} note",
                    "capability", "level", "evidence"
                );
                for e in &m.entries {
                    println!(
                        "{:<20} {:<13} {:<14} {}",
                        format!("{:?}", e.capability),
                        format!("{:?}", e.level),
                        format!("{:?}", e.evidence),
                        e.note
                    );
                }
            }
            Ok(())
        }
        Cmd::Serve {
            bind,
            allow_non_loopback,
        } => http::serve(*bind, *allow_non_loopback).await,
        Cmd::Store { cmd } => match cmd {
            StoreCmd::Init => {
                let c = open_client(&cli, true).await?;
                println!(
                    "store ready at {} (generation {})",
                    c.store().path().display(),
                    c.store().generation().0
                );
                c.shutdown().await;
                Ok(())
            }
            StoreCmd::Inspect => {
                let c = open_client(&cli, false).await?;
                let store = c.store().clone();
                let report = store
                    .run_blocking(|s| {
                        s.read(|tx| {
                            let mut counts = serde_json::Map::new();
                            for t in Table::ALL {
                                counts.insert(format!("{t:?}"), serde_json::json!(tx.count(t)?));
                            }
                            let account = AccountRecord::load(tx)?;
                            Ok(serde_json::json!({
                                "account_id": s.account_id().as_str(),
                                "generation": s.generation().0,
                                "journal_head": tx.journal_head()?.0,
                                "registration": account.as_ref().map(|a| format!("{:?}", a.registration)).unwrap_or_else(|| "no account record".into()),
                                "tables": counts,
                            }))
                        })
                    })
                    .await?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report)
                        .map_err(|e| Error::invalid("report", e.to_string()))?
                );
                c.shutdown().await;
                Ok(())
            }
            StoreCmd::RotateKey { new_key_env } => {
                let c = open_client(&cli, false).await?;
                let store = c.store().clone();
                let source = EnvKeySource::new(new_key_env.clone());
                store
                    .run_blocking(move |s| s.rotate_master_key(&source))
                    .await?;
                println!("master key rotated; the previous key no longer opens this store");
                c.shutdown().await;
                Ok(())
            }
        },
        Cmd::Registration { cmd } => {
            let c = open_client(&cli, true).await?;
            let result = match cmd {
                RegCmd::Begin {
                    country,
                    phone,
                    method,
                } => {
                    let phone = PhoneNumber::from_e164_with_country(*country, phone)?;
                    let method = match method {
                        Method::Sms => ChallengeMethod::Sms,
                        Method::Voice => ChallengeMethod::Voice,
                    };
                    let step = c.registration().begin(phone, method).await?;
                    print_step(&step, cli.json);
                    if let RegistrationStep::Action(RequiredAction::EnterCode(handle)) = step {
                        eprint!("enter code (stdin): ");
                        let mut code = Zeroizing::new(String::new());
                        std::io::stdin().read_to_string(&mut code)?;
                        let trimmed = Zeroizing::new(code.trim().to_owned());
                        let next = c.verify_otp(handle, trimmed).await?;
                        print_step(&next, cli.json);
                    }
                    Ok(())
                }
                RegCmd::Status => {
                    match c.registration().current().await? {
                        Some(a) => println!("{a:?}"),
                        None => println!("no registration attempt"),
                    }
                    Ok(())
                }
                RegCmd::Reconcile => {
                    let step = c.registration().reconcile().await?;
                    print_step(&step, cli.json);
                    Ok(())
                }
                RegCmd::Reset => c.registration().reset().await,
            };
            c.shutdown().await;
            result
        }
        Cmd::Connect { timeout } => {
            let c = open_client(&cli, false).await?;
            let r = c.connect(Duration::from_secs(*timeout)).await;
            let status = c.connection_state().borrow().clone();
            println!(
                "connection state: {:?} (epoch {})",
                status.state, status.epoch.0
            );
            c.shutdown().await;
            r
        }
        Cmd::Send {
            to,
            text,
            request_id,
        } => {
            let c = open_client(&cli, false).await?;
            let req = SendRequest {
                client_request_id: ClientRequestId::new(request_id.clone())?,
                chat: ChatId::from_jid(&Jid::parse(to)?)?,
                content: MessageContent::Text {
                    body: text.clone(),
                    quoted: None,
                },
            };
            let ticket = c.send_message(req).await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&ticket)
                    .map_err(|e| Error::invalid("ticket", e.to_string()))?
            );
            eprintln!("note: durably queued; no transmission path exists for this profile yet");
            c.shutdown().await;
            Ok(())
        }
        Cmd::Status { local_id } => {
            let c = open_client(&cli, false).await?;
            let ticket = whatsapp_api::model::send::SendTicket {
                local_id: local_id.parse::<LocalMessageId>()?,
                client_request_id: ClientRequestId::new("lookup")?,
                wire_id: None,
                accepted_at: Default::default(),
            };
            let st = c.message_status(&ticket).await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&st)
                    .map_err(|e| Error::invalid("status", e.to_string()))?
            );
            c.shutdown().await;
            Ok(())
        }
        Cmd::Events {
            summary,
            consumer,
            limit,
            ack,
        } => {
            let c = open_client(&cli, false).await?;
            let engine = c.events(ConsumerId::new(consumer.clone())?);
            let after: EventCursor = engine.acknowledged_cursor().await?;
            let batch = engine.next_batch(after, *limit).await?;
            for e in &batch.events {
                if *summary {
                    println!("event: {}", e.event.kind());
                    continue;
                }
                println!(
                    "{}",
                    serde_json::to_string(e)
                        .map_err(|err| Error::invalid("event", err.to_string()))?
                );
            }
            eprintln!(
                "{} events after cursor {}; has_more={}",
                batch.events.len(),
                after.0,
                batch.has_more
            );
            if *ack && !batch.events.is_empty() {
                engine.ack(batch.ack_cursor).await?;
                eprintln!("acknowledged through {}", batch.ack_cursor.0);
            }
            drop(engine);
            c.shutdown().await;
            Ok(())
        }
    }
}

fn master_key_bytes(key: &MasterKey) -> Zeroizing<Vec<u8>> {
    // `MasterKey` only exposes hex through the CLI; keep the copy zeroized.
    Zeroizing::new(key.to_hex_bytes())
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();
    match run(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            match e {
                Error::Unsupported { .. } | Error::ProfileIncomplete { .. } => ExitCode::from(3),
                Error::NotRegistered | Error::RegistrationActionRequired(_) => ExitCode::from(4),
                Error::WrongStorageKey => ExitCode::from(5),
                _ => ExitCode::FAILURE,
            }
        }
    }
}
