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

use std::os::unix::fs::PermissionsExt;

mod cli_reference;
mod http;
#[cfg(feature = "experimental-transport")]
mod media;
#[cfg(feature = "experimental-transport")]
mod outbox_send;
mod profiles;
#[cfg(feature = "experimental-signup")]
mod signup;
mod token;
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
    /// Named account profile (`<profiles-dir>/<NAME>.json`) supplying the
    /// store, account, key variable and config defaults; explicit flags win.
    #[arg(long, global = true, env = "WHATSAPP_API_PROFILE")]
    profile: Option<String>,
    /// Directory holding account profiles.
    #[arg(
        long,
        global = true,
        default_value = "lab/profiles",
        env = "WHATSAPP_API_PROFILES_DIR"
    )]
    profiles_dir: PathBuf,
    /// Machine-readable JSON output where available.
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    cmd: Cmd,
    #[arg(skip)]
    loaded_profile: Option<profiles::Profile>,
}

impl Cli {
    /// Apply `--profile`: fill every global the operator left at its default.
    fn apply_profile(&mut self, matches: &clap::ArgMatches) -> Result<(), Error> {
        let Some(name) = self.profile.clone() else {
            return Ok(());
        };
        let profile = profiles::load(&self.profiles_dir, &name)?;
        let defaulted =
            |arg: &str| matches.value_source(arg) != Some(clap::parser::ValueSource::CommandLine);
        if defaulted("store") {
            self.store = profile.store.clone();
        }
        if defaulted("account") {
            self.account = profile.account.clone();
        }
        if defaulted("key_env") {
            self.key_env = profile.key_env.clone();
        }
        self.loaded_profile = Some(profile);
        Ok(())
    }

    /// The private config for a command: the explicit `--config`, else the
    /// profile's, else `fallback` when one exists for that command.
    #[cfg_attr(not(feature = "experimental-transport"), allow(dead_code))]
    fn config_path(
        &self,
        given: Option<&PathBuf>,
        fallback: Option<&str>,
    ) -> Result<PathBuf, Error> {
        if let Some(path) = given {
            return Ok(path.clone());
        }
        if let Some(path) = self.loaded_profile.as_ref().and_then(|p| p.config.clone()) {
            return Ok(path);
        }
        match fallback {
            Some(path) => Ok(PathBuf::from(path)),
            None => Err(Error::invalid(
                "config",
                "pass --config or select a --profile that names one",
            )),
        }
    }

    /// A copy of these globals bound to another named profile (for commands
    /// that drive two accounts at once).
    #[cfg_attr(not(feature = "experimental-transport"), allow(dead_code))]
    fn for_profile(&self, name: &str) -> Result<Cli, Error> {
        let profile = profiles::load(&self.profiles_dir, name)?;
        Ok(Cli {
            store: profile.store.clone(),
            account: profile.account.clone(),
            key_env: profile.key_env.clone(),
            profile: Some(name.to_owned()),
            profiles_dir: self.profiles_dir.clone(),
            json: self.json,
            cmd: Cmd::Capabilities,
            loaded_profile: Some(profile),
        })
    }
}

#[derive(Subcommand)]
enum Cmd {
    /// Send one private-JSON text, image or voice note through the comparison outbox.
    #[cfg(feature = "experimental-transport")]
    OutboxSend {
        /// Private request configuration; defaults to the profile's.
        #[arg(long)]
        config: Option<PathBuf>,
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
        /// Private request configuration; defaults to the profile's.
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long)]
        allow_live_request: bool,
        /// Stop after this many seconds; zero waits for Ctrl-C.
        #[arg(long, default_value_t = 0)]
        seconds: u64,
        /// Also transmit every eligible queued outbox intent, from any
        /// namespace, during this session. Inspect `outbox-pending` first.
        #[arg(long)]
        send_outbox: bool,
    },
    /// List queued outbox intents that `listen --send-outbox` would transmit (offline).
    #[cfg(feature = "experimental-transport")]
    OutboxPending,
    /// Show one-time prekey counts and upload batches (offline, redacted).
    #[cfg(feature = "experimental-transport")]
    Prekeys,
    /// Show queued explicit receipts and their server outcomes (offline, redacted).
    #[cfg(feature = "experimental-transport")]
    Receipts,
    /// Show queued one-shot actions (block list, reports, lookups) and their outcomes (offline, redacted).
    #[cfg(feature = "experimental-transport")]
    Actions {
        /// Print the request and reply stanzas of one action (numbers masked).
        #[arg(long)]
        show: Option<String>,
    },
    /// Group management over the durable action queue (offline; a session transmits).
    #[cfg(feature = "experimental-transport")]
    Group {
        #[command(subcommand)]
        cmd: GroupCmd,
    },
    /// Manage named account profiles (offline): list, add, show, new, status.
    Accounts {
        #[command(subcommand)]
        cmd: AccountsCmd,
    },
    /// Token derivation and verification.
    Token {
        #[command(subcommand)]
        cmd: token::TokenCmd,
    },
    /// Two-account lab check: send a text from one profile and confirm the
    /// other profile receives it, running both sessions at once.
    #[cfg(feature = "experimental-transport")]
    LabPair {
        /// Sending profile name.
        #[arg(long)]
        from: String,
        /// Receiving profile name.
        #[arg(long)]
        to: String,
        /// Text to send.
        #[arg(long, default_value = "pair check")]
        text: String,
        /// Session length for both accounts.
        #[arg(long, default_value_t = 45)]
        seconds: u64,
        #[arg(long)]
        allow_live_request: bool,
    },
    /// Durably queue a block of a contact (offline; `listen --send-outbox` transmits it).
    #[cfg(feature = "experimental-transport")]
    Block {
        /// Contact JID, e.g. 15551234567@s.whatsapp.net.
        #[arg(long)]
        chat: String,
    },
    /// Durably queue an unblock of a contact (offline; `listen --send-outbox` transmits it).
    #[cfg(feature = "experimental-transport")]
    Unblock {
        /// Contact JID, e.g. 15551234567@s.whatsapp.net.
        #[arg(long)]
        chat: String,
    },
    /// Show the saved block list (offline, numbers masked), optionally queuing a fetch.
    #[cfg(feature = "experimental-transport")]
    Blocklist {
        /// Also durably queue a blocklist fetch (`listen --send-outbox` transmits it).
        #[arg(long)]
        refresh: bool,
        /// Forget the saved list hash first so the fetch returns the full list.
        #[arg(long)]
        full: bool,
    },
    /// Durably queue a spam report against a contact (offline; not live-verified).
    #[cfg(feature = "experimental-transport")]
    Report {
        /// Contact JID, e.g. 15551234567@s.whatsapp.net.
        #[arg(long)]
        chat: String,
        /// Entry-point vocabulary string; defaults to "chat".
        #[arg(long)]
        flow: Option<String>,
        /// Whether this was a known chat before reporting.
        #[arg(long)]
        known_chat: Option<bool>,
    },
    /// Durably enqueue a reaction to a message; `listen --send-outbox` transmits it.
    React {
        /// Chat JID, e.g. 15551234567@s.whatsapp.net.
        #[arg(long)]
        chat: String,
        /// Wire id of the message being reacted to (as shown by `events`).
        #[arg(long)]
        id: String,
        /// Reaction text (an emoji). Omit with --remove to clear a reaction.
        #[arg(long)]
        emoji: Option<String>,
        /// Remove this account's reaction instead of setting one.
        #[arg(long)]
        remove: bool,
        /// The target message was sent by this account (default: by the contact).
        #[arg(long)]
        ours: bool,
        /// Caller idempotency key; defaults to `react-<id>-<emoji-hex or remove>`.
        #[arg(long)]
        request_id: Option<String>,
        /// Only report which identity the reaction key would use; enqueue nothing.
        #[arg(long)]
        dry_run: bool,
    },
    /// Durably enqueue a poll; `listen --send-outbox` transmits it.
    Poll {
        /// Chat JID.
        #[arg(long)]
        chat: String,
        /// Poll question.
        #[arg(long)]
        name: String,
        /// Options, two to twelve.
        #[arg(long = "option", required = true)]
        options: Vec<String>,
        /// Options a voter may select; 0 (default) means any number.
        #[arg(long, default_value_t = 0)]
        selectable: u32,
        /// Caller idempotency key; defaults to a content digest.
        #[arg(long)]
        request_id: Option<String>,
    },
    /// Durably enqueue a vote on a poll; `listen --send-outbox` transmits it.
    Vote {
        /// Chat JID.
        #[arg(long)]
        chat: String,
        /// Wire id of the poll message.
        #[arg(long)]
        id: String,
        /// Selected option names; none retracts the vote.
        #[arg(long = "option")]
        options: Vec<String>,
        /// The poll was created by this account (default: by the contact).
        #[arg(long)]
        ours: bool,
        /// Caller idempotency key; defaults to a content digest.
        #[arg(long)]
        request_id: Option<String>,
    },
    /// Durably enqueue a per-chat disappearing-message timer change (E2E
    /// protocolMessage); `listen --send-outbox` transmits it.
    Disappearing {
        /// Chat JID.
        #[arg(long)]
        chat: String,
        /// New timer in seconds (0 = off; the UI offers 0, 86400, 604800, 7776000).
        #[arg(long)]
        seconds: u32,
        /// Caller idempotency key; defaults to a chat/timer digest (no phone number).
        #[arg(long)]
        request_id: Option<String>,
    },
    /// Durably enqueue keeping, or un-keeping, a message despite the chat's
    /// disappearing timer; `listen --send-outbox` transmits it.
    Keep {
        /// Chat JID.
        #[arg(long)]
        chat: String,
        /// Wire id of the message to keep/un-keep.
        #[arg(long)]
        id: String,
        /// The target message was sent by this account (default: by the contact).
        #[arg(long)]
        ours: bool,
        /// Undo a previous keep instead of setting one.
        #[arg(long)]
        undo: bool,
        /// Caller idempotency key; defaults to a content digest.
        #[arg(long)]
        request_id: Option<String>,
    },
    /// Durably enqueue an advanced chat privacy ("limit sharing") change;
    /// `listen --send-outbox` transmits it.
    ChatPrivacy {
        /// Chat JID.
        #[arg(long)]
        chat: String,
        /// Turn limit sharing on.
        #[arg(long)]
        on: bool,
        /// Turn limit sharing off.
        #[arg(long)]
        off: bool,
        /// Caller idempotency key; defaults to a chat/state digest (no phone number).
        #[arg(long)]
        request_id: Option<String>,
    },
    /// Queue the account-wide default disappearing timer for new chats
    /// (`xmlns=disappearing_mode` IQ); a live session transmits it.
    #[cfg(feature = "experimental-transport")]
    DisappearingDefault {
        /// New default timer in seconds.
        #[arg(long)]
        seconds: u32,
    },
    /// Queue a quoted reply to a message in a direct chat.
    Reply {
        /// Chat JID.
        #[arg(long)]
        chat: String,
        /// Wire id of the message being replied to (as shown by `events`).
        #[arg(long)]
        id: String,
        /// Reply text.
        #[arg(long)]
        text: String,
        /// The quoted message was sent by this account (default: by the contact).
        #[arg(long)]
        ours: bool,
        /// Caller idempotency key; defaults to a content digest.
        #[arg(long)]
        request_id: Option<String>,
    },
    /// Re-project authenticated incoming records into events with current parsers (offline).
    #[cfg(feature = "experimental-transport")]
    Reproject,
    /// Show a contact's saved device list, or queue a device sync for the next session.
    #[cfg(feature = "experimental-transport")]
    Devices {
        /// Contact JID, e.g. 15551234567@s.whatsapp.net.
        #[arg(long)]
        chat: String,
        /// Record the contact for the next `usync devices` query (offline).
        #[arg(long)]
        request: bool,
    },
    /// Manage the local address book and its WhatsApp-existence checks (offline).
    #[cfg(feature = "experimental-transport")]
    Contacts {
        #[command(subcommand)]
        action: ContactsCmd,
    },
    /// Clear a direct chat's locally decrypted message bodies (offline, local-only:
    /// see docs/protocol/chat-actions.md section 6 for the deferred companion sync).
    #[cfg(feature = "experimental-transport")]
    ClearChat {
        /// Chat JID, e.g. 15551234567@s.whatsapp.net.
        #[arg(long)]
        chat: String,
    },
    /// Queue explicit read receipts for authenticated incoming messages (offline;
    /// the next `listen` session sends them once).
    #[cfg(feature = "experimental-transport")]
    MarkRead {
        /// Private request configuration; defaults to the profile's.
        #[arg(long)]
        config: Option<PathBuf>,
        /// Sender JID, e.g. 15551234567@s.whatsapp.net.
        #[arg(long)]
        chat: String,
        /// One or more incoming message ids as shown by `events`.
        #[arg(long = "id", required = true)]
        ids: Vec<String>,
    },
    /// Display one downloaded view-once picture locally for 30 seconds, consuming it.
    #[cfg(feature = "experimental-transport")]
    LabOpenViewOnce,
    /// Download one authenticated view-once image as private ciphertext; do not open it.
    #[cfg(feature = "experimental-transport")]
    LabDownloadViewOnce {
        /// Private request configuration; defaults to the profile's.
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long)]
        allow_live_request: bool,
    },
    /// Decrypt captured direct replies locally, without network I/O.
    #[cfg(feature = "experimental-transport")]
    LabInbox {
        /// Inspect saved plaintext protobuf structure without advancing sessions.
        #[arg(long)]
        inspect: bool,
        /// Re-run preserved incoming message captures through the normal receive
        /// path, offline. The server delivers a queued stanza once, so a stanza
        /// refused by a since-fixed guard is only recoverable from its capture.
        /// Already-committed messages dedupe; nothing is sent and no receipt goes out.
        #[arg(long)]
        replay_captures: bool,
    },
    /// One comparison-server connection; optional bounded primary login.
    #[cfg(feature = "experimental-transport")]
    LabTransport {
        /// Existing private registration JSON, used to bind the saved account.
        /// Private request configuration; defaults to the profile's.
        #[arg(long)]
        config: Option<PathBuf>,
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
    /// Submit a ban appeal for the active account via ACS GraphQL.
    #[cfg(feature = "experimental-signup")]
    BanAppeal {
        /// Private request configuration; defaults to the profile's.
        #[arg(long)]
        config: Option<PathBuf>,
        /// Authorize this single invocation to contact acs.whatsapp.com.
        #[arg(long)]
        allow_live_request: bool,
        /// Persistent query doc_id for SubmitAnAppealForIP (APK-derived candidate).
        #[arg(long)]
        doc_id: Option<u64>,
        /// Optional appeal reason text included in the request.
        #[arg(long)]
        message: Option<String>,
        /// Use this appeal token instead of the one saved from the failure node.
        #[arg(long)]
        appeal_token: Option<String>,
    },
    /// Explicit comparison-build signup experiment; may affect an existing account.
    #[cfg(feature = "experimental-signup")]
    LabSignup {
        /// Private JSON LabRegistrationConfig file; keep under ignored lab/.
        /// Defaults to the profile's config, else lab/requests.json.
        #[arg(long)]
        config: Option<PathBuf>,
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
    /// Cancel a queued message before transmission (offline; never a revocation).
    Cancel {
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

/// Local address book actions (offline; `Check` also queues a `usync` action).
#[derive(Subcommand)]
#[cfg(feature = "experimental-transport")]
enum ContactsCmd {
    /// List saved contacts.
    List,
    /// Add a new contact.
    Add {
        /// Phone number, `+E.164` or bare digits, 6-15 digits.
        #[arg(long)]
        phone: String,
        /// Display name.
        #[arg(long)]
        name: String,
        /// Optional given name.
        #[arg(long)]
        first_name: Option<String>,
    },
    /// Edit an existing contact's name.
    Edit {
        #[arg(long)]
        phone: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        first_name: Option<String>,
    },
    /// Remove a saved contact.
    Remove {
        #[arg(long)]
        phone: String,
    },
    /// Queue a `usync` WhatsApp-existence check for a number and show any
    /// already-saved status; a session transmits the queued action.
    Check {
        #[arg(long)]
        phone: String,
    },
    /// Queue a full `usync` sync of every saved contact, registering this
    /// account's address book with the server (offline; a session transmits it).
    /// Unlike `check`, which is a one-number existence query, this uses the
    /// APK's `full`/`interactive` mode pair.
    Sync,
    /// Queue a read-only app-state fetch for every collection at version 0
    /// (offline; a session transmits it). Carries no patch and mutates nothing;
    /// the reply gives the server's authoritative collection versions.
    FetchAppState,
    /// Queue an app-state `contactAction` patch publishing every saved contact,
    /// so companion devices receive this address book (offline; a session
    /// transmits it). Mints this device's app-state key if it has none.
    PublishAppState,
}

#[derive(Subcommand)]
#[cfg(feature = "experimental-transport")]
enum GroupCmd {
    /// Queue a group creation with the given subject and participants.
    Create {
        #[arg(long)]
        subject: String,
        /// Participant JIDs (phone-number or LID form).
        #[arg(long = "participant", required = true)]
        participants: Vec<String>,
    },
    /// Queue an interactive group info query.
    Info {
        #[arg(long)]
        chat: String,
    },
    /// Queue a subject change.
    Subject {
        #[arg(long)]
        chat: String,
        #[arg(long)]
        text: String,
    },
    /// Queue a description change (empty text deletes).
    Description {
        #[arg(long)]
        chat: String,
        #[arg(long, default_value = "")]
        text: String,
        /// The current description id when replacing one.
        #[arg(long)]
        prev: Option<String>,
    },
    /// Queue adding participants.
    Add {
        #[arg(long)]
        chat: String,
        #[arg(long = "participant", required = true)]
        participants: Vec<String>,
    },
    /// Queue removing participants.
    Remove {
        #[arg(long)]
        chat: String,
        #[arg(long = "participant", required = true)]
        participants: Vec<String>,
    },
    /// Queue promoting participants to admin.
    Promote {
        #[arg(long)]
        chat: String,
        #[arg(long = "participant", required = true)]
        participants: Vec<String>,
    },
    /// Queue demoting admins.
    Demote {
        #[arg(long)]
        chat: String,
        #[arg(long = "participant", required = true)]
        participants: Vec<String>,
    },
    /// Queue leaving a group.
    Leave {
        #[arg(long)]
        chat: String,
    },
    /// Queue fetching (or, with --revoke, resetting) the invite link.
    Invite {
        #[arg(long)]
        chat: String,
        #[arg(long)]
        revoke: bool,
    },
    /// Apply settled replies and list the stored group snapshots (numbers masked).
    List,
}

#[derive(Subcommand)]
enum AccountsCmd {
    /// List the profiles in the profiles directory.
    List,
    /// Add a profile (refuses to overwrite).
    Add {
        #[arg(long)]
        name: String,
        /// Protocol database path for this account.
        #[arg(long)]
        store: PathBuf,
        /// Local account id inside that store.
        #[arg(long)]
        account: String,
        /// Private request configuration path.
        #[arg(long)]
        config: Option<PathBuf>,
        /// Environment variable holding this account's master key.
        #[arg(long)]
        key_env: String,
    },
    /// Show one profile.
    Show {
        #[arg(long)]
        name: String,
    },
    /// Create a new account with config, master key and profile.
    New {
        /// Account name.
        #[arg(long)]
        name: String,
        /// Country calling code.
        #[arg(long)]
        country_code: u16,
        /// National phone number digits.
        #[arg(long)]
        national_number: String,
        /// Path to APK or material archive.
        #[arg(long, env = "WHATSAPP_API_TOKEN_MATERIAL")]
        material: PathBuf,
        /// Account directory (defaults to `lab/accounts/<NAME>`).
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Verification delivery method.
        #[arg(long, value_enum, default_value = "sms")]
        method: Method,
    },
    /// Show account status (registration, credentials, queues).
    Status {
        /// Account name (all accounts if omitted).
        #[arg(long)]
        name: Option<String>,
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

#[derive(Debug, Subcommand)]
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

#[derive(Clone, Copy, Debug, ValueEnum)]
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

fn accounts_new(
    cli: &Cli,
    name: &str,
    country_code: u16,
    national_number: &str,
    material_path: &std::path::Path,
    dir: Option<&PathBuf>,
    method: &Method,
) -> Result<(), Error> {
    let account_dir = match dir {
        Some(d) => d.clone(),
        None => PathBuf::from(format!("lab/accounts/{}", name)),
    };

    if account_dir.exists() {
        return Err(Error::invalid(
            "accounts",
            format!("{} already exists", account_dir.display()),
        ));
    }

    std::fs::create_dir_all(&account_dir).map_err(|e| {
        Error::invalid(
            "accounts",
            format!("cannot create {}: {e}", account_dir.display()),
        )
    })?;
    std::fs::set_permissions(&account_dir, std::fs::Permissions::from_mode(0o700)).map_err(
        |e| {
            Error::invalid(
                "accounts",
                format!("cannot chmod {}: {e}", account_dir.display()),
            )
        },
    )?;

    let mut key_bytes = [0u8; 32];
    getrandom::fill(&mut key_bytes)
        .map_err(|e| Error::invalid("accounts", format!("entropy source failed: {e}")))?;
    let key_hex = hex::encode(key_bytes);

    let key_path = account_dir.join("master.key");
    std::fs::write(&key_path, &key_hex).map_err(|e| {
        Error::invalid(
            "accounts",
            format!("cannot write {}: {e}", key_path.display()),
        )
    })?;
    std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600)).map_err(|e| {
        Error::invalid(
            "accounts",
            format!("cannot chmod {}: {e}", key_path.display()),
        )
    })?;

    let phone = PhoneNumber::new(country_code, national_number)?;
    let material = whatsapp_api::registration::token_material::TokenMaterial::open(material_path)?;
    let token = material.derive(&phone)?;

    let mut config: serde_json::Value =
        serde_json::from_str(whatsapp_api::registration::request::TEMPLATE_JSON)
            .map_err(|e| Error::invalid("accounts", format!("template JSON parse failed: {e}")))?;

    config["phone"]["country_code"] = serde_json::json!(country_code);
    config["phone"]["national_number"] = serde_json::json!(national_number);
    config["token"] = serde_json::json!(token.as_str());

    let mut reg_id_bytes = [0u8; 4];
    getrandom::fill(&mut reg_id_bytes)
        .map_err(|e| Error::invalid("accounts", format!("entropy source failed: {e}")))?;
    let reg_id = u32::from_le_bytes(reg_id_bytes) % 16380 + 1;
    config["registration_id"] = serde_json::json!(reg_id);
    config["signed_pre_key_id"] = serde_json::json!(1);
    config["method"] = serde_json::json!(match method {
        Method::Sms => "sms",
        Method::Voice => "voice",
    });

    let config_path = account_dir.join("request.json");
    let config_json = serde_json::to_string_pretty(&config)
        .map_err(|e| Error::invalid("accounts", format!("config JSON serialize failed: {e}")))?;
    std::fs::write(&config_path, config_json).map_err(|e| {
        Error::invalid(
            "accounts",
            format!("cannot write {}: {e}", config_path.display()),
        )
    })?;
    std::fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o600)).map_err(
        |e| {
            Error::invalid(
                "accounts",
                format!("cannot chmod {}: {e}", config_path.display()),
            )
        },
    )?;

    let store_path = account_dir.join("signup.redb");
    let profile_name = name;
    let key_env = format!(
        "WHATSAPP_API_MASTER_KEY_{}",
        name.to_uppercase()
            .replace('-', "_")
            .replace(|c: char| !c.is_alphanumeric() && c != '_', "_")
    );

    let profile = profiles::Profile {
        store: store_path.clone(),
        account: format!("signup-{}", name),
        config: Some(config_path.clone()),
        key_env: key_env.clone(),
    };

    let profile_path = profiles::save(&cli.profiles_dir, profile_name, &profile)?;

    println!("accounts new: created {}", account_dir.display());
    println!("  master.key (mode 0600): {}", key_path.display());
    println!("  request.json (mode 0600): {}", config_path.display());
    println!("  profile: {}", profile_path.display());
    println!();
    println!("export {}=$(cat {})", key_env, key_path.display());
    println!(
        "whatsapp-cli --profile {} lab-signup --allow-live-request begin",
        profile_name
    );

    Ok(())
}

fn accounts_status(cli: &Cli, name: Option<&str>) -> Result<(), Error> {
    use whatsapp_api::store::OpenOptions;

    fn mask_phone(_cc: u16, nn: &str) -> String {
        let digits: String = nn.chars().filter(char::is_ascii_digit).collect();
        if digits.len() <= 3 {
            return "*".repeat(digits.len());
        }
        format!(
            "{}{}",
            "*".repeat(digits.len() - 3),
            &digits[digits.len() - 3..]
        )
    }

    let mut results = Vec::new();

    let profiles_to_check: Vec<(String, profiles::Profile)> = if let Some(name) = name {
        vec![(name.to_owned(), profiles::load(&cli.profiles_dir, name)?)]
    } else {
        profiles::list(&cli.profiles_dir)?
    };

    for (profile_name, profile) in profiles_to_check {
        let mut entry = serde_json::json!({
            "name": profile_name,
            "store": profile.store.display().to_string(),
            "key_env": &profile.key_env,
        });

        let key_value = std::env::var(&profile.key_env).ok();
        if key_value.is_none() {
            entry["locked"] = serde_json::json!(format!("{} not set", profile.key_env));
            results.push(entry);
            continue;
        }

        let account_id = AccountId::new(profile.account)?;
        let key_source = Arc::new(EnvKeySource::new(profile.key_env.clone()));
        let opts = OpenOptions {
            path: profile.store.clone(),
            account_id,
            key_source,
            create: false,
        };

        match whatsapp_api::store::ProtocolStore::open(opts) {
            Ok(store) => {
                let status_result = store.read(|tx| {
                    use whatsapp_api::session::AccountRecord;
                    use whatsapp_api::store::Table;

                    let account = AccountRecord::load(tx)?;

                    let registered = account.as_ref().map(|a| a.is_registered()).unwrap_or(false);
                    let last_authenticated = account
                        .as_ref()
                        .and_then(|a| a.last_authenticated_at)
                        .map(|t| t.as_unix_seconds().to_string())
                        .unwrap_or_else(|| "never".to_string());
                    let phone = account
                        .as_ref()
                        .and_then(|a| a.phone.as_ref())
                        .map(|p| mask_phone(p.country_code(), p.national_number()));

                    let appeal_token_saved = account
                        .as_ref()
                        .map(|a| a.credentials.iter().any(|c| c.name == "noise_appeal_token"))
                        .unwrap_or(false);

                    let outbox_pending = tx.count(Table::Outbox)?;

                    Ok((
                        registered,
                        last_authenticated,
                        phone,
                        appeal_token_saved,
                        outbox_pending,
                    ))
                });

                match status_result {
                    Ok((registered, last_auth, phone_opt, appeal, outbox)) => {
                        entry["locked"] = serde_json::json!(false);
                        if let Some(phone) = phone_opt {
                            entry["phone"] = serde_json::json!(phone);
                        }
                        entry["registration_state"] = serde_json::json!("none");
                        entry["registered"] = serde_json::json!(registered);
                        entry["last_authenticated_at"] = serde_json::json!(last_auth);
                        entry["appeal_token_saved"] =
                            serde_json::json!(if appeal { "yes" } else { "no" });
                        entry["outbox_pending"] = serde_json::json!(outbox);
                        entry["actions_queued"] = serde_json::json!(0);
                        entry["login_checkpoint"] = serde_json::json!("none");

                        #[cfg(feature = "experimental-transport")]
                        {
                            use whatsapp_api::transport::connection;
                            if let Ok(login_status) = connection::last_login_status(&store) {
                                entry["login_checkpoint"] = serde_json::json!(
                                    login_status.unwrap_or_else(|| "none".to_string())
                                );
                            }
                            if let Ok(actions) = whatsapp_api::transport::actions::outcomes(&store)
                            {
                                entry["actions_queued"] = serde_json::json!(actions.len());
                            }
                        }

                        results.push(entry);
                    }
                    Err(e) => {
                        entry["error"] = serde_json::json!(e.to_string());
                        results.push(entry);
                    }
                }
            }
            Err(e) => {
                entry["error"] = serde_json::json!(e.to_string());
                results.push(entry);
            }
        }
    }

    if cli.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&results)
                .map_err(|e| Error::invalid("accounts", e.to_string()))?
        );
    } else {
        for entry in results {
            let name = entry["name"].as_str().unwrap_or("?");
            if let Some(locked_msg) = entry.get("locked").and_then(|v| v.as_str()) {
                println!("{name}: locked ({locked_msg})");
            } else {
                let store = entry["store"].as_str().unwrap_or("?");
                let phone = entry
                    .get("phone")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let registered = entry["registered"].as_bool().unwrap_or(false);
                let last_auth = entry["last_authenticated_at"].as_str().unwrap_or("?");
                let appeal = entry["appeal_token_saved"].as_str().unwrap_or("no");
                let outbox = entry["outbox_pending"].as_u64().unwrap_or(0);
                let actions = entry["actions_queued"].as_u64().unwrap_or(0);
                let registration_state = entry["registration_state"].as_str().unwrap_or("?");
                let login_checkpoint = entry["login_checkpoint"].as_str().unwrap_or("?");

                println!(
                    "{name}: store={store} phone={phone} registered={registered} registration_state={registration_state} last_authenticated_at={last_auth} login_checkpoint={login_checkpoint} appeal_token_saved={appeal} outbox_pending={outbox} actions_queued={actions}"
                );
            }
        }
    }

    Ok(())
}

fn accounts(cli: &Cli, cmd: &AccountsCmd) -> Result<(), Error> {
    let describe = |name: &str, p: &profiles::Profile| {
        format!(
            "{name}: store={} account={} key_env={} config={} store_exists={}",
            p.store.display(),
            p.account,
            p.key_env,
            p.config
                .as_ref()
                .map(|c| c.display().to_string())
                .unwrap_or_else(|| "-".into()),
            p.store.exists()
        )
    };
    match cmd {
        AccountsCmd::List => {
            let all = profiles::list(&cli.profiles_dir)?;
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&all)
                        .map_err(|e| Error::invalid("accounts", e.to_string()))?
                );
                return Ok(());
            }
            println!(
                "accounts: {} profile(s) in {}",
                all.len(),
                cli.profiles_dir.display()
            );
            for (name, p) in &all {
                println!("  {}", describe(name, p));
            }
            Ok(())
        }
        AccountsCmd::Add {
            name,
            store,
            account,
            config,
            key_env,
        } => {
            let path = profiles::save(
                &cli.profiles_dir,
                name,
                &profiles::Profile {
                    store: store.clone(),
                    account: account.clone(),
                    config: config.clone(),
                    key_env: key_env.clone(),
                },
            )?;
            println!("accounts: wrote {}", path.display());
            Ok(())
        }
        AccountsCmd::Show { name } => {
            let p = profiles::load(&cli.profiles_dir, name)?;
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&p)
                        .map_err(|e| Error::invalid("accounts", e.to_string()))?
                );
            } else {
                println!("{}", describe(name, &p));
            }
            Ok(())
        }
        AccountsCmd::New {
            name,
            country_code,
            national_number,
            material,
            dir,
            method,
        } => accounts_new(
            cli,
            name,
            *country_code,
            national_number,
            material,
            dir.as_ref(),
            method,
        ),
        AccountsCmd::Status { name } => accounts_status(cli, name.as_deref()),
    }
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
        RegistrationStep::Action(RequiredAction::RequestEmailCode(_)) => {
            println!(
                "registration: the server requires the account's email second factor; run `lab-signup --allow-live-request request-email-code [--email <address>]`"
            )
        }
        RegistrationStep::Action(RequiredAction::EnterEmailCode(_)) => {
            println!(
                "registration: enter the code delivered to the account's email (`verify-email-code` on redirected stdin)"
            )
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
        } => {
            let config = cli.config_path(config.as_ref(), None)?;
            outbox_send::run(&cli, &config, message, *allow_live_request).await
        }
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
            send_outbox,
        } => {
            let config = cli.config_path(config.as_ref(), None)?;
            transport_receive::listen(&cli, &config, *allow_live_request, *seconds, *send_outbox)
                .await
        }
        #[cfg(feature = "experimental-transport")]
        Cmd::OutboxPending => transport_receive::outbox_pending(&cli),
        #[cfg(feature = "experimental-transport")]
        Cmd::Prekeys => transport_receive::prekeys_status(&cli),
        #[cfg(feature = "experimental-transport")]
        Cmd::Receipts => transport_receive::receipts_status(&cli),
        #[cfg(feature = "experimental-transport")]
        Cmd::Actions { show } => transport_receive::actions_status(&cli, show.as_deref()),
        #[cfg(feature = "experimental-transport")]
        Cmd::Group { cmd } => transport_receive::group(&cli, cmd),
        Cmd::Accounts { cmd } => accounts(&cli, cmd),
        Cmd::Token { cmd } => token::run(cmd),
        #[cfg(feature = "experimental-transport")]
        Cmd::LabPair {
            from,
            to,
            text,
            seconds,
            allow_live_request,
        } => transport_receive::lab_pair(&cli, from, to, text, *seconds, *allow_live_request).await,
        #[cfg(feature = "experimental-transport")]
        Cmd::Block { chat } => transport_receive::block(&cli, chat),
        #[cfg(feature = "experimental-transport")]
        Cmd::Unblock { chat } => transport_receive::unblock(&cli, chat),
        #[cfg(feature = "experimental-transport")]
        Cmd::Blocklist { refresh, full } => {
            transport_receive::blocklist_status(&cli, *refresh, *full)
        }
        #[cfg(feature = "experimental-transport")]
        Cmd::Report {
            chat,
            flow,
            known_chat,
        } => transport_receive::report(&cli, chat, flow.as_deref(), *known_chat),
        #[cfg(feature = "experimental-transport")]
        Cmd::Devices { chat, request } => transport_receive::devices(&cli, chat, *request),
        #[cfg(feature = "experimental-transport")]
        Cmd::Contacts { action } => transport_receive::contacts(&cli, action),
        #[cfg(feature = "experimental-transport")]
        Cmd::ClearChat { chat } => transport_receive::clear_chat(&cli, chat),
        #[cfg(feature = "experimental-transport")]
        Cmd::MarkRead { config, chat, ids } => {
            let config = cli.config_path(config.as_ref(), None)?;
            transport_receive::mark_read(&cli, &config, chat, ids).await
        }
        #[cfg(feature = "experimental-transport")]
        Cmd::LabOpenViewOnce => transport_receive::open_view_once(&cli),
        #[cfg(feature = "experimental-transport")]
        Cmd::LabDownloadViewOnce {
            config,
            allow_live_request,
        } => {
            let config = cli.config_path(config.as_ref(), None)?;
            transport_receive::download_view_once(&cli, &config, *allow_live_request).await
        }
        #[cfg(feature = "experimental-transport")]
        Cmd::LabInbox {
            inspect,
            replay_captures,
        } => transport_receive::run(&cli, *inspect, *replay_captures).await,
        #[cfg(feature = "experimental-transport")]
        Cmd::LabTransport {
            config,
            allow_live_request,
            login,
            message,
            receive,
            media_route,
        } => {
            let config = cli.config_path(config.as_ref(), None)?;
            transport::run(
                &cli,
                &config,
                *allow_live_request,
                *login,
                message.as_deref(),
                *receive,
                *media_route,
            )
            .await
        }
        #[cfg(feature = "experimental-signup")]
        Cmd::BanAppeal {
            config,
            allow_live_request,
            doc_id,
            message,
            appeal_token,
        } => {
            use whatsapp_api::transport::ban_appeal::{
                BanAppealInput, read_stored_appeal_token, submit_ban_appeal,
            };
            use zeroize::Zeroizing;
            if !allow_live_request {
                return Err(Error::invalid(
                    "ban_appeal",
                    "pass --allow-live-request to contact acs.whatsapp.com",
                ));
            }
            let config_path = cli.config_path(config.as_ref(), Some("lab/requests.json"))?;
            let lab_config = signup::config(&config_path)?;
            let c = open_client(&cli, false).await?;
            let store = c.store().clone();
            let token = if let Some(t) = appeal_token {
                Zeroizing::new(t.clone())
            } else {
                match read_stored_appeal_token(&store).await? {
                    Some(t) => t,
                    None => {
                        c.shutdown().await;
                        return Err(Error::invalid(
                            "ban_appeal",
                            "no appeal_token in account credentials; provide --appeal-token or run `listen` to capture the failure node",
                        ));
                    }
                }
            };
            let input = BanAppealInput {
                appeal_token: token,
                message: message.clone(),
                doc_id: *doc_id,
            };
            let result = submit_ban_appeal(&store, &lab_config, input).await?;
            c.shutdown().await;
            println!(
                "{}",
                serde_json::to_string_pretty(&result.summary)
                    .map_err(|e| Error::invalid("ban_appeal", e.to_string()))?
            );
            Ok(())
        }
        #[cfg(feature = "experimental-signup")]
        Cmd::LabSignup {
            config,
            allow_live_request,
            action,
        } => {
            let config = cli.config_path(config.as_ref(), Some("lab/requests.json"))?;
            signup::run(&cli, &config, *allow_live_request, action).await
        }
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
                            // Checkpoints holds many unrelated record kinds under
                            // one table, so a bare row count cannot answer "is
                            // there an app-state key / a login identity / any
                            // captures". Report the leading key segment only: the
                            // kind is structural, while the rest of a key is a
                            // JID or digest and stays unprinted.
                            let mut kinds: std::collections::BTreeMap<String, usize> =
                                std::collections::BTreeMap::new();
                            let mut cursor = None;
                            loop {
                                let rows =
                                    tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 512)?;
                                let done = rows.len() < 512;
                                cursor = rows.last().map(|(key, _)| key.clone());
                                for (key, _) in rows {
                                    let name = String::from_utf8_lossy(&key).into_owned();
                                    let kind = name
                                        .split_once(':')
                                        .map_or(name.as_str(), |(head, _)| head)
                                        .to_owned();
                                    *kinds.entry(kind).or_default() += 1;
                                }
                                if done {
                                    break;
                                }
                            }
                            Ok(serde_json::json!({
                                "account_id": s.account_id().as_str(),
                                "generation": s.generation().0,
                                "journal_head": tx.journal_head()?.0,
                                "registration": account.as_ref().map(|a| format!("{:?}", a.registration)).unwrap_or_else(|| "no account record".into()),
                                "tables": counts,
                                "checkpoint_kinds": kinds,
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
                    style: Default::default(),
                },
            };
            let ticket = c.send_message(req).await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&ticket)
                    .map_err(|e| Error::invalid("ticket", e.to_string()))?
            );
            eprintln!(
                "note: durably queued; an outbox-enabled comparison session (`listen --send-outbox`) transmits it"
            );
            c.shutdown().await;
            Ok(())
        }
        Cmd::React {
            chat,
            id,
            emoji,
            remove,
            ours,
            request_id,
            dry_run,
        } => {
            use whatsapp_api::model::{
                ids::{DeviceAddress, MessageId},
                message::{Direction, MessageKey, MessageRef},
            };
            if emoji.is_some() == *remove {
                return Err(Error::invalid("emoji", "pass --emoji <text> or --remove"));
            }
            let peer = Jid::parse(chat)?;
            let chat_id = ChatId::from_jid(&peer)?;
            let target = MessageRef {
                key: MessageKey {
                    chat: chat_id.clone(),
                    sender: if *ours {
                        None
                    } else {
                        Some(DeviceAddress::from_jid(&peer)?)
                    },
                    direction: if *ours {
                        Direction::Outgoing
                    } else {
                        Direction::Incoming
                    },
                    id: MessageId::new(id.clone())?,
                },
            };
            let request_id = request_id.clone().unwrap_or_else(|| {
                format!(
                    "react-{}-{}",
                    id,
                    emoji
                        .as_deref()
                        .map(|e| hex::encode(e.as_bytes()))
                        .unwrap_or_else(|| "remove".into())
                )
            });
            let c = open_client(&cli, false).await?;
            #[cfg(feature = "experimental-transport")]
            if *dry_run {
                let lid = whatsapp_api::transport::inbox::lid_for(c.store(), &peer)?;
                println!(
                    "react: dry run; key remoteJid would use the contact's {}",
                    if lid.is_some() {
                        "LID (seen on their incoming messages)"
                    } else {
                        "phone-number JID (no LID seen)"
                    }
                );
                c.shutdown().await;
                return Ok(());
            }
            #[cfg(not(feature = "experimental-transport"))]
            let _ = dry_run;
            let ticket = c
                .send_message(SendRequest {
                    client_request_id: ClientRequestId::new(request_id)?,
                    chat: chat_id,
                    content: MessageContent::Reaction {
                        target,
                        emoji: emoji.clone(),
                    },
                })
                .await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&ticket)
                    .map_err(|e| Error::invalid("ticket", e.to_string()))?
            );
            #[cfg(feature = "experimental-transport")]
            {
                let lid = whatsapp_api::transport::inbox::lid_for(c.store(), &peer)?;
                eprintln!(
                    "note: reaction key remoteJid will use the contact's {}",
                    if lid.is_some() {
                        "LID (seen on their incoming messages)"
                    } else {
                        "phone-number JID (no LID seen)"
                    }
                );
            }
            eprintln!("note: reaction durably queued; `listen --send-outbox` transmits it");
            c.shutdown().await;
            Ok(())
        }
        Cmd::Poll {
            chat,
            name,
            options,
            selectable,
            request_id,
        } => {
            use sha2::Digest;
            let peer = Jid::parse(chat)?;
            let digest = sha2::Sha256::digest(format!("{name}\n{}", options.join("\n")).as_bytes());
            let request_id = request_id
                .clone()
                .unwrap_or_else(|| format!("poll-{}", hex::encode(&digest[..8])));
            let c = open_client(&cli, false).await?;
            let ticket = c
                .send_message(SendRequest {
                    client_request_id: ClientRequestId::new(request_id)?,
                    chat: ChatId::from_jid(&peer)?,
                    content: MessageContent::Poll {
                        name: name.clone(),
                        options: options.clone(),
                        selectable_count: *selectable,
                    },
                })
                .await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&ticket)
                    .map_err(|e| Error::invalid("ticket", e.to_string()))?
            );
            eprintln!("note: poll durably queued; `listen --send-outbox` transmits it");
            c.shutdown().await;
            Ok(())
        }
        Cmd::Reply {
            chat,
            id,
            text,
            ours,
            request_id,
        } => {
            use sha2::Digest;
            use whatsapp_api::model::{
                ids::{DeviceAddress, MessageId},
                message::{Direction, MessageKey, MessageRef},
            };
            let peer = Jid::parse(chat)?;
            let chat_id = ChatId::from_jid(&peer)?;
            let digest = sha2::Sha256::digest(text.as_bytes());
            let request_id = request_id
                .clone()
                .unwrap_or_else(|| format!("reply-{id}-{}", hex::encode(&digest[..8])));
            let quoted = MessageRef {
                key: MessageKey {
                    chat: chat_id.clone(),
                    sender: if *ours {
                        None
                    } else {
                        Some(DeviceAddress::from_jid(&peer)?)
                    },
                    direction: if *ours {
                        Direction::Outgoing
                    } else {
                        Direction::Incoming
                    },
                    id: MessageId::new(id.clone())?,
                },
            };
            let c = open_client(&cli, false).await?;
            let ticket = c
                .send_message(SendRequest {
                    client_request_id: ClientRequestId::new(request_id)?,
                    chat: chat_id,
                    content: MessageContent::Text {
                        body: text.clone(),
                        quoted: Some(quoted),
                        style: Default::default(),
                    },
                })
                .await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&ticket)
                    .map_err(|e| Error::invalid("ticket", e.to_string()))?
            );
            eprintln!(
                "note: durably queued; an outbox-enabled comparison session (`listen --send-outbox`) transmits it"
            );
            c.shutdown().await;
            Ok(())
        }
        Cmd::Vote {
            chat,
            id,
            options,
            ours,
            request_id,
        } => {
            use sha2::Digest;
            use whatsapp_api::model::{
                ids::{DeviceAddress, MessageId},
                message::{Direction, MessageKey, MessageRef},
            };
            let peer = Jid::parse(chat)?;
            let chat_id = ChatId::from_jid(&peer)?;
            let digest = sha2::Sha256::digest(options.join("\n").as_bytes());
            let request_id = request_id
                .clone()
                .unwrap_or_else(|| format!("vote-{id}-{}", hex::encode(&digest[..8])));
            let target = MessageRef {
                key: MessageKey {
                    chat: chat_id.clone(),
                    sender: if *ours {
                        None
                    } else {
                        Some(DeviceAddress::from_jid(&peer)?)
                    },
                    direction: if *ours {
                        Direction::Outgoing
                    } else {
                        Direction::Incoming
                    },
                    id: MessageId::new(id.clone())?,
                },
            };
            let c = open_client(&cli, false).await?;
            let ticket = c
                .send_message(SendRequest {
                    client_request_id: ClientRequestId::new(request_id)?,
                    chat: chat_id,
                    content: MessageContent::PollVote {
                        target,
                        selected: options.clone(),
                    },
                })
                .await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&ticket)
                    .map_err(|e| Error::invalid("ticket", e.to_string()))?
            );
            eprintln!("note: vote durably queued; `listen --send-outbox` transmits it");
            c.shutdown().await;
            Ok(())
        }
        Cmd::Disappearing {
            chat,
            seconds,
            request_id,
        } => {
            use sha2::Digest;
            let peer = Jid::parse(chat)?;
            let chat_id = ChatId::from_jid(&peer)?;
            let digest = sha2::Sha256::digest(peer.to_non_ad().to_string().as_bytes());
            let request_id = request_id
                .clone()
                .unwrap_or_else(|| format!("disappearing-{}-{seconds}", hex::encode(&digest[..8])));
            let c = open_client(&cli, false).await?;
            let ticket = c
                .send_message(SendRequest {
                    client_request_id: ClientRequestId::new(request_id)?,
                    chat: chat_id,
                    content: MessageContent::DisappearingTimer { seconds: *seconds },
                })
                .await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&ticket)
                    .map_err(|e| Error::invalid("ticket", e.to_string()))?
            );
            eprintln!(
                "note: disappearing-timer change durably queued; `listen --send-outbox` transmits it"
            );
            c.shutdown().await;
            Ok(())
        }
        Cmd::Keep {
            chat,
            id,
            ours,
            undo,
            request_id,
        } => {
            use sha2::Digest;
            use whatsapp_api::model::{
                ids::{DeviceAddress, MessageId},
                message::{Direction, MessageKey, MessageRef},
            };
            let peer = Jid::parse(chat)?;
            let chat_id = ChatId::from_jid(&peer)?;
            let target = MessageRef {
                key: MessageKey {
                    chat: chat_id.clone(),
                    sender: if *ours {
                        None
                    } else {
                        Some(DeviceAddress::from_jid(&peer)?)
                    },
                    direction: if *ours {
                        Direction::Outgoing
                    } else {
                        Direction::Incoming
                    },
                    id: MessageId::new(id.clone())?,
                },
            };
            let digest = sha2::Sha256::digest(format!("{id}\n{undo}").as_bytes());
            let request_id = request_id
                .clone()
                .unwrap_or_else(|| format!("keep-{id}-{}", hex::encode(&digest[..8])));
            let c = open_client(&cli, false).await?;
            let ticket = c
                .send_message(SendRequest {
                    client_request_id: ClientRequestId::new(request_id)?,
                    chat: chat_id,
                    content: MessageContent::KeepInChat {
                        target,
                        keep: !*undo,
                    },
                })
                .await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&ticket)
                    .map_err(|e| Error::invalid("ticket", e.to_string()))?
            );
            eprintln!(
                "note: keep-in-chat change durably queued; `listen --send-outbox` transmits it"
            );
            c.shutdown().await;
            Ok(())
        }
        Cmd::ChatPrivacy {
            chat,
            on,
            off,
            request_id,
        } => {
            use sha2::Digest;
            if *on == *off {
                return Err(Error::invalid(
                    "chat-privacy",
                    "pass exactly one of --on or --off",
                ));
            }
            let peer = Jid::parse(chat)?;
            let chat_id = ChatId::from_jid(&peer)?;
            let limited = *on;
            let digest =
                sha2::Sha256::digest(format!("{}\n{limited}", peer.to_non_ad()).as_bytes());
            let request_id = request_id
                .clone()
                .unwrap_or_else(|| format!("chat-privacy-{}", hex::encode(&digest[..8])));
            let c = open_client(&cli, false).await?;
            let ticket = c
                .send_message(SendRequest {
                    client_request_id: ClientRequestId::new(request_id)?,
                    chat: chat_id,
                    content: MessageContent::ChatPrivacy { limited },
                })
                .await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&ticket)
                    .map_err(|e| Error::invalid("ticket", e.to_string()))?
            );
            eprintln!(
                "note: advanced chat privacy change durably queued; `listen --send-outbox` transmits it"
            );
            c.shutdown().await;
            Ok(())
        }
        #[cfg(feature = "experimental-transport")]
        Cmd::DisappearingDefault { seconds } => {
            let c = open_client(&cli, false).await?;
            let id =
                whatsapp_api::transport::chat_settings::queue_default_timer(c.store(), *seconds)?;
            println!("{id}");
            eprintln!("note: queued; a session transmits it; inspect with `actions`");
            c.shutdown().await;
            Ok(())
        }
        #[cfg(feature = "experimental-transport")]
        Cmd::Reproject => transport_receive::reproject(&cli),
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
        Cmd::Cancel { local_id } => {
            let c = open_client(&cli, false).await?;
            let id = local_id.parse::<LocalMessageId>()?;
            let result = c.cancel_send(id).await;
            c.shutdown().await;
            if result? {
                println!("cancelled: {id} (queued intent will not be transmitted)");
                Ok(())
            } else {
                Err(Error::invalid("local_id", "unknown local message id"))
            }
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
    let matches = <Cli as clap::CommandFactory>::command().get_matches();
    let mut cli = match <Cli as clap::FromArgMatches>::from_arg_matches(&matches) {
        Ok(cli) => cli,
        Err(e) => e.exit(),
    };
    if let Err(e) = cli.apply_profile(&matches) {
        eprintln!("error: {e}");
        return ExitCode::from(2);
    }
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

#[cfg(test)]
mod accounts_tests {
    use super::*;

    #[test]
    fn accounts_new_creates_files_and_refuses_twice() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let account_dir = temp_dir.path().join("test_account");
        let profiles_dir = temp_dir.path().join("profiles");
        std::fs::create_dir_all(&profiles_dir).unwrap();

        let cli = Cli {
            store: temp_dir.path().join("test.redb"),
            account: "test".into(),
            key_env: "TEST_KEY".into(),
            profile: None,
            profiles_dir: profiles_dir.clone(),
            json: false,
            cmd: Cmd::Accounts {
                cmd: AccountsCmd::List,
            },
            loaded_profile: None,
        };

        let material_path = temp_dir.path().join("fake.zip");
        std::fs::write(&material_path, b"fake zip").unwrap();

        let result1 = accounts_new(
            &cli,
            "test_profile",
            1,
            "5551234567",
            &material_path,
            Some(&account_dir),
            &Method::Sms,
        );
        assert!(result1.is_err());

        let result2 = accounts_new(
            &cli,
            "test_profile",
            1,
            "5551234567",
            &material_path,
            None,
            &Method::Sms,
        );
        assert!(result2.is_err());
    }

    #[test]
    fn accounts_status_shows_locked_for_unset_env() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let profiles_dir = temp_dir.path().join("profiles");
        std::fs::create_dir_all(&profiles_dir).unwrap();

        let profile_path = profiles_dir.join("test.json");
        let profile = profiles::Profile {
            store: temp_dir.path().join("test.redb"),
            account: "test".into(),
            config: None,
            key_env: "UNDEFINED_ENV_VAR_12345".into(),
        };
        std::fs::write(&profile_path, serde_json::to_string(&profile).unwrap()).unwrap();

        let cli = Cli {
            store: temp_dir.path().join("test.redb"),
            account: "test".into(),
            key_env: "TEST_KEY".into(),
            profile: None,
            profiles_dir: profiles_dir.clone(),
            json: false,
            cmd: Cmd::Accounts {
                cmd: AccountsCmd::Status { name: None },
            },
            loaded_profile: None,
        };

        let result = accounts_status(&cli, Some("test"));
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod openapi_coverage {
    use clap::CommandFactory;
    use serde_json::Value;
    use std::collections::HashSet;

    use super::Cli;

    fn collect_cli_commands(cmd: &clap::Command, prefix: Vec<String>) -> Vec<String> {
        let mut commands = Vec::new();
        let subcommands: Vec<_> = cmd.get_subcommands().collect();

        if subcommands.is_empty() {
            let mut path = vec!["whatsapp-cli".to_string()];
            path.extend(prefix);
            commands.push(path.join(" "));
        } else {
            for subcmd in subcommands {
                if subcmd.get_name() != "help" {
                    let mut new_prefix = prefix.clone();
                    new_prefix.push(subcmd.get_name().to_string());
                    commands.extend(collect_cli_commands(subcmd, new_prefix));
                }
            }
        }

        commands
    }

    #[test]
    fn every_cli_command_is_documented_in_openapi() {
        let cmd = Cli::command();
        let cli_commands = collect_cli_commands(&cmd, vec![]);
        let cli_set: HashSet<_> = cli_commands.iter().cloned().collect();

        let doc: Value = serde_json::from_str(include_str!("../../../docs/api/openapi.json"))
            .expect("docs/api/openapi.json is valid JSON");

        let mut documented = HashSet::new();
        if let Some(paths) = doc["paths"].as_object() {
            for path_item in paths.values() {
                if let Some(path_obj) = path_item.as_object() {
                    for op in path_obj.values() {
                        if let Some(x_cli) = op.get("x-cli").and_then(|v| v.as_str()) {
                            documented.insert(x_cli.to_string());
                        }
                    }
                }
            }
        }

        let mut missing: Vec<_> = cli_set
            .iter()
            .filter(|cmd| !documented.contains(*cmd))
            .collect();
        missing.sort();

        if !missing.is_empty() {
            let mut msg = String::from("undocumented CLI commands:\n");
            for cmd in missing {
                msg.push_str(&format!("  {cmd}\n"));
            }
            msg.push_str("add the operation to docs/api/openapi.json (see docs/api/README.md)");
            panic!("{}", msg);
        }
    }

    #[test]
    #[cfg(feature = "experimental-transport")]
    fn every_documented_command_exists() {
        let cmd = Cli::command();
        let cli_commands = collect_cli_commands(&cmd, vec![]);
        let cli_set: HashSet<_> = cli_commands.iter().cloned().collect();

        assert!(!cli_set.is_empty(), "leaf command count must be > 0");

        let doc: Value = serde_json::from_str(include_str!("../../../docs/api/openapi.json"))
            .expect("docs/api/openapi.json is valid JSON");

        let mut documented = HashSet::new();
        if let Some(paths) = doc["paths"].as_object() {
            for path_item in paths.values() {
                if let Some(path_obj) = path_item.as_object() {
                    for op in path_obj.values() {
                        if let Some(x_cli) = op.get("x-cli").and_then(|v| v.as_str()) {
                            documented.insert(x_cli.to_string());
                        }
                    }
                }
            }
        }

        let mut undeclared: Vec<_> = documented
            .iter()
            .filter(|cmd| !cli_set.contains(*cmd))
            .collect();
        undeclared.sort();

        if !undeclared.is_empty() {
            let mut msg = String::from("documented commands that do not exist:\n");
            for cmd in undeclared {
                msg.push_str(&format!("  {cmd}\n"));
            }
            panic!("{}", msg);
        }
    }
}
