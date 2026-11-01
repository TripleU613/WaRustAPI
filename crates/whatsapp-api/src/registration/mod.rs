//! Primary registration as a durable, branching state machine (docs/design/README.md section 7).
//!
//! The Android registration protocol remains incomplete for the active
//! profile. A version-scoped request-construction layer for a newer signed
//! comparison build lives in [`wire`], but complete verification steps and
//! live interoperability are still missing. The state machine, persistence, challenge
//! binding and typed steps are implemented and tested. The default
//! profile-specific [`RegistrationFlow`] reports
//! [`BlockedReason::ProfileIncomplete`]. A comparison-scoped injectable HTTP
//! flow exists for controlled lab use, but it is never selected implicitly;
//! the default client sends no request and requests no OTP.

#[cfg(feature = "experimental-signal-crypto")]
mod envelope;
#[cfg(feature = "experimental-signal-crypto")]
pub mod factory;
pub mod prerequisites;
pub mod request;
mod request_config;
pub mod token;
#[cfg(feature = "token-material")]
pub mod token_material;
pub mod wire;

use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

pub mod consent;

use crate::error::{Error, Result};
use crate::model::ids::AccountGeneration;
use crate::model::jid::Jid;
use crate::model::phone::PhoneNumber;
use crate::model::time::Timestamp;
use crate::profile::ProtocolProfile;
use crate::session::{AccountRecord, ObservedCredential, RegistrationStatus};
use crate::store::{ProtocolStore, Table, WriteTx};

const RECORD_V1: u16 = 1;
const CURRENT_KEY: &[u8] = b"current";
const MAX_HISTORY: usize = 64;

/// Opaque identifier of one registration attempt (serialized as hex).
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AttemptId(pub [u8; 16]);

crate::model::ids::hex16_serde!(AttemptId);

impl AttemptId {
    fn generate() -> Result<Self> {
        let mut b = [0u8; 16];
        getrandom::fill(&mut b)
            .map_err(|e| Error::Io(std::io::Error::other(format!("entropy source failed: {e}"))))?;
        Ok(Self(b))
    }
}

impl fmt::Debug for AttemptId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AttemptId({})", hex::encode(&self.0[..4]))
    }
}

/// How a verification challenge is delivered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ChallengeMethod {
    /// One-time code by SMS.
    Sms,
    /// One-time code by voice call.
    Voice,
    /// Another server-directed method, described but not implemented.
    Other(String),
}

/// A challenge handle bound to an attempt, generation and step. It carries
/// no secret. An old handle cannot answer a newer step or attempt.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChallengeHandle {
    attempt: AttemptId,
    generation: AccountGeneration,
    step: u32,
    method: ChallengeMethod,
    /// When the challenge expires, if the server said.
    pub expires_at: Option<Timestamp>,
    /// Earliest retry time, if the server said.
    pub retry_after: Option<Timestamp>,
    /// Earliest time another code may be submitted on this step, after a
    /// `mismatch` (2.26.37.71 `VerifyPhoneNumber.A1v` -> `GKR.A0s` stores
    /// `code_verification_retry_time`). Distinct from `retry_after`, which
    /// gates requesting a new code. Absent in attempts saved before it existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answer_not_before: Option<Timestamp>,
}

impl fmt::Debug for ChallengeHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChallengeHandle")
            .field("attempt", &self.attempt)
            .field("generation", &self.generation)
            .field("step", &self.step)
            .field("method", &self.method)
            .finish_non_exhaustive()
    }
}

impl ChallengeHandle {
    /// The method of this challenge.
    pub fn method(&self) -> &ChallengeMethod {
        &self.method
    }
}

/// Why registration is blocked for this profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum BlockedReason {
    /// Consent requires an explicit, supported user-interaction flow. Keeping
    /// context does not accept it or authorize following its URL.
    ConsentRequired {
        /// Server-provided branch fields; Debug redacts the continuation URL.
        details: wire::RegistrationConsent,
        /// Earliest generic retry time, if supplied.
        retry_after: Option<Timestamp>,
        /// SMS delivery deadline, if supplied.
        sms_retry_after: Option<Timestamp>,
        /// Voice delivery deadline, if supplied.
        voice_retry_after: Option<Timestamp>,
    },
    /// The protocol profile lacks required facts (ledger references listed).
    ProfileIncomplete {
        /// Missing ledger entries.
        missing: Vec<String>,
    },
    /// The server refused the flow with a sanitized code.
    ServerRefused {
        /// Sanitized code.
        code: Option<u32>,
        /// Actual HTTP status, including a successful HTTP response carrying a refusal.
        /// Absent in records written before refusal diagnostics were persisted.
        #[serde(default)]
        http_status: Option<u16>,
        /// Recognized protocol reason only; arbitrary server text is never persisted.
        #[serde(default)]
        reason: Option<wire::RegistrationResponseReason>,
        /// Earliest time another request of the refused kind may be sent: the
        /// longest `sms_wait`/`voice_wait`/`email_otp_wait` the refusal carried
        /// (a refusal with `retry_after` is a cooldown instead). Enforced by
        /// the one-shot actions and the email-challenge resume (registration
        /// safety audit, 2026-10-13). Absent in older records and when the
        /// refusal carried no wait.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        not_before: Option<Timestamp>,
    },
    /// The server requires a step this implementation does not support.
    UnsupportedStep {
        /// Description of the step.
        description: String,
    },
}

/// A caller action required to continue.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RequiredAction {
    /// Enter the one-time code delivered by the challenge method.
    EnterCode(ChallengeHandle),
    /// Enter the account PIN (two-step verification).
    EnterPin(ChallengeHandle),
    /// Ask the server to email the one-time code (`/v2/code` with
    /// `method=email_otp`); the server named email verification as the second factor.
    RequestEmailCode(ChallengeHandle),
    /// Enter the one-time code delivered to the account's email.
    EnterEmailCode(ChallengeHandle),
    /// Wait until the given time before retrying.
    WaitForCooldown {
        /// Earliest retry time.
        until: Timestamp,
    },
}

/// Result of `begin` / `submit`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RegistrationStep {
    /// Registration completed and committed.
    Completed,
    /// A caller action is required.
    Action(RequiredAction),
    /// This profile cannot proceed.
    Blocked(BlockedReason),
    /// The outcome of a submitted answer was lost; reconciliation is needed
    /// before any repeated mutation.
    OutcomeUnknown,
}

/// Account material returned by a successful profile-specific registration
/// flow and committed atomically with the `Registered` state.
#[derive(Default)]
pub struct RegistrationCompletion {
    own_jid: Option<Jid>,
    credentials: Vec<ObservedCredential>,
}

impl fmt::Debug for RegistrationCompletion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegistrationCompletion")
            .field("own_jid", &self.own_jid)
            .field("credential_count", &self.credentials.len())
            .finish()
    }
}

impl RegistrationCompletion {
    /// Create an empty completion update.
    pub const fn new() -> Self {
        Self {
            own_jid: None,
            credentials: Vec::new(),
        }
    }

    /// Attach the server-assigned account JID.
    pub fn with_own_jid(mut self, own_jid: Jid) -> Self {
        self.own_jid = Some(own_jid);
        self
    }

    /// Attach one observed credential. Its value remains encrypted at rest
    /// and redacted from debug output by [`ObservedCredential`].
    pub fn with_credential(mut self, credential: ObservedCredential) -> Result<Self> {
        if self.credentials.len() >= 64 {
            return Err(Error::ResourceExhausted {
                resource: "registration completion credentials",
                limit: 64,
            });
        }
        if credential.name.is_empty() || credential.name.len() > 128 {
            return Err(Error::invalid(
                "registration_credential_name",
                "must contain 1..=128 UTF-8 bytes",
            ));
        }
        if credential.value.len() > wire::MAX_RESPONSE_SECRET_BYTES {
            return Err(Error::ResourceExhausted {
                resource: "registration completion credential bytes",
                limit: wire::MAX_RESPONSE_SECRET_BYTES as u64,
            });
        }
        self.credentials.push(credential);
        Ok(self)
    }
}

/// Result returned by a profile-specific registration driver.
pub struct RegistrationFlowOutput {
    state: RegistrationState,
    completion: Option<RegistrationCompletion>,
}

impl fmt::Debug for RegistrationFlowOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegistrationFlowOutput")
            .field("state", &self.state)
            .field("completion", &self.completion)
            .finish()
    }
}

impl RegistrationFlowOutput {
    /// Return a state transition with no account-record update.
    pub const fn state(state: RegistrationState) -> Self {
        Self {
            state,
            completion: None,
        }
    }

    /// Complete registration and atomically apply account material.
    pub const fn registered(completion: RegistrationCompletion) -> Self {
        Self {
            state: RegistrationState::Registered,
            completion: Some(completion),
        }
    }
}

impl From<RegistrationState> for RegistrationFlowOutput {
    fn from(state: RegistrationState) -> Self {
        Self::state(state)
    }
}

/// A typed answer to a challenge. Zeroized on drop; never logged or stored.
pub enum Answer {
    /// One-time code.
    Code(Zeroizing<String>),
    /// Account PIN.
    Pin(Zeroizing<String>),
    /// One-time code delivered to the account's email (`/v2/code` `method=email_otp`).
    EmailCode(Zeroizing<String>),
}

impl fmt::Debug for Answer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Code(_) => f.write_str("Answer::Code(<redacted>)"),
            Self::Pin(_) => f.write_str("Answer::Pin(<redacted>)"),
            Self::EmailCode(_) => f.write_str("Answer::EmailCode(<redacted>)"),
        }
    }
}

/// Durable registration state (docs/design/README.md section 7 table).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum RegistrationState {
    /// No established account credentials.
    Unregistered,
    /// Prerequisites are being created and persisted.
    Preparing,
    /// Verification requested.
    ChallengePending {
        /// Bound handle.
        handle: ChallengeHandle,
    },
    /// A supported additional step is required.
    NeedsUserAction {
        /// Bound handle.
        handle: ChallengeHandle,
    },
    /// `/v2/register` answered `challenge`: the account's email second factor
    /// must be satisfied before registration completes. Next: request the
    /// email code.
    EmailChallengeRequired {
        /// Bound handle.
        handle: ChallengeHandle,
    },
    /// The email one-time code was requested; awaiting its entry.
    EmailCodePending {
        /// Bound handle.
        handle: ChallengeHandle,
    },
    /// A submitted answer is in progress.
    Verifying {
        /// Step being verified.
        step: u32,
    },
    /// A request outcome was lost.
    RegistrationOutcomeUnknown {
        /// Step whose outcome is unknown.
        step: u32,
    },
    /// Server-issued state committed.
    Registered,
    /// Observing the server's retry timing.
    CoolingDown {
        /// Earliest retry time.
        until: Timestamp,
    },
    /// The flow cannot proceed.
    Blocked {
        /// Why.
        reason: BlockedReason,
    },
}

impl RegistrationState {
    /// Stable name for diagnostics.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Unregistered => "unregistered",
            Self::Preparing => "preparing",
            Self::ChallengePending { .. } => "challenge_pending",
            Self::NeedsUserAction { .. } => "needs_user_action",
            Self::EmailChallengeRequired { .. } => "email_challenge_required",
            Self::EmailCodePending { .. } => "email_code_pending",
            Self::Verifying { .. } => "verifying",
            Self::RegistrationOutcomeUnknown { .. } => "outcome_unknown",
            Self::Registered => "registered",
            Self::CoolingDown { .. } => "cooling_down",
            Self::Blocked { .. } => "blocked",
        }
    }

    /// Whether a transition to `to` is allowed. Registration is never
    /// restarted implicitly: `Registered` and `Blocked` only leave through an
    /// explicit reset to `Unregistered`.
    pub fn can_transition(&self, to: &RegistrationState) -> bool {
        use RegistrationState as S;
        if matches!(
            self,
            S::Blocked {
                reason: BlockedReason::ConsentRequired { .. }
                    | BlockedReason::ServerRefused {
                        reason: Some(wire::RegistrationResponseReason::Consent),
                        ..
                    }
            }
        ) && matches!(to, S::Verifying { .. })
        {
            return true;
        }
        matches!(
            (self, to),
            (S::Unregistered, S::Preparing)
                | (
                    S::Preparing,
                    S::ChallengePending { .. }
                        | S::RegistrationOutcomeUnknown { .. }
                        | S::Blocked { .. }
                        | S::CoolingDown { .. }
                )
                | (
                    S::ChallengePending { .. },
                    // `Preparing` is the resend path: ask the server for a new
                    // code on the *same* attempt, keeping its identity, keys and
                    // history, exactly as the official client's "resend" does.
                    // Only `resend_code` takes it, and only once the handle's
                    // `retry_after` has elapsed; `begin` still short-circuits on
                    // a pending challenge rather than re-requesting implicitly.
                    S::Verifying { .. } | S::CoolingDown { .. } | S::Blocked { .. } | S::Preparing
                )
                | (
                    S::NeedsUserAction { .. },
                    S::Verifying { .. } | S::Blocked { .. }
                )
                | (
                    S::EmailChallengeRequired { .. } | S::EmailCodePending { .. },
                    S::Verifying { .. } | S::CoolingDown { .. } | S::Blocked { .. }
                )
                | (
                    S::Blocked {
                        reason: BlockedReason::ServerRefused {
                            reason: Some(wire::RegistrationResponseReason::Challenge),
                            ..
                        }
                    },
                    S::EmailChallengeRequired { .. }
                )
                // An attempt saved as blocked by an expired code before
                // `stale` returned to a pending challenge: only `resend_code`
                // takes this, exactly as it would from that pending challenge.
                | (
                    S::Blocked {
                        reason: BlockedReason::ServerRefused {
                            reason:
                                Some(
                                    wire::RegistrationResponseReason::Stale
                                    | wire::RegistrationResponseReason::Incorrect
                                    | wire::RegistrationResponseReason::Mismatch,
                                ),
                            ..
                        }
                    },
                    S::Preparing
                )
                | (
                    S::Verifying { .. },
                    S::Registered
                        | S::NeedsUserAction { .. }
                        | S::EmailChallengeRequired { .. }
                        | S::EmailCodePending { .. }
                        | S::ChallengePending { .. }
                        | S::RegistrationOutcomeUnknown { .. }
                        | S::CoolingDown { .. }
                        | S::Blocked { .. },
                )
                | (
                    S::RegistrationOutcomeUnknown { .. },
                    S::Registered
                        | S::ChallengePending { .. }
                        | S::EmailChallengeRequired { .. }
                        | S::Blocked { .. }
                )
                | (
                    S::CoolingDown { .. },
                    S::ChallengePending { .. } | S::Preparing | S::Blocked { .. }
                )
                | (S::Registered | S::Blocked { .. }, S::Unregistered)
        )
    }

    /// Whether the state is terminal for an attempt.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Registered | Self::Blocked { .. })
    }

    /// The wait a saved server refusal carried, if any (`ServerRefused`'s
    /// `not_before`).
    pub fn refusal_not_before(&self) -> Option<Timestamp> {
        match self {
            Self::Blocked {
                reason: BlockedReason::ServerRefused { not_before, .. },
            } => *not_before,
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Transition {
    at: Timestamp,
    from: String,
    to: String,
}

/// A request that could not be built was not sent (registration safety audit,
/// 2026-10-13): for example a config edited since the attempt froze it, or a
/// resend method the frozen config does not use. Keeps the cause.
fn not_built(error: Error) -> Error {
    match error {
        Error::RequestNotSent(_) => error,
        other => Error::RequestNotSent(Box::new(other)),
    }
}

/// A client-error status that refuses the request outright: 4xx except 408
/// (request timeout) and 429 (rate limited), whose waits we would lose.
fn definite_refusal_status(status: u16) -> bool {
    (400..500).contains(&status) && status != 408 && status != 429
}

/// What a not-sent restore undoes besides the attempt's state: the one-shot
/// latch its reservation took, and the request material its `prepare` changed
/// (registration safety audit review M2, 2026-10-13). A latch is released only
/// together with the material it guards, or where nothing was prepared.
#[derive(Default)]
struct Undo {
    latch: Option<&'static [u8]>,
    material: Option<Zeroizing<Vec<u8>>>,
}

const STORE_OWNER_KEY: &[u8] = b"store-owner";

/// The number this store was created for (`accounts new`, or an explicit
/// `lab-signup begin --create-store`), if recorded. Stores created before this
/// record existed have none; their saved attempt names their number instead.
pub fn store_owner(store: &ProtocolStore) -> Result<Option<PhoneNumber>> {
    store.read(|tx| tx.get_json(Table::RegistrationAttempts, STORE_OWNER_KEY, RECORD_V1))
}

/// Record that this store belongs to `phone`. Idempotent for the same number;
/// refuses to re-assign a store to another number, so one store never holds
/// two numbers' identities (registration safety audit, 2026-10-13).
pub fn claim_store(store: &ProtocolStore, phone: &PhoneNumber) -> Result<()> {
    store.write(|tx| {
        match tx.get_json::<PhoneNumber>(Table::RegistrationAttempts, STORE_OWNER_KEY, RECORD_V1)? {
            Some(owner) if &owner == phone => Ok(()),
            Some(_) => Err(Error::invalid(
                "registration_store",
                "this store was created for another number",
            )),
            // An older store may hold another number's account material
            // without an owner record (its attempt reset); claiming it would
            // record the wrong owner for good (registration review L6).
            None if AccountRecord::load_in_write(tx)?
                .and_then(|account| account.phone)
                .is_some_and(|holder| &holder != phone) =>
            {
                Err(Error::invalid(
                    "registration_store",
                    "this store holds another number's account material",
                ))
            }
            None => tx.put_json(
                Table::RegistrationAttempts,
                STORE_OWNER_KEY,
                RECORD_V1,
                phone,
            ),
        }
    })
}

/// The saved attempt's state name (`preparing`, `challenge_pending`,
/// `cooling_down`, `outcome_unknown`, ...), or `None` when this store holds no
/// attempt. Read-only, for status displays such as `accounts status`.
pub fn current_state_name(store: &ProtocolStore) -> Result<Option<&'static str>> {
    Ok(current_attempt(store)?.map(|attempt| attempt.state.name()))
}

/// The saved attempt, whichever profile created it, or `None`. Read-only and
/// needs no flow, for inspections such as the `lab-signup status` store route.
/// Its `Debug` redacts the phone; serialised JSON does not, so a printer must
/// redact it (and the consent secrets) itself.
pub fn current_attempt(store: &ProtocolStore) -> Result<Option<RegistrationAttempt>> {
    store.read(|tx| tx.get_json(Table::RegistrationAttempts, CURRENT_KEY, RECORD_V1))
}

/// Marks an email-code request whose answer arrived but could not be read.
fn email_answered_key(attempt: AttemptId, step: u32) -> String {
    format!("email-request-answered:{}:{step}", hex::encode(attempt.0))
}

/// The longest of the server-directed waits that are present.
fn longest<const N: usize>(waits: [Option<std::time::Duration>; N]) -> Option<std::time::Duration> {
    waits.into_iter().flatten().max()
}

/// Durable record of one attempt. `Debug` redacts the phone number.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistrationAttempt {
    /// Attempt identity.
    pub id: AttemptId,
    /// Account generation the attempt belongs to.
    pub generation: AccountGeneration,
    /// Profile the attempt uses.
    pub profile_id: String,
    phone: PhoneNumber,
    /// Requested challenge method.
    pub method: ChallengeMethod,
    /// Current state.
    pub state: RegistrationState,
    /// Creation time.
    pub created_at: Timestamp,
    /// Last update time.
    pub updated_at: Timestamp,
    history: Vec<Transition>,
}

impl fmt::Debug for RegistrationAttempt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegistrationAttempt")
            .field("id", &self.id)
            .field("generation", &self.generation)
            .field("profile_id", &self.profile_id)
            .field("phone", &self.phone)
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}

impl RegistrationAttempt {
    /// The phone number being registered.
    pub fn phone(&self) -> &PhoneNumber {
        &self.phone
    }

    /// Apply a validated transition.
    pub fn transition(&mut self, to: RegistrationState, now: Timestamp) -> Result<()> {
        if !self.state.can_transition(&to) {
            return Err(Error::invalid(
                "registration_state",
                format!("illegal transition {} -> {}", self.state.name(), to.name()),
            ));
        }
        if self.history.len() >= MAX_HISTORY {
            self.history.remove(0);
        }
        self.history.push(Transition {
            at: now,
            from: self.state.name().to_owned(),
            to: to.name().to_owned(),
        });
        self.state = to;
        self.updated_at = now;
        Ok(())
    }

    /// Whether the current state was reached by a code request's refusal (the
    /// preflight or `/v2/code`): the last transition is `preparing -> blocked`.
    /// OTP submission, security, consent and email-code refusals are reached
    /// from `verifying`. The code-request diagnostics and the corrected-token
    /// resend send a new code request, so only this refusal makes them eligible.
    pub fn refused_at_code_request(&self) -> bool {
        self.history
            .last()
            .is_some_and(|entry| entry.from == "preparing" && entry.to == "blocked")
    }

    /// Number of recorded transitions.
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Code-request preparations retained in complete local history. This is
    /// not a delivery count; a crash may occur after reservation and before I/O.
    /// Returns None if bounded history could have discarded earlier entries.
    pub fn code_preparation_count(&self) -> Option<usize> {
        (self.history.len() < MAX_HISTORY).then(|| {
            self.history
                .iter()
                .filter(|entry| entry.to == "preparing")
                .count()
        })
    }
}

/// Future returned by a profile-specific registration driver.
pub type RegistrationFuture<'a> =
    Pin<Box<dyn Future<Output = Result<RegistrationFlowOutput>> + Send + 'a>>;

/// Profile-specific registration driver. Implementations may perform network
/// requests; they must never persist or log answers. The service invokes these
/// futures only after its durable pre-request transition has committed and the
/// store transaction has been released.
pub trait RegistrationFlow: Send + Sync + fmt::Debug {
    /// Inspect consent context or submit an explicitly supplied decision.
    fn consent<'a>(
        &'a self,
        _attempt: &'a RegistrationAttempt,
        _decision: Option<&'a consent::ConsentDecision>,
    ) -> RegistrationFuture<'a> {
        // Nothing is sent by a flow without this step: not an unknown outcome.
        Box::pin(async {
            Err(Error::RequestNotSent(Box::new(Error::invalid(
                "consent",
                "flow does not implement consent",
            ))))
        })
    }
    /// Generate and persist profile prerequisites inside the same transaction
    /// as the `Preparing` transition. This method must be bounded and perform
    /// no network I/O. If it fails, neither the attempt nor its prerequisites
    /// are committed.
    fn prepare(&self, tx: &mut WriteTx<'_>, attempt: &RegistrationAttempt) -> Result<()>;

    /// The saved request material a correcting `prepare` may change, read in
    /// the reservation transaction just before it runs. Held in memory only,
    /// so a request that provably never left can be undone exactly
    /// ([`Self::restore_prepared`]). `None`: this flow cannot undo `prepare`.
    fn prepared_snapshot(&self, _tx: &WriteTx<'_>) -> Result<Option<Zeroizing<Vec<u8>>>> {
        Ok(None)
    }

    /// Put back a [`Self::prepared_snapshot`] in the restoring transaction.
    fn restore_prepared(&self, _tx: &mut WriteTx<'_>, _snapshot: Zeroizing<Vec<u8>>) -> Result<()> {
        Err(Error::invalid(
            "registration_material",
            "flow cannot restore prepared material",
        ))
    }

    /// Request the first challenge. Returns the next state (typically
    /// `ChallengePending`, `CoolingDown` or `Blocked`) and any atomic account
    /// update.
    fn begin<'a>(&'a self, attempt: &'a RegistrationAttempt) -> RegistrationFuture<'a>;

    /// Submit an answer for the bound handle. Returns the next state and any
    /// account material that must commit with it.
    fn submit<'a>(
        &'a self,
        attempt: &'a RegistrationAttempt,
        handle: &'a ChallengeHandle,
        answer: &'a Answer,
    ) -> RegistrationFuture<'a>;

    /// Ask the server to email the second-factor code (`/v2/code` with
    /// `method=email_otp`). Returns the next state; no address is sent.
    fn request_email_code<'a>(
        &'a self,
        _attempt: &'a RegistrationAttempt,
        _handle: &'a ChallengeHandle,
        _email: Option<&'a str>,
    ) -> RegistrationFuture<'a> {
        // Nothing is sent by a flow without this step: not an unknown outcome.
        Box::pin(async {
            Err(Error::RequestNotSent(Box::new(Error::invalid(
                "challenge",
                "flow does not implement the email challenge",
            ))))
        })
    }

    /// Reconcile a request whose response was lost. Implementations must use
    /// an observed read-safe flow and must not blindly repeat the mutation.
    fn reconcile<'a>(&'a self, _attempt: &'a RegistrationAttempt) -> RegistrationFuture<'a> {
        Box::pin(async {
            Err(Error::invalid(
                "registration_reconciliation",
                "this registration flow has no observed reconciliation procedure",
            ))
        })
    }
}

/// The default flow while the active Android registration protocol is
/// `Unknown`: it reports the missing evidence and never sends anything.
#[derive(Debug, Clone)]
pub struct UnsupportedFlow {
    missing: Vec<String>,
}

impl UnsupportedFlow {
    /// Build from a profile's missing registration facts.
    pub fn for_profile(profile: &ProtocolProfile) -> Self {
        Self {
            missing: profile
                .missing_for_registration()
                .into_iter()
                .map(str::to_owned)
                .collect(),
        }
    }
}

impl RegistrationFlow for UnsupportedFlow {
    fn prepare(&self, _tx: &mut WriteTx<'_>, _attempt: &RegistrationAttempt) -> Result<()> {
        Ok(())
    }

    /// A fresh attempt (only its `unregistered -> preparing` entry) is
    /// blocked locally. Resuming an existing attempt, which some other flow
    /// under this profile id created (a cooldown, say), sends nothing and is
    /// restored as it was, rather than having a local block written over it
    /// (registration review, 2026-10-13).
    fn begin<'a>(&'a self, attempt: &'a RegistrationAttempt) -> RegistrationFuture<'a> {
        Box::pin(async move {
            if attempt.history_len() > 1 {
                return Err(self.not_sent());
            }
            Ok(RegistrationState::Blocked {
                reason: BlockedReason::ProfileIncomplete {
                    missing: self.missing.clone(),
                },
            }
            .into())
        })
    }

    /// This flow never produces a challenge, so a pending one belongs to
    /// another flow under the same profile id; nothing is sent and the
    /// challenge is restored, not overwritten by a local block that `reset`
    /// would then accept (registration review, 2026-10-13).
    fn submit<'a>(
        &'a self,
        _attempt: &'a RegistrationAttempt,
        _handle: &'a ChallengeHandle,
        _answer: &'a Answer,
    ) -> RegistrationFuture<'a> {
        Box::pin(async move { Err(self.not_sent()) })
    }
}

impl UnsupportedFlow {
    fn not_sent(&self) -> Error {
        Error::RequestNotSent(Box::new(Error::invalid(
            "registration",
            format!(
                "this registration flow sends nothing; the profile is missing {:?}",
                self.missing
            ),
        )))
    }
}

/// Supplies exact comparison-build requests and persists their prerequisites.
///
/// Implementations normally use [`wire::assemble_code_request`] and
/// [`wire::assemble_complete_request`]. Keeping this boundary separate lets
/// callers source device identifiers and key material from their durable
/// store without exposing them through the high-level service.
pub trait ComparisonRequestFactory: Send + Sync + fmt::Debug {
    /// Same-identity consent request; None inspects existence metadata only.
    fn consent_request(
        &self,
        _attempt: &RegistrationAttempt,
        _decision: Option<&consent::ConsentDecision>,
    ) -> Result<wire::EncodedRegistrationRequest> {
        Err(Error::invalid(
            "consent",
            "factory does not implement consent",
        ))
    }
    /// Optional same-identity `/v2/exist` request before selecting SMS/voice.
    fn preflight_request(
        &self,
        _attempt: &RegistrationAttempt,
    ) -> Result<Option<wire::EncodedRegistrationRequest>> {
        Ok(None)
    }
    /// The same-identity `/v2/exist` request regardless of the preflight opt-in
    /// (a read-safe probe of method eligibility).
    fn exist_request(
        &self,
        _attempt: &RegistrationAttempt,
    ) -> Result<wire::EncodedRegistrationRequest> {
        Err(Error::invalid(
            "exist",
            "factory does not implement the existence probe",
        ))
    }
    /// `/v2/code` with `method=email_otp`: asks the server to email the
    /// one-time code to the address on file (2.26.37.71 `LX/0GB.A01`).
    fn email_code_request(
        &self,
        _attempt: &RegistrationAttempt,
    ) -> Result<wire::EncodedRegistrationRequest> {
        Err(Error::invalid(
            "email_otp",
            "factory does not implement the email one-time code request",
        ))
    }
    /// Persist keys and identifiers needed by later request construction.
    fn prepare(&self, tx: &mut WriteTx<'_>, attempt: &RegistrationAttempt) -> Result<()>;

    /// See [`RegistrationFlow::prepared_snapshot`].
    fn prepared_snapshot(&self, _tx: &WriteTx<'_>) -> Result<Option<Zeroizing<Vec<u8>>>> {
        Ok(None)
    }

    /// See [`RegistrationFlow::restore_prepared`].
    fn restore_prepared(&self, _tx: &mut WriteTx<'_>, _snapshot: Zeroizing<Vec<u8>>) -> Result<()> {
        Err(Error::invalid(
            "registration_material",
            "factory cannot restore prepared material",
        ))
    }

    /// Build a `/v2/challenge` request (email second factor).
    fn challenge_request(
        &self,
        _attempt: &RegistrationAttempt,
        _params: &wire::RegistrationChallengeParams,
    ) -> Result<wire::EncodedRegistrationRequest> {
        Err(Error::invalid(
            "challenge",
            "factory does not implement the email challenge",
        ))
    }

    /// Build the first `/v2/code` mutation.
    fn code_request(
        &self,
        attempt: &RegistrationAttempt,
    ) -> Result<wire::EncodedRegistrationRequest>;

    /// Build the `/v2/register` mutation for an entered one-time code.
    fn complete_request(
        &self,
        attempt: &RegistrationAttempt,
        code: &Zeroizing<String>,
    ) -> Result<wire::EncodedRegistrationRequest>;

    /// Build the `/v2/security` mutation for an account PIN.
    fn security_request(
        &self,
        attempt: &RegistrationAttempt,
        pin: &Zeroizing<String>,
    ) -> Result<wire::EncodedRegistrationRequest>;
}

/// Conservative happy-path driver for the signed 2.26.37.70 comparison
/// request format.
///
/// Only an HTTP-success `sent` response advances `/v2/code` to an OTP
/// challenge, and only an HTTP-success `ok` response completes
/// `/v2/register`. Other branches become explicit blocked/cooldown states;
/// the driver does not guess additional mutations.
pub struct ComparisonHttpFlow {
    requests: Arc<dyn ComparisonRequestFactory>,
    transport: Arc<dyn wire::RegistrationTransport>,
}

impl fmt::Debug for ComparisonHttpFlow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ComparisonHttpFlow")
            .field("requests", &self.requests)
            .field("transport", &self.transport)
            .finish()
    }
}

impl ComparisonHttpFlow {
    /// Create a comparison-scoped flow. Constructing it does not perform I/O.
    pub fn new(
        requests: Arc<dyn ComparisonRequestFactory>,
        transport: Arc<dyn wire::RegistrationTransport>,
    ) -> Self {
        Self {
            requests,
            transport,
        }
    }

    async fn send(
        &self,
        request: wire::EncodedRegistrationRequest,
    ) -> Result<(u16, wire::RegistrationResponse)> {
        let response = self.transport.send(request).await?;
        let status_code = response.status_code();
        let parsed = response.parse_json().map_err(|error| match error {
            // An unreadable body under a definite client-error status is a
            // refusal we cannot read (the email `/v2/challenge` guess got HTTP
            // 404 with a non-JSON body). Under 2xx the request may have
            // succeeded; a 5xx may come from a gateway after the backend acted;
            // 408 and 429 carry a wait we cannot read. Those keep the generic,
            // uncertain classification (reviews M1 and its follow-up).
            Error::InvalidInput {
                field: "registration_response",
                ..
            } if definite_refusal_status(status_code) => Error::invalid(
                "registration_refusal_body",
                format!("HTTP {status_code} with a body that is not a registration answer"),
            ),
            other => other,
        })?;
        Ok((status_code, parsed))
    }

    /// Read-safe probe: the same-identity `/v2/exist` request the client sends
    /// before choosing a delivery method. Changes no state; the caller reports
    /// eligibility (`email_otp_eligible`, `wa_old_eligible`, masked email).
    pub async fn exist_probe(
        &self,
        attempt: &RegistrationAttempt,
    ) -> Result<(u16, wire::RegistrationResponse)> {
        let request = self.requests.exist_request(attempt)?;
        self.send(request).await
    }

    fn blocked_response(
        status_code: u16,
        response: &wire::RegistrationResponse,
    ) -> RegistrationFlowOutput {
        if (200..300).contains(&status_code)
            && response.status == wire::RegistrationResponseStatus::Fail
            && response.reason == Some(wire::RegistrationResponseReason::Consent)
        {
            let now = Timestamp::now();
            return RegistrationState::Blocked {
                reason: BlockedReason::ConsentRequired {
                    details: response.consent.clone(),
                    retry_after: response.retry_after.map(|wait| now.saturating_add(wait)),
                    sms_retry_after: response.sms_wait.map(|wait| now.saturating_add(wait)),
                    voice_retry_after: response.voice_wait.map(|wait| now.saturating_add(wait)),
                },
            }
            .into();
        }
        if let Some(wait) = response.retry_after {
            // A refusal that sets a cooldown also keeps the longer of any SMS,
            // voice or email wait it carries (as the blocked branch below
            // does): the next request uses one of them; it only delays.
            let wait = longest([
                Some(wait),
                response.sms_wait,
                response.voice_wait,
                response.email_otp_wait,
            ])
            .unwrap_or(wait);
            return RegistrationState::CoolingDown {
                until: Timestamp::now().saturating_add(wait),
            }
            .into();
        }
        if response.status == wire::RegistrationResponseStatus::Fail || status_code >= 400 {
            // Without `retry_after` the refusal stays blocked, but any method
            // wait it carries still binds the next request (longest wins).
            let not_before = longest([
                response.sms_wait,
                response.voice_wait,
                response.email_otp_wait,
            ])
            .map(|wait| Timestamp::now().saturating_add(wait));
            return RegistrationState::Blocked {
                reason: BlockedReason::ServerRefused {
                    code: (status_code >= 400).then_some(u32::from(status_code)),
                    http_status: Some(status_code),
                    reason: response.reason,
                    not_before,
                },
            }
            .into();
        }
        RegistrationState::Blocked {
            reason: BlockedReason::UnsupportedStep {
                description: format!(
                    "comparison registration response requires an unimplemented branch: status={:?}, reason={:?}",
                    response.status, response.reason
                ),
            },
        }
        .into()
    }

    fn completion_from(
        response: &mut wire::RegistrationResponse,
        source_step: &str,
    ) -> Result<RegistrationCompletion> {
        let mut completion = RegistrationCompletion::new();
        let mut secrets = response.take_secrets();
        for (name, value) in [
            ("login", secrets.take_login()),
            ("wipe_token", secrets.take_wipe_token()),
            ("passkey_credential", secrets.take_passkey_credential()),
            ("appeal_token", secrets.take_appeal_token()),
            ("edge_routing_info", secrets.take_edge_routing_info()),
        ] {
            if let Some(value) = value {
                completion = completion.with_credential(ObservedCredential {
                    name: name.to_owned(),
                    value: Zeroizing::new(value.as_bytes().to_vec()),
                    observed_at: Timestamp::now(),
                    source_step: Some(source_step.into()),
                })?;
            }
        }
        Ok(completion)
    }
}

// Conservative lab policy, not a complete reconstruction of APK UI decisions.
fn preflight_decision(
    status: u16,
    response: &wire::RegistrationResponse,
    method: &ChallengeMethod,
) -> Option<RegistrationFlowOutput> {
    use wire::{
        RegistrationResponseReason as Reason, RegistrationResponseStatus as Status,
        VerificationMethod as Method,
    };
    let (selected, method_wait) = match method {
        ChallengeMethod::Sms => (Method::Sms, response.sms_wait),
        ChallengeMethod::Voice => (Method::Voice, response.voice_wait),
        ChallengeMethod::Other(_) => (Method::Other, None),
    };
    if let Some(wait) = response
        .retry_after
        .into_iter()
        .chain(method_wait)
        .filter(|v| !v.is_zero())
        .max()
    {
        return Some(
            RegistrationState::CoolingDown {
                until: Timestamp::now().saturating_add(wait),
            }
            .into(),
        );
    }
    if (200..300).contains(&status)
        && response.status == Status::Fail
        && response.reason == Some(Reason::Incorrect)
        && !response.security_code_set
        && !response.secure_verifier
        && response.verify_pn_device.unwrap_or(0) == 0
        && selected != Method::Other
        && response
            .recommended_method
            .as_ref()
            .or(response.fallback_methods.as_ref())
            .is_some_and(|v| v.contains(&selected))
    {
        return None;
    }
    Some(RegistrationState::Blocked {
        reason: BlockedReason::UnsupportedStep {
            description: format!("preflight requires explicit handling; HTTP={status}, status={:?}, reason={:?}, recommended={:?}, fallback={:?}, device_branch={:?}; no code request sent", response.status, response.reason, response.recommended_method, response.fallback_methods, response.verify_pn_device),
        },
    }.into())
}

impl RegistrationFlow for ComparisonHttpFlow {
    fn consent<'a>(
        &'a self,
        attempt: &'a RegistrationAttempt,
        decision: Option<&'a consent::ConsentDecision>,
    ) -> RegistrationFuture<'a> {
        Box::pin(async move {
            let request = self
                .requests
                .consent_request(attempt, decision)
                .map_err(not_built)?;
            let (http, mut response) = self.send(request).await?;
            if response.consent.login.is_none()
                && let RegistrationState::Blocked {
                    reason: BlockedReason::ConsentRequired { details, .. },
                } = &attempt.state
            {
                response.consent.login = details.login.clone();
            }
            if decision.is_some()
                && (200..300).contains(&http)
                && response.status == wire::RegistrationResponseStatus::Ok
                && (response
                    .consent
                    .pending
                    .as_ref()
                    .is_some_and(|v| !v.is_empty())
                    || response
                        .consent
                        .parent_consent_url
                        .as_ref()
                        .is_some_and(|v| !v.is_empty()))
            {
                // `ok` can still carry a pending consent/parent step.
                response.status = wire::RegistrationResponseStatus::Fail;
                response.reason = Some(wire::RegistrationResponseReason::Consent);
                return Ok(Self::blocked_response(http, &response));
            }
            if decision.is_some()
                && (200..300).contains(&http)
                && response.status == wire::RegistrationResponseStatus::Ok
            {
                let mut completion = Self::completion_from(&mut response, "/v2/consent")?;
                if !completion.credentials.iter().any(|v| v.name == "login")
                    && let Some(login) = &response.consent.login
                {
                    completion = completion.with_credential(ObservedCredential {
                        name: "login".into(),
                        value: Zeroizing::new(login.as_bytes().to_vec()),
                        observed_at: Timestamp::now(),
                        source_step: Some("saved-consent-context".into()),
                    })?;
                }
                return Ok(RegistrationFlowOutput::registered(completion));
            }
            if decision.is_none()
                && (200..300).contains(&http)
                && response.consent.consent_id.is_some()
                && response.consent.consent_version.is_some()
            {
                // Inspection is not registration and cannot accept consent.
                response.status = wire::RegistrationResponseStatus::Fail;
                response.reason = Some(wire::RegistrationResponseReason::Consent);
            }
            Ok(Self::blocked_response(http, &response))
        })
    }
    fn prepare(&self, tx: &mut WriteTx<'_>, attempt: &RegistrationAttempt) -> Result<()> {
        self.requests.prepare(tx, attempt)
    }
    fn prepared_snapshot(&self, tx: &WriteTx<'_>) -> Result<Option<Zeroizing<Vec<u8>>>> {
        self.requests.prepared_snapshot(tx)
    }
    fn restore_prepared(&self, tx: &mut WriteTx<'_>, snapshot: Zeroizing<Vec<u8>>) -> Result<()> {
        self.requests.restore_prepared(tx, snapshot)
    }

    fn begin<'a>(&'a self, attempt: &'a RegistrationAttempt) -> RegistrationFuture<'a> {
        Box::pin(async move {
            // Both requests are built before either is sent, so a build
            // failure can only happen while nothing has left. Once the
            // preflight has been sent, a later failure is not "not sent": the
            // server saw the preflight (and its preparation counter).
            let preflight = self
                .requests
                .preflight_request(attempt)
                .map_err(not_built)?;
            let request = self.requests.code_request(attempt).map_err(not_built)?;
            let preflight_sent = preflight.is_some();
            if let Some(preflight) = preflight {
                let (status, response) = self.send(preflight).await?;
                if let Some(output) = preflight_decision(status, &response, &attempt.method) {
                    return Ok(output);
                }
            }
            let (status_code, response) = match self.send(request).await {
                Err(Error::RequestNotSent(cause)) if preflight_sent => return Err(*cause),
                other => other?,
            };
            if (200..300).contains(&status_code)
                && response.status == wire::RegistrationResponseStatus::Sent
            {
                // The longer of the method's wait and the generic one: a
                // shorter value must never shorten a server-directed wait.
                let retry_wait = match attempt.method {
                    ChallengeMethod::Sms => longest([response.sms_wait, response.retry_after]),
                    ChallengeMethod::Voice => longest([response.voice_wait, response.retry_after]),
                    ChallengeMethod::Other(_) => response.retry_after,
                };
                return Ok(RegistrationState::ChallengePending {
                    handle: ChallengeHandle {
                        answer_not_before: None,
                        attempt: attempt.id,
                        generation: attempt.generation,
                        step: 1,
                        method: attempt.method.clone(),
                        expires_at: None,
                        retry_after: retry_wait.map(|wait| Timestamp::now().saturating_add(wait)),
                    },
                }
                .into());
            }
            Ok(Self::blocked_response(status_code, &response))
        })
    }

    fn request_email_code<'a>(
        &'a self,
        attempt: &'a RegistrationAttempt,
        handle: &'a ChallengeHandle,
        email: Option<&'a str>,
    ) -> RegistrationFuture<'a> {
        Box::pin(async move {
            // 2.26.37.71: the email second factor is a `/v2/code` delivery method
            // (`method=email_otp`); the server emails the address it has on file,
            // so no address travels in the request. Live 2026-09-20: the earlier
            // `/v2/challenge email_enter` guess was answered with HTTP 404.
            if email.is_some() {
                return Err(Error::invalid(
                    "email",
                    "the comparison client sends no address; the server uses the one on file",
                ));
            }
            let request = self
                .requests
                .email_code_request(attempt)
                .map_err(not_built)?;
            let (status_code, response) = self.send(request).await?;
            if (200..300).contains(&status_code)
                && matches!(
                    response.status,
                    wire::RegistrationResponseStatus::Sent | wire::RegistrationResponseStatus::Ok
                )
            {
                // Both are server waits; honour the longer one, as the
                // refusal path does, rather than letting one hide the other.
                let wait = longest([response.email_otp_wait, response.retry_after]);
                return Ok(RegistrationState::EmailCodePending {
                    handle: ChallengeHandle {
                        answer_not_before: None,
                        attempt: attempt.id,
                        generation: attempt.generation,
                        step: handle.step.saturating_add(1),
                        method: ChallengeMethod::Other("email_otp".into()),
                        expires_at: None,
                        retry_after: wait.map(|wait| Timestamp::now().saturating_add(wait)),
                    },
                }
                .into());
            }
            Ok(Self::blocked_response(status_code, &response))
        })
    }

    fn submit<'a>(
        &'a self,
        attempt: &'a RegistrationAttempt,
        handle: &'a ChallengeHandle,
        answer: &'a Answer,
    ) -> RegistrationFuture<'a> {
        Box::pin(async move {
            let request = match answer {
                Answer::Code(code) => self
                    .requests
                    .complete_request(attempt, code)
                    .map_err(not_built)?,
                Answer::Pin(pin) => self
                    .requests
                    .security_request(attempt, pin)
                    .map_err(not_built)?,
                // The emailed code is verified by the same `/v2/register` call as
                // an SMS or voice code (2.26.37.71 `KotlinRegistrationBridge.A0D`).
                Answer::EmailCode(code) => self
                    .requests
                    .complete_request(attempt, code)
                    .map_err(not_built)?,
            };
            let (status_code, mut response) = self.send(request).await?;
            if (200..300).contains(&status_code)
                && response.status == wire::RegistrationResponseStatus::Ok
            {
                return Ok(RegistrationFlowOutput::registered(Self::completion_from(
                    &mut response,
                    match answer {
                        Answer::Code(_) | Answer::EmailCode(_) => "/v2/register",
                        Answer::Pin(_) => "/v2/security",
                    },
                )?));
            }
            // 2.26.37.71 `VerifyPhoneNumber.A5P` launchEmailChallengeFlow: a
            // `challenge` refusal of /v2/register moves to the email second
            // factor. Only email_otp is implemented; wa_old/passkey stay blocked.
            if (200..300).contains(&status_code)
                && response.status == wire::RegistrationResponseStatus::Fail
                && matches!(answer, Answer::Code(_))
                && response.reason == Some(wire::RegistrationResponseReason::Challenge)
                && response
                    .second_factor_methods
                    .as_ref()
                    .is_none_or(|m| m.contains(&wire::SecondFactorMethod::EmailOtp))
            {
                return Ok(RegistrationState::EmailChallengeRequired {
                    handle: ChallengeHandle {
                        answer_not_before: None,
                        attempt: attempt.id,
                        generation: attempt.generation,
                        step: handle.step.saturating_add(1),
                        method: ChallengeMethod::Other("email_otp".into()),
                        expires_at: None,
                        // **Inferred:** the refusal's `email_otp_wait` is the
                        // wait before an email code may be requested (the
                        // `/v2/code` answer carries the same field). Keep the
                        // longer of it and `retry_after`; it only delays.
                        retry_after: longest([response.retry_after, response.email_otp_wait])
                            .map(|wait| Timestamp::now().saturating_add(wait)),
                    },
                }
                .into());
            }
            if (200..300).contains(&status_code)
                && response.status == wire::RegistrationResponseStatus::Fail
                && matches!(answer, Answer::Code(_))
                && response.reason == Some(wire::RegistrationResponseReason::SecurityCode)
            {
                return Ok(RegistrationState::NeedsUserAction {
                    handle: ChallengeHandle {
                        answer_not_before: None,
                        attempt: attempt.id,
                        generation: attempt.generation,
                        step: handle.step.saturating_add(1),
                        method: ChallengeMethod::Other("pin".into()),
                        expires_at: None,
                        retry_after: response
                            .retry_after
                            .map(|wait| Timestamp::now().saturating_add(wait)),
                    },
                }
                .into());
            }
            // 2.26.37.71: `KotlinRegistrationBridge.A08` parses reason `stale`
            // to result 9 (`02S.A1R`), and `VerifyPhoneNumber.A5O` case 9
            // (+0x078e) only forgets the stored SMS code (`I49.A07` removes the
            // `...VerifyPhoneNumber.sms_code` preferences), clears the input
            // (`A16`) and shows an error, staying on code entry with the attempt
            // intact. `blocked` (case 11) instead leaves for the blocked flow.
            // So an expired code returns to a pending challenge, from which
            // `resend_code` asks for a new one; it is not a block.
            //
            // A wrong code is not terminal either (2.26.37.71): `incorrect` (20)
            // takes `A5O`'s default branch (`0682`), which forgets the stored
            // code (`I49.A07`) and returns to code entry; `mismatch` (5) calls
            // `A1v`, which clears the input, forgets the code and arms
            // `GKR.A0s(wait)` = `code_verification_retry_time`, using the
            // server's `retry_after` or 60 000 ms when it is absent or
            // unparsable. That wait gates the next answer, not a resend.
            let retry_reason = matches!(
                response.reason,
                Some(
                    wire::RegistrationResponseReason::Stale
                        | wire::RegistrationResponseReason::Incorrect
                        | wire::RegistrationResponseReason::Mismatch
                )
            );
            if (200..300).contains(&status_code)
                && response.status == wire::RegistrationResponseStatus::Fail
                && matches!(answer, Answer::Code(_))
                && retry_reason
            {
                let answer_not_before = (response.reason
                    == Some(wire::RegistrationResponseReason::Mismatch))
                .then(|| {
                    let wait = response
                        .retry_after
                        .filter(|wait| !wait.is_zero())
                        .unwrap_or(std::time::Duration::from_millis(60_000));
                    Timestamp::now().saturating_add(wait)
                });
                return Ok(RegistrationState::ChallengePending {
                    handle: ChallengeHandle {
                        answer_not_before,
                        attempt: attempt.id,
                        generation: attempt.generation,
                        step: handle.step.saturating_add(1),
                        method: handle.method.clone(),
                        expires_at: None,
                        retry_after: response
                            .retry_after
                            .map(|wait| Timestamp::now().saturating_add(wait)),
                    },
                }
                .into());
            }
            Ok(Self::blocked_response(status_code, &response))
        })
    }
}

enum BeginPlan {
    Existing(RegistrationStep),
    /// The reserved attempt and its exact copy from before the reservation,
    /// restored if nothing is sent.
    Start(Box<(RegistrationAttempt, RegistrationAttempt)>),
}

/// Durable registration service over the protocol store.
#[derive(Debug, Clone)]
pub struct RegistrationService {
    store: Arc<ProtocolStore>,
    profile_id: String,
    flow: Arc<dyn RegistrationFlow>,
}

fn step_for(state: &RegistrationState) -> RegistrationStep {
    match state {
        RegistrationState::Registered => RegistrationStep::Completed,
        RegistrationState::ChallengePending { handle } => {
            RegistrationStep::Action(RequiredAction::EnterCode(handle.clone()))
        }
        RegistrationState::NeedsUserAction { handle } => {
            RegistrationStep::Action(RequiredAction::EnterPin(handle.clone()))
        }
        RegistrationState::EmailChallengeRequired { handle } => {
            RegistrationStep::Action(RequiredAction::RequestEmailCode(handle.clone()))
        }
        RegistrationState::EmailCodePending { handle } => {
            RegistrationStep::Action(RequiredAction::EnterEmailCode(handle.clone()))
        }
        RegistrationState::CoolingDown { until } => {
            RegistrationStep::Action(RequiredAction::WaitForCooldown { until: *until })
        }
        RegistrationState::Blocked { reason } => RegistrationStep::Blocked(reason.clone()),
        RegistrationState::RegistrationOutcomeUnknown { .. } => RegistrationStep::OutcomeUnknown,
        RegistrationState::Unregistered
        | RegistrationState::Preparing
        | RegistrationState::Verifying { .. } => {
            RegistrationStep::Blocked(BlockedReason::UnsupportedStep {
                description: format!("attempt is in transient state {}", state.name()),
            })
        }
    }
}

impl RegistrationService {
    /// Create a service with the given flow.
    pub fn new(
        store: Arc<ProtocolStore>,
        profile: &ProtocolProfile,
        flow: Arc<dyn RegistrationFlow>,
    ) -> Self {
        Self {
            store,
            profile_id: profile.id.clone(),
            flow,
        }
    }

    fn load_current(tx: &WriteTx<'_>) -> Result<Option<RegistrationAttempt>> {
        tx.get_json(Table::RegistrationAttempts, CURRENT_KEY, RECORD_V1)
    }

    fn save_current(tx: &mut WriteTx<'_>, attempt: &RegistrationAttempt) -> Result<()> {
        tx.put_json(Table::RegistrationAttempts, CURRENT_KEY, RECORD_V1, attempt)
    }

    /// An attempt is answered only through the profile (and so the flow) that
    /// created it: another flow could save a local block over a pending server
    /// challenge, or misclassify its own missing support (registration review
    /// L3, 2026-10-13).
    fn check_profile(attempt: &RegistrationAttempt, profile_id: &str) -> Result<()> {
        if attempt.profile_id != profile_id {
            return Err(Error::invalid(
                "registration_state",
                "the saved attempt belongs to another profile; continue it with that profile",
            ));
        }
        Ok(())
    }

    /// Record a failed request. A request that provably did not leave
    /// (`Error::RequestNotSent`) puts back `before`, the attempt exactly as it
    /// was before the reservation (state, method, history), and applies `undo`
    /// (the material the reservation's `prepare` changed and the one-shot latch
    /// it took), all in one transaction. Restoring the copy rather than
    /// transitioning back leaves no trace of a request that never left: it is
    /// not counted by `code_preparation_count` (which feeds the preflight's
    /// `client_metrics.attempts`) and uses none of the bounded history
    /// (registration review L4, 2026-10-13). Recording `outcome_unknown` for it
    /// stranded the attempt, since the HTTPS flow has no reconciliation. Any
    /// other failure may have reached the server and is recorded as `unknown`.
    /// The error is returned either way.
    async fn finish_failed(
        &self,
        attempt: &RegistrationAttempt,
        expected_state: &'static str,
        unknown: RegistrationState,
        before: RegistrationAttempt,
        undo: Undo,
        error: Error,
    ) -> Result<RegistrationStep> {
        if !matches!(error, Error::RequestNotSent(_)) {
            self.finish_transition(
                attempt.id,
                attempt.generation,
                expected_state,
                unknown.into(),
            )
            .await?;
            return Err(error);
        }
        // Logged before the write, so the cause survives a failed restore.
        tracing::warn!(
            error = %error,
            "registration request not sent; restoring the attempt to its state before the reservation"
        );
        let (attempt_id, generation) = (attempt.id, attempt.generation);
        let flow = self.flow.clone();
        let restored = self
            .store
            .run_blocking(move |s| {
                s.write(|tx| {
                    let current = Self::load_current(tx)?.ok_or_else(|| {
                        Error::invalid("registration_state", "attempt disappeared during request")
                    })?;
                    if current.id != attempt_id
                        || current.generation != generation
                        || current.generation != tx.generation()
                        || current.state.name() != expected_state
                        || before.id != attempt_id
                        || before.generation != generation
                    {
                        return Err(Error::invalid(
                            "registration_state",
                            "attempt changed while request was in flight",
                        ));
                    }
                    if let Some(material) = undo.material {
                        flow.restore_prepared(tx, material)?;
                    }
                    if let Some(latch) = undo.latch {
                        tx.delete(Table::RegistrationAttempts, latch)?;
                    }
                    Self::save_current(tx, &before)
                })
            })
            .await;
        if let Err(restore) = restored {
            // A store or material failure leaves the attempt in
            // `preparing`/`verifying` (refused by every command, safe), unless
            // only the commit failed and still landed (a failed `fsync`), which
            // leaves it restored; an attempt that changed or vanished meanwhile
            // is not ours to describe.
            let changed = matches!(
                restore,
                Error::InvalidInput {
                    field: "registration_state",
                    ..
                }
            );
            return Err(Error::invalid(
                "registration_state",
                if changed {
                    format!("{error}; the attempt was not restored: {restore}")
                } else {
                    format!(
                        "{error}; restoring the attempt failed, so it stays in {expected_state} unless the failed write still reached the disk (`lab-signup status` shows which): {restore}"
                    )
                },
            ));
        }
        Err(error)
    }

    async fn finish_transition(
        &self,
        attempt_id: AttemptId,
        generation: AccountGeneration,
        expected_state: &'static str,
        output: RegistrationFlowOutput,
    ) -> Result<RegistrationStep> {
        self.finish_transition_marked(attempt_id, generation, expected_state, output, None)
            .await
    }

    /// [`Self::finish_transition`], also recording `marker` (a key in
    /// `RegistrationAttempts`) in the same transaction, so a state that a
    /// later command recognises only together with its marker can never be
    /// committed without it.
    async fn finish_transition_marked(
        &self,
        attempt_id: AttemptId,
        generation: AccountGeneration,
        expected_state: &'static str,
        output: RegistrationFlowOutput,
        marker: Option<String>,
    ) -> Result<RegistrationStep> {
        self.store
            .run_blocking(move |s| {
                s.write(|tx| {
                    let mut attempt = Self::load_current(tx)?.ok_or_else(|| {
                        Error::invalid("registration_state", "attempt disappeared during request")
                    })?;
                    if attempt.id != attempt_id
                        || attempt.generation != generation
                        || attempt.generation != tx.generation()
                        || attempt.state.name() != expected_state
                    {
                        return Err(Error::invalid(
                            "registration_state",
                            "attempt changed while request was in flight",
                        ));
                    }
                    if output.completion.is_some()
                        && !matches!(output.state, RegistrationState::Registered)
                    {
                        return Err(Error::invalid(
                            "registration_completion",
                            "account material may only accompany Registered state",
                        ));
                    }
                    if matches!(output.state, RegistrationState::Registered) {
                        let mut account = AccountRecord::load_mut(tx)?.unwrap_or_default();
                        if account.generation != AccountGeneration::default()
                            && account.generation != attempt.generation
                        {
                            return Err(Error::invalid(
                                "registration_generation",
                                "account record belongs to another generation",
                            ));
                        }
                        if account
                            .phone
                            .as_ref()
                            .is_some_and(|phone| phone != attempt.phone())
                        {
                            return Err(Error::invalid(
                                "registration_phone",
                                "account record belongs to another phone number",
                            ));
                        }
                        account.generation = attempt.generation;
                        account.phone = Some(attempt.phone.clone());
                        account.registration = RegistrationStatus::Registered;
                        if let Some(completion) = output.completion {
                            if account
                                .credentials
                                .len()
                                .saturating_add(completion.credentials.len())
                                > 64
                            {
                                return Err(Error::ResourceExhausted {
                                    resource: "account registration credentials",
                                    limit: 64,
                                });
                            }
                            if let Some(own_jid) = completion.own_jid {
                                account.own_jid = Some(own_jid);
                            }
                            account.credentials.extend(completion.credentials);
                        }
                        AccountRecord::save(&account, tx)?;
                    }
                    attempt.transition(output.state, Timestamp::now())?;
                    Self::save_current(tx, &attempt)?;
                    if let Some(marker) = &marker {
                        tx.put_json(
                            Table::RegistrationAttempts,
                            marker.as_bytes(),
                            RECORD_V1,
                            &Timestamp::now(),
                        )?;
                    }
                    Ok(step_for(&attempt.state))
                })
            })
            .await
    }

    /// The current attempt, if any.
    pub async fn current(&self) -> Result<Option<RegistrationAttempt>> {
        self.store
            .run_blocking(|s| {
                s.read(|tx| tx.get_json(Table::RegistrationAttempts, CURRENT_KEY, RECORD_V1))
            })
            .await
    }

    /// Start or resume a durable attempt for `phone` and return its step.
    ///
    /// Resumes an existing non-terminal attempt for the same number. A
    /// terminal `Registered` attempt is returned as `Completed` and is never
    /// restarted implicitly (docs/design/README.md section 7).
    pub async fn begin(
        &self,
        phone: PhoneNumber,
        method: ChallengeMethod,
    ) -> Result<RegistrationStep> {
        let profile_id = self.profile_id.clone();
        let flow = self.flow.clone();
        let plan = self
            .store
            .run_blocking(move |s| {
                s.write(|tx| {
                    let now = Timestamp::now();
                    if let Some(mut existing) = Self::load_current(tx)? {
                        if existing.generation == tx.generation()
                            && *existing.phone() == phone
                            && existing.profile_id == profile_id
                        {
                            // An elapsed cooldown, or a first request that
                            // never left, resumes with the same attempt and keys.
                            if matches!(&existing.state,
                                RegistrationState::CoolingDown { until } if now >= *until)
                                || matches!(existing.state, RegistrationState::Unregistered)
                            {
                                // The resume sends with the attempt's method; a
                                // different one was silently ignored (review L2).
                                if existing.method != method {
                                    return Err(Error::invalid(
                                        "registration",
                                        format!(
                                            "this attempt uses {:?}; resume it with that method (begin does not change a saved attempt's delivery method; a lab attempt's method is also frozen in its saved request material)",
                                            existing.method
                                        ),
                                    ));
                                }
                                let before = existing.clone();
                                existing.transition(RegistrationState::Preparing, now)?;
                                Self::save_current(tx, &existing)?;
                                return Ok(BeginPlan::Start(Box::new((existing, before))));
                            }
                            return Ok(BeginPlan::Existing(step_for(&existing.state)));
                        }
                        if matches!(existing.state, RegistrationState::Registered) {
                            return Err(Error::invalid(
                                "phone",
                                "an account is already registered in this store; logout/reset first",
                            ));
                        }
                        // Another number's or profile's attempt in this account
                        // generation was silently replaced, losing its waits,
                        // uncertain outcome or pending challenge and stranding
                        // its key material (registration safety audit). A new
                        // number gets its own store; nothing is replaced.
                        if existing.generation == tx.generation() {
                            return Err(Error::invalid(
                                "registration_state",
                                format!(
                                    "this store holds another number's or profile's attempt (state {}); use a separate store, nothing is replaced",
                                    existing.state.name()
                                ),
                            ));
                        }
                    }
                    let mut attempt = RegistrationAttempt {
                        id: AttemptId::generate()?,
                        generation: tx.generation(),
                        profile_id: profile_id.clone(),
                        phone,
                        method,
                        state: RegistrationState::Unregistered,
                        created_at: now,
                        updated_at: now,
                        history: Vec::new(),
                    };
                    // Not sent: this unregistered copy, with no history, is
                    // saved back, and the next `begin` resumes it with the
                    // attempt's identity and keys. (It used to become an
                    // elapsed cooldown, which read as a server-directed wait.)
                    let before = attempt.clone();
                    attempt.transition(RegistrationState::Preparing, now)?;
                    // Prerequisites and Preparing commit atomically before
                    // any request can be sent (docs/design/README.md section 7).
                    flow.prepare(tx, &attempt)?;
                    Self::save_current(tx, &attempt)?;
                    Ok(BeginPlan::Start(Box::new((attempt, before))))
                })
            })
            .await?;

        let (attempt, before) = match plan {
            BeginPlan::Existing(step) => return Ok(step),
            BeginPlan::Start(start) => *start,
        };
        self.execute_begin(attempt, before, Undo::default()).await
    }

    /// Ask the server for a **new** code on the attempt already in progress.
    ///
    /// A delivered code expires long before the attempt does, so resuming a
    /// saved `ChallengePending` eventually leaves an account that can only be
    /// finished with a code nobody has any more. This is the official client's
    /// "resend" and nothing more: the attempt id, its identity, its keys and its
    /// history are all preserved, no new attempt is created, and the phone must
    /// match the saved one.
    ///
    /// It refuses unless the challenge handle's `retry_after` has elapsed, so a
    /// server-directed wait is honoured rather than talked over, and it refuses
    /// any state other than `ChallengePending` — a cooldown, a block or an
    /// unknown outcome each have their own path and none of them is a resend.
    /// The one block it accepts is a `stale` (expired code) refusal saved
    /// before that refusal returned to a pending challenge.
    pub async fn resend_code(
        &self,
        phone: PhoneNumber,
        method: ChallengeMethod,
    ) -> Result<RegistrationStep> {
        let profile_id = self.profile_id.clone();
        let attempt = self
            .store
            .run_blocking(move |s| {
                s.write(|tx| {
                    let now = Timestamp::now();
                    let mut existing = Self::load_current(tx)?.ok_or_else(|| {
                        Error::invalid("registration", "no attempt to resend a code for")
                    })?;
                    if existing.generation != tx.generation()
                        || *existing.phone() != phone
                        || existing.profile_id != profile_id
                    {
                        return Err(Error::invalid(
                            "registration",
                            "saved attempt belongs to another phone, profile or generation",
                        ));
                    }
                    let wait = match &existing.state {
                        RegistrationState::ChallengePending { handle } => handle.retry_after,
                        // Saved before an expired code returned to a pending
                        // challenge (see `submit`); same resend, same waits.
                        RegistrationState::Blocked {
                            reason:
                                BlockedReason::ServerRefused {
                                    reason:
                                        Some(
                                            wire::RegistrationResponseReason::Stale
                                            | wire::RegistrationResponseReason::Incorrect
                                            | wire::RegistrationResponseReason::Mismatch,
                                        ),
                                    not_before,
                                    ..
                                },
                        } => *not_before,
                        other => {
                            return Err(Error::invalid(
                                "registration",
                                format!(
                                    "a resend needs a pending challenge or an expired-code refusal, not {}",
                                    other.name()
                                ),
                            ));
                        }
                    };
                    if let Some(retry_after) = wait
                        && now < retry_after
                    {
                        return Err(Error::invalid(
                            "registration",
                            "the server asked us to wait before requesting another code",
                        ));
                    }
                    let before = existing.clone();
                    existing.method = method;
                    existing.transition(RegistrationState::Preparing, now)?;
                    Self::save_current(tx, &existing)?;
                    Ok((existing, before))
                })
            })
            .await?;
        let (attempt, before) = attempt;
        self.execute_begin(attempt, before, Undo::default()).await
    }

    /// Explicit one-shot diagnostic recovery for the lab's old, opaque refusals.
    ///
    /// Only legacy records that lost both HTTP status and protocol reason are
    /// eligible. This is never invoked by `begin`. A durable account-local latch
    /// commits before network I/O, so crashes, repeated calls and subsequent
    /// refusals cannot cause another diagnostic resend. Keys and history stay
    /// bound to the same attempt; known refusals and cooldowns are ineligible.
    #[cfg(feature = "experimental-registration-https")]
    pub async fn diagnose_legacy_refusal_once(
        &self,
        phone: PhoneNumber,
    ) -> Result<RegistrationStep> {
        self.diagnose_refusal_once(phone, false).await
    }

    /// Explicit one-shot repeat of HTTP 200 / bad_param to collect improved
    /// diagnostics. Preserves identity and payload; never retries other known
    /// refusals, cooldowns or unknown outcomes. Not called by ordinary begin.
    #[cfg(feature = "experimental-registration-https")]
    pub async fn diagnose_bad_parameter_once(
        &self,
        phone: PhoneNumber,
    ) -> Result<RegistrationStep> {
        self.diagnose_refusal_once(phone, true).await
    }

    #[cfg(feature = "experimental-registration-https")]
    async fn diagnose_refusal_once(
        &self,
        phone: PhoneNumber,
        bad_parameter: bool,
    ) -> Result<RegistrationStep> {
        let latch: &'static [u8] = if bad_parameter {
            b"bad-parameter-diagnostic-used"
        } else {
            b"legacy-refusal-diagnostic-used"
        };
        self.diagnose_refusal_with_latch(phone, bad_parameter, latch)
            .await
    }

    /// One explicitly authorized comparison-envelope experiment after bad_param.
    /// The caller must install the encrypted transport; account identity and
    /// previous diagnostic reservations remain unchanged. Never called by begin.
    #[cfg(feature = "experimental-registration-https")]
    pub async fn diagnose_encrypted_parameter_once(
        &self,
        phone: PhoneNumber,
    ) -> Result<RegistrationStep> {
        self.diagnose_refusal_with_latch(phone, true, b"encrypted-parameter-diagnostic-used")
            .await
    }

    /// One authorized experiment with the observed empty-H envelope fallback.
    /// Requires the caller's matching transport; does not clear prior reservations.
    #[cfg(feature = "experimental-registration-https")]
    pub async fn diagnose_unsigned_envelope_once(
        &self,
        phone: PhoneNumber,
    ) -> Result<RegistrationStep> {
        self.diagnose_refusal_with_latch(phone, true, b"unsigned-envelope-diagnostic-used")
            .await
    }

    /// One authorized additive-field experiment, preserving account identity.
    #[cfg(feature = "experimental-registration-https")]
    pub async fn diagnose_additional_fields_once(
        &self,
        phone: PhoneNumber,
    ) -> Result<RegistrationStep> {
        self.diagnose_refusal_with_latch(phone, true, b"additional-fields-diagnostic-used")
            .await
    }

    /// One post-failure flow-state experiment with explicit additive fields.
    #[cfg(feature = "experimental-registration-https")]
    pub async fn diagnose_flow_fields_once(&self, phone: PhoneNumber) -> Result<RegistrationStep> {
        self.diagnose_refusal_with_latch(phone, true, b"flow-fields-diagnostic-used")
            .await
    }

    /// One explicit experiment with the APK's unknown-build-properties header.
    #[cfg(feature = "experimental-registration-https")]
    pub async fn diagnose_unknown_agent_once(
        &self,
        phone: PhoneNumber,
    ) -> Result<RegistrationStep> {
        self.diagnose_refusal_with_latch(phone, true, b"unknown-agent-diagnostic-used")
            .await
    }

    #[cfg(feature = "experimental-registration-https")]
    async fn diagnose_refusal_with_latch(
        &self,
        phone: PhoneNumber,
        bad_parameter: bool,
        latch: &'static [u8],
    ) -> Result<RegistrationStep> {
        self.retry_refusal_with_latch(
            phone,
            bad_parameter.then_some(wire::RegistrationResponseReason::BadParam),
            latch,
        )
        .await
    }

    /// One explicit retry after HTTP 200 / bad_token with a corrected-token
    /// factory. Its preparation must reject all unrelated configuration changes.
    /// The reservation, token correction and attempt update commit atomically.
    #[cfg(feature = "experimental-registration-https")]
    pub async fn retry_corrected_token_once(&self, phone: PhoneNumber) -> Result<RegistrationStep> {
        self.retry_refusal_with_latch(
            phone,
            Some(wire::RegistrationResponseReason::BadToken),
            b"corrected-token-retry-used",
        )
        .await
    }

    #[cfg(feature = "experimental-registration-https")]
    async fn retry_refusal_with_latch(
        &self,
        phone: PhoneNumber,
        expected_reason: Option<wire::RegistrationResponseReason>,
        latch: &'static [u8],
    ) -> Result<RegistrationStep> {
        self.retry_guarded(phone, expected_reason, latch, false)
            .await
    }

    /// One retry of our historical missing-recommendation guard, not a server
    /// refusal. Rechecks preflight before any code request; preserves identity.
    #[cfg(feature = "experimental-registration-https")]
    pub async fn resume_preflight_once(&self, phone: PhoneNumber) -> Result<RegistrationStep> {
        self.retry_guarded(phone, None, b"resume-preflight-used", true)
            .await
    }

    #[cfg(feature = "experimental-registration-https")]
    async fn retry_guarded(
        &self,
        phone: PhoneNumber,
        expected_reason: Option<wire::RegistrationResponseReason>,
        latch: &'static [u8],
        resume_preflight: bool,
    ) -> Result<RegistrationStep> {
        if self.profile_id != request::LAB_PROFILE_ID {
            return Err(Error::invalid(
                "registration_diagnostic",
                "requires explicit lab profile",
            ));
        }
        let profile_id = self.profile_id.clone();
        let flow = self.flow.clone();
        let attempt = self
            .store
            .run_blocking(move |s| {
                s.write(|tx| {
                    let mut attempt = Self::load_current(tx)?.ok_or_else(|| {
                        Error::invalid("registration_diagnostic", "no saved attempt")
                    })?;
                    let eligible = if resume_preflight {
                        matches!(&attempt.state,
                            RegistrationState::Blocked { reason: BlockedReason::UnsupportedStep { description } }
                            if description.starts_with("preflight requires explicit handling; HTTP=200, status=Fail, reason=Some(Incorrect), recommended=None, fallback=Some(")
                                && description.ends_with("device_branch=None; no code request sent")
                        )
                    } else if let Some(expected_reason) = expected_reason {
                        matches!(
                            attempt.state,
                            RegistrationState::Blocked {
                                reason: BlockedReason::ServerRefused {
                                    code: None,
                                    http_status: Some(200),
                                    reason: Some(reason),
                                    ..
                                }
                            } if reason == expected_reason
                        )
                    } else {
                        matches!(
                            attempt.state,
                            RegistrationState::Blocked {
                                reason: BlockedReason::ServerRefused {
                                    code: None,
                                    http_status: None,
                                    reason: None,
                                    ..
                                }
                            }
                        )
                    };
                    if attempt.generation != tx.generation()
                        || attempt.profile_id != profile_id
                        || *attempt.phone() != phone
                        || !eligible
                    {
                        return Err(Error::invalid(
                            "registration_diagnostic",
                            "saved refusal is not eligible for this diagnostic action",
                        ));
                    }
                    // A bad_param or bad_token from an OTP submission, consent,
                    // security or email-code request has the same saved shape;
                    // answering it with a new code request would ask for
                    // another SMS or call (registration audit 2026-10-13).
                    if !attempt.refused_at_code_request() {
                        return Err(Error::invalid(
                            "registration_diagnostic",
                            "saved refusal did not come from a code request; this action would request a new code",
                        ));
                    }
                    // Every one of these sends a new code request, so the SMS or
                    // voice wait the refusal carried applies.
                    let now = Timestamp::now();
                    if let Some(not_before) = attempt.state.refusal_not_before()
                        && now < not_before
                    {
                        return Err(Error::RateLimited {
                            retry_after: Some(std::time::Duration::from_millis(
                                not_before.as_millis().saturating_sub(now.as_millis()) as u64,
                            )),
                            code: None,
                        });
                    }
                    if tx
                        .get(Table::RegistrationAttempts, latch, RECORD_V1)?
                        .is_some()
                    {
                        return Err(Error::invalid(
                            "registration_diagnostic",
                            "one-shot diagnostic already reserved; no resend",
                        ));
                    }
                    let before = attempt.clone();
                    attempt.transition(RegistrationState::Unregistered, now)?;
                    attempt.transition(RegistrationState::Preparing, now)?;
                    // A correcting `prepare` saves corrected material (a new
                    // token, say); keep what it replaces so a request that
                    // never left can put it back with the refusal.
                    let material = flow.prepared_snapshot(tx)?;
                    flow.prepare(tx, &attempt)?;
                    tx.put_json(Table::RegistrationAttempts, latch, RECORD_V1, &attempt.id)?;
                    Self::save_current(tx, &attempt)?;
                    Ok((attempt, before, material))
                })
            })
            .await?;
        let (attempt, before, material) = attempt;
        // A not-sent failure releases the one-shot only when the material is
        // restored with it: a released latch with corrected material still
        // saved could never be used ("must differ from the saved token").
        let undo = Undo {
            latch: material.is_some().then_some(latch),
            material,
        };
        self.execute_begin(attempt, before, undo).await
    }

    async fn execute_begin(
        &self,
        attempt: RegistrationAttempt,
        before: RegistrationAttempt,
        undo: Undo,
    ) -> Result<RegistrationStep> {
        let result = self.flow.begin(&attempt).await;
        match result {
            Ok(next) => {
                self.finish_transition(attempt.id, attempt.generation, "preparing", next)
                    .await
            }
            Err(error) => {
                self.finish_failed(
                    &attempt,
                    "preparing",
                    RegistrationState::RegistrationOutcomeUnknown { step: 0 },
                    before,
                    undo,
                    error,
                )
                .await
            }
        }
    }

    /// Submit a typed answer for a specific challenge step.
    pub async fn submit(
        &self,
        handle: ChallengeHandle,
        answer: Answer,
    ) -> Result<RegistrationStep> {
        let is_pin = matches!(answer, Answer::Pin(_));
        let is_email = matches!(answer, Answer::EmailCode(_));
        let (Answer::Code(value) | Answer::Pin(value) | Answer::EmailCode(value)) = &answer;
        if value.is_empty() || value.len() > 16 || !value.bytes().all(|v| v.is_ascii_digit()) {
            return Err(Error::invalid(
                "challenge",
                "answer must contain 1..=16 ASCII digits",
            ));
        }
        let profile_id = self.profile_id.clone();
        let (attempt, bound, before) = self
            .store
            .run_blocking(move |s| {
                s.write(|tx| {
                    let mut attempt = Self::load_current(tx)?.ok_or_else(|| {
                        Error::invalid("challenge", "no registration attempt in progress")
                    })?;
                    Self::check_profile(&attempt, &profile_id)?;
                    let bound = match &attempt.state {
                        RegistrationState::ChallengePending { handle: h }
                        | RegistrationState::NeedsUserAction { handle: h }
                        | RegistrationState::EmailCodePending { handle: h } => h.clone(),
                        other => {
                            return Err(Error::invalid(
                                "challenge",
                                format!(
                                    "attempt is not awaiting an answer (state {})",
                                    other.name()
                                ),
                            ));
                        }
                    };
                    if is_pin != matches!(attempt.state, RegistrationState::NeedsUserAction { .. })
                        || is_email
                            != matches!(attempt.state, RegistrationState::EmailCodePending { .. })
                    {
                        return Err(Error::invalid(
                            "challenge",
                            "answer type does not match the current challenge",
                        ));
                    }
                    if bound.attempt != handle.attempt
                        || bound.generation != handle.generation
                        || bound.step != handle.step
                        || bound.generation != tx.generation()
                    {
                        return Err(Error::invalid(
                            "challenge",
                            "handle is not bound to the current step",
                        ));
                    }
                    if let Some(exp) = bound.expires_at
                        && Timestamp::now() > exp
                    {
                        return Err(Error::RateLimited {
                            retry_after: None,
                            code: None,
                        });
                    }
                    let now = Timestamp::now();
                    // A `mismatch` wait gates the next code (see `ChallengeHandle`).
                    if !is_pin
                        && let Some(until) = bound.answer_not_before
                        && now < until
                    {
                        return Err(Error::RateLimited {
                            retry_after: Some(std::time::Duration::from_millis(
                                until.as_millis().saturating_sub(now.as_millis()) as u64,
                            )),
                            code: None,
                        });
                    }
                    // OTP retry_after records delivery wait, not an answer wait.
                    // The PIN challenge instead carries /register's generic wait.
                    if is_pin
                        && let Some(until) = bound.retry_after
                        && now < until
                    {
                        return Err(Error::RateLimited {
                            retry_after: Some(std::time::Duration::from_millis(
                                until.as_millis().saturating_sub(now.as_millis()) as u64,
                            )),
                            code: None,
                        });
                    }
                    let before = attempt.clone();
                    attempt.transition(RegistrationState::Verifying { step: bound.step }, now)?;
                    Self::save_current(tx, &attempt)?;
                    Ok((attempt, bound, before))
                })
            })
            .await?;

        let result = self.flow.submit(&attempt, &bound, &answer).await;
        match result {
            Ok(next) => {
                self.finish_transition(attempt.id, attempt.generation, "verifying", next)
                    .await
            }
            Err(error) => {
                self.finish_failed(
                    &attempt,
                    "verifying",
                    RegistrationState::RegistrationOutcomeUnknown { step: bound.step },
                    before,
                    Undo::default(),
                    error,
                )
                .await
            }
        }
    }

    /// Ask the server to send the email second-factor code. Requires the
    /// `EmailChallengeRequired` state; the address, when given, is used for
    /// this request only and never persisted.
    pub async fn request_email_code(
        &self,
        handle: ChallengeHandle,
        email: Option<String>,
    ) -> Result<RegistrationStep> {
        // Rejected before any transition: the comparison client sends no
        // address, the server emails the one it has on file.
        if email.is_some() {
            return Err(Error::invalid(
                "email",
                "the comparison client sends no address; the server uses the one on file",
            ));
        }
        let profile_id = self.profile_id.clone();
        let (attempt, bound, before) = self
            .store
            .run_blocking(move |s| {
                s.write(|tx| {
                    let mut attempt = Self::load_current(tx)?.ok_or_else(|| {
                        Error::invalid("challenge", "no registration attempt in progress")
                    })?;
                    Self::check_profile(&attempt, &profile_id)?;
                    let RegistrationState::EmailChallengeRequired { handle: bound } =
                        &attempt.state
                    else {
                        return Err(Error::invalid(
                            "challenge",
                            format!(
                                "attempt is not awaiting an email code request (state {})",
                                attempt.state.name()
                            ),
                        ));
                    };
                    let bound = bound.clone();
                    if bound.attempt != handle.attempt
                        || bound.generation != handle.generation
                        || bound.step != handle.step
                        || bound.generation != tx.generation()
                    {
                        return Err(Error::invalid(
                            "challenge",
                            "handle is not bound to the current step",
                        ));
                    }
                    let now = Timestamp::now();
                    if let Some(until) = bound.retry_after
                        && now < until
                    {
                        return Err(Error::RateLimited {
                            retry_after: Some(std::time::Duration::from_millis(
                                until.as_millis().saturating_sub(now.as_millis()) as u64,
                            )),
                            code: None,
                        });
                    }
                    let before = attempt.clone();
                    attempt.transition(RegistrationState::Verifying { step: bound.step }, now)?;
                    Self::save_current(tx, &attempt)?;
                    Ok((attempt, bound, before))
                })
            })
            .await?;
        let result = self
            .flow
            .request_email_code(&attempt, &bound, email.as_deref())
            .await;
        match result {
            Ok(next) => {
                self.finish_transition(attempt.id, attempt.generation, "verifying", next)
                    .await
            }
            Err(error) => {
                // The server refused with an HTTP error status and a body we
                // could not read (for example an HTTP 404 with a non-JSON
                // body). Only this kind of unknown outcome may later be
                // resumed. An unreadable 2xx answer, an oversized body, a
                // timeout or a reset after writing may have sent an email and
                // stays unknown (registration review M1, 2026-10-13).
                let answered = matches!(
                    &error,
                    Error::InvalidInput {
                        field: "registration_refusal_body",
                        ..
                    }
                );
                let unknown = RegistrationState::RegistrationOutcomeUnknown { step: bound.step };
                if answered {
                    // `resume_email_challenge` accepts this unknown state only
                    // with its marker, so both commit together: a crash in
                    // between would otherwise strand an attempt the server
                    // definitely answered (Codex review on PR #4).
                    self.finish_transition_marked(
                        attempt.id,
                        attempt.generation,
                        "verifying",
                        unknown.into(),
                        Some(email_answered_key(attempt.id, bound.step)),
                    )
                    .await?;
                    return Err(error);
                }
                self.finish_failed(
                    &attempt,
                    "verifying",
                    unknown,
                    before,
                    Undo::default(),
                    error,
                )
                .await
            }
        }
    }

    /// Operator recovery, no network: an attempt refused with `challenge`
    /// before this branch existed moves to `EmailChallengeRequired` so the
    /// email second factor can be requested. Only that exact refusal qualifies.
    ///
    /// Each of the two recoveries (a `challenge` refusal, a lost email-code
    /// request) is allowed once per attempt, recorded in the same transaction.
    /// Without that, request, lose, resume and request again could repeat
    /// without bound, re-sending an email request whose outcome is uncertain
    /// (registration safety audit, 2026-10-13).
    pub async fn resume_email_challenge(&self) -> Result<RegistrationStep> {
        let profile_id = self.profile_id.clone();
        self.store
            .run_blocking(move |s| {
                s.write(|tx| {
                    let mut attempt = Self::load_current(tx)?.ok_or_else(|| {
                        Error::invalid("challenge", "no registration attempt in progress")
                    })?;
                    Self::check_profile(&attempt, &profile_id)?;
                    let refused_with_challenge = matches!(
                        attempt.state,
                        RegistrationState::Blocked {
                            reason: BlockedReason::ServerRefused {
                                reason: Some(wire::RegistrationResponseReason::Challenge),
                                ..
                            }
                        }
                    );
                    // A lost outcome of the email-code request itself (for
                    // example an HTTP 404 with a non-JSON body) may also come
                    // back to the request step: the history must show it.
                    // Only an email-code request the server answered with
                    // something unreadable; a timeout or reset after writing
                    // stays unknown and is never re-sent.
                    let lost_email_request = match attempt.state {
                        RegistrationState::RegistrationOutcomeUnknown { step } => {
                            attempt
                                .history
                                .iter()
                                .rev()
                                .nth(1)
                                .is_some_and(|t| t.from == "email_challenge_required")
                                && tx
                                    .get(
                                        Table::RegistrationAttempts,
                                        email_answered_key(attempt.id, step).as_bytes(),
                                        RECORD_V1,
                                    )?
                                    .is_some()
                        }
                        _ => false,
                    };
                    if !refused_with_challenge && !lost_email_request {
                        return Err(Error::invalid(
                            "challenge",
                            format!(
                                "only a `challenge` refusal or a lost email-code request can resume (state {})",
                                attempt.state.name()
                            ),
                        ));
                    }
                    let latch = format!(
                        "email-challenge-resumed:{}:{}",
                        if refused_with_challenge { "challenge" } else { "lost-request" },
                        hex::encode(attempt.id.0)
                    );
                    if tx
                        .get(Table::RegistrationAttempts, latch.as_bytes(), RECORD_V1)?
                        .is_some()
                    {
                        return Err(Error::invalid(
                            "challenge",
                            "this recovery was already used for this attempt; an uncertain email-code request is not re-sent again",
                        ));
                    }
                    tx.put_json(
                        Table::RegistrationAttempts,
                        latch.as_bytes(),
                        RECORD_V1,
                        &Timestamp::now(),
                    )?;
                    let step = u32::try_from(attempt.history_len()).unwrap_or(u32::MAX);
                    let now = Timestamp::now();
                    // A `challenge` refusal's `email_otp_wait` (when it came
                    // without `retry_after`) still gates the email request.
                    let retry_after = attempt.state.refusal_not_before();
                    attempt.transition(
                        RegistrationState::EmailChallengeRequired {
                            handle: ChallengeHandle {
                                answer_not_before: None,
                                attempt: attempt.id,
                                generation: attempt.generation,
                                step,
                                method: ChallengeMethod::Other("email_otp".into()),
                                expires_at: None,
                                retry_after,
                            },
                        },
                        now,
                    )?;
                    Self::save_current(tx, &attempt)?;
                    Ok(step_for(&attempt.state))
                })
            })
            .await
    }

    /// Reconcile an attempt after a request result was lost.
    ///
    /// The profile flow performs its observed read-safe reconciliation with
    /// no store transaction held. Failure leaves the attempt in
    /// `RegistrationOutcomeUnknown`; success commits the returned state only
    /// if the same attempt and generation are still current.
    pub async fn reconcile(&self) -> Result<RegistrationStep> {
        let attempt = self
            .current()
            .await?
            .ok_or_else(|| Error::invalid("registration_state", "no registration attempt"))?;
        if !matches!(
            attempt.state,
            RegistrationState::RegistrationOutcomeUnknown { .. }
        ) {
            return Err(Error::invalid(
                "registration_state",
                format!(
                    "attempt is not awaiting reconciliation (state {})",
                    attempt.state.name()
                ),
            ));
        }
        let next = self.flow.reconcile(&attempt).await?;
        self.finish_transition(attempt.id, attempt.generation, "outcome_unknown", next)
            .await
    }

    /// Explicitly discard a blocked or completed attempt. This is a local
    /// operation; it does not log out or contact the server.
    pub async fn reset(&self) -> Result<()> {
        let profile_id = self.profile_id.clone();
        self.store
            .run_blocking(move |s| {
                s.write(|tx| {
                    if let Some(mut a) = Self::load_current(tx)? {
                        // Only the profile that created an attempt may discard
                        // it: the ordinary CLI must not delete a lab attempt's
                        // refusal and evidence (registration safety audit).
                        // A `ProfileIncomplete` block is the exception: it is
                        // the local "cannot proceed" answer of a flow that
                        // sends nothing (`UnsupportedFlow`), so it holds no
                        // server state. Without it, an attempt saved under a
                        // previous pinned version (`android-primary/<version>`)
                        // could be neither replaced nor reset (review L3).
                        // Narrowed after review L3: a library caller could
                        // save `ProfileIncomplete` over a pending server
                        // challenge by answering it through another flow, so
                        // the history must show the attempt never got past
                        // its first local block, and lab attempts never qualify.
                        let local_block = matches!(
                            a.state,
                            RegistrationState::Blocked {
                                reason: BlockedReason::ProfileIncomplete { .. }
                            }
                        ) && a.profile_id != request::LAB_PROFILE_ID
                            && a.history.iter().all(|t| {
                                matches!(t.to.as_str(), "unregistered" | "preparing" | "blocked")
                            });
                        if a.profile_id != profile_id && !local_block {
                            return Err(Error::invalid(
                                "registration_state",
                                "the saved attempt belongs to another profile; it is not reset from this one",
                            ));
                        }
                        if matches!(a.state, RegistrationState::Unregistered) {
                            return Err(Error::invalid(
                                "registration_state",
                                "this attempt has sent nothing yet and keeps its keys; resume it with begin rather than resetting it",
                            ));
                        }
                        if !a.state.is_terminal() {
                            return Err(Error::invalid(
                                "registration_state",
                                "only terminal attempts can be reset",
                            ));
                        }
                        a.transition(RegistrationState::Unregistered, Timestamp::now())?;
                        tx.delete(Table::RegistrationAttempts, CURRENT_KEY)?;
                    }
                    Ok(())
                })
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ids::AccountId;
    use crate::session::AccountRecord;
    use crate::store::{MasterKey, OpenOptions, StaticKeySource};

    fn store() -> (tempfile::TempDir, Arc<ProtocolStore>) {
        let dir = tempfile::tempdir().unwrap();
        let s = ProtocolStore::open(OpenOptions {
            path: dir.path().join("p.redb"),
            account_id: AccountId::new("a").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([1u8; 32]))),
            create: true,
        })
        .unwrap();
        (dir, Arc::new(s))
    }

    fn phone() -> PhoneNumber {
        PhoneNumber::new(1, "5551234567").unwrap()
    }

    /// A resend is the one path from a pending challenge back to `Preparing`,
    /// and it must stay narrow: `begin` still refuses to re-request implicitly,
    /// and no other state reaches `Preparing` that way.
    #[test]
    fn only_a_pending_challenge_may_reopen_for_a_resend() {
        use RegistrationState as S;
        let handle = ChallengeHandle {
            answer_not_before: None,
            attempt: AttemptId([0; 16]),
            generation: AccountGeneration(1),
            step: 1,
            method: ChallengeMethod::Sms,
            expires_at: None,
            retry_after: None,
        };
        assert!(
            S::ChallengePending {
                handle: handle.clone()
            }
            .can_transition(&S::Preparing),
            "a pending challenge reopens for a resend"
        );
        // `Unregistered` and an elapsed `CoolingDown` already reached
        // `Preparing` before this existed — those are the ordinary start and
        // the cooldown resume. The states below never should: a finished
        // registration, a block, and the two that are waiting on the operator
        // rather than on the server.
        for state in [
            S::Preparing,
            S::Registered,
            S::NeedsUserAction {
                handle: handle.clone(),
            },
            S::EmailChallengeRequired {
                handle: handle.clone(),
            },
            S::Blocked {
                reason: BlockedReason::UnsupportedStep {
                    description: "x".into(),
                },
            },
            S::Blocked {
                reason: BlockedReason::ServerRefused {
                    code: None,
                    http_status: Some(200),
                    reason: Some(wire::RegistrationResponseReason::Blocked),
                    not_before: None,
                },
            },
        ] {
            assert!(
                !state.can_transition(&S::Preparing),
                "{} must not reopen for a resend",
                state.name()
            );
        }
    }

    #[test]
    fn transitions_follow_the_table() {
        use RegistrationState as S;
        let h = ChallengeHandle {
            answer_not_before: None,
            attempt: AttemptId([0; 16]),
            generation: AccountGeneration(1),
            step: 1,
            method: ChallengeMethod::Sms,
            expires_at: None,
            retry_after: None,
        };
        assert!(S::Unregistered.can_transition(&S::Preparing));
        assert!(!S::Unregistered.can_transition(&S::Registered));
        assert!(S::Preparing.can_transition(&S::ChallengePending { handle: h.clone() }));
        assert!(S::Preparing.can_transition(&S::RegistrationOutcomeUnknown { step: 0 }));
        assert!(
            S::ChallengePending { handle: h.clone() }.can_transition(&S::Verifying { step: 1 })
        );
        assert!(S::Verifying { step: 1 }.can_transition(&S::Registered));
        assert!(
            S::Verifying { step: 1 }.can_transition(&S::RegistrationOutcomeUnknown { step: 1 })
        );
        assert!(
            !S::Registered.can_transition(&S::Preparing),
            "no implicit re-registration"
        );
        assert!(S::Registered.can_transition(&S::Unregistered));
        assert!(
            S::Verifying { step: 2 }
                .can_transition(&S::EmailChallengeRequired { handle: h.clone() })
        );
        assert!(
            S::EmailChallengeRequired { handle: h.clone() }
                .can_transition(&S::Verifying { step: 2 })
        );
        assert!(
            S::Verifying { step: 2 }.can_transition(&S::EmailCodePending { handle: h.clone() })
        );
        assert!(
            S::EmailCodePending { handle: h.clone() }.can_transition(&S::Verifying { step: 3 })
        );
        assert!(
            S::Blocked {
                reason: BlockedReason::ServerRefused {
                    code: None,
                    http_status: Some(200),
                    reason: Some(wire::RegistrationResponseReason::Challenge),
                    not_before: None,
                }
            }
            .can_transition(&S::EmailChallengeRequired { handle: h.clone() })
        );
        assert!(
            !S::Blocked {
                reason: BlockedReason::ServerRefused {
                    code: None,
                    http_status: Some(200),
                    reason: Some(wire::RegistrationResponseReason::SecurityCode),
                    not_before: None,
                }
            }
            .can_transition(&S::EmailChallengeRequired { handle: h.clone() })
        );
        assert!(
            !S::Blocked {
                reason: BlockedReason::ServerRefused {
                    code: None,
                    http_status: None,
                    reason: None,
                    not_before: None,
                }
            }
            .can_transition(&S::ChallengePending { handle: h })
        );
    }

    #[tokio::test]
    async fn begin_with_unsupported_flow_is_blocked_and_persisted() {
        let (_d, s) = store();
        let profile = ProtocolProfile::android_reference().unwrap();
        let svc = RegistrationService::new(
            s.clone(),
            &profile,
            Arc::new(UnsupportedFlow::for_profile(&profile)),
        );
        assert!(svc.current().await.unwrap().is_none());
        let step = svc.begin(phone(), ChallengeMethod::Sms).await.unwrap();
        match step {
            RegistrationStep::Blocked(BlockedReason::ProfileIncomplete { missing }) => {
                assert!(missing.iter().any(|m| m == "registration.endpoints"));
            }
            other => panic!("unexpected {other:?}"),
        }
        let cur = svc.current().await.unwrap().unwrap();
        assert!(matches!(cur.state, RegistrationState::Blocked { .. }));
        assert_eq!(cur.history_len(), 2);
        // Begin again resumes the same blocked attempt instead of looping.
        let again = svc.begin(phone(), ChallengeMethod::Sms).await.unwrap();
        assert!(matches!(again, RegistrationStep::Blocked(_)));
        assert_eq!(svc.current().await.unwrap().unwrap().id, cur.id);
        // Submitting anything is rejected: nothing is awaiting an answer.
        let handle = ChallengeHandle {
            answer_not_before: None,
            attempt: cur.id,
            generation: cur.generation,
            step: 1,
            method: ChallengeMethod::Sms,
            expires_at: None,
            retry_after: None,
        };
        let err = svc
            .submit(handle, Answer::Code(Zeroizing::new("928374".into())))
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            Error::InvalidInput {
                field: "challenge",
                ..
            }
        ));
        // Reset is explicit and only for terminal attempts.
        svc.reset().await.unwrap();
        assert!(svc.current().await.unwrap().is_none());
    }

    #[derive(Debug, Default)]
    struct CooldownFlow {
        prepare_calls: std::sync::atomic::AtomicUsize,
        begin_calls: std::sync::atomic::AtomicUsize,
    }

    impl RegistrationFlow for CooldownFlow {
        fn prepare(&self, _tx: &mut WriteTx<'_>, _attempt: &RegistrationAttempt) -> Result<()> {
            self.prepare_calls
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }

        fn begin<'a>(&'a self, attempt: &'a RegistrationAttempt) -> RegistrationFuture<'a> {
            Box::pin(async move {
                if self
                    .begin_calls
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                    == 0
                {
                    return Ok(RegistrationState::CoolingDown {
                        until: Timestamp::default(),
                    }
                    .into());
                }
                Ok(RegistrationState::ChallengePending {
                    handle: ChallengeHandle {
                        answer_not_before: None,
                        attempt: attempt.id,
                        generation: attempt.generation,
                        step: 1,
                        method: attempt.method.clone(),
                        expires_at: None,
                        retry_after: None,
                    },
                }
                .into())
            })
        }

        fn submit<'a>(
            &'a self,
            _attempt: &'a RegistrationAttempt,
            _handle: &'a ChallengeHandle,
            _answer: &'a Answer,
        ) -> RegistrationFuture<'a> {
            Box::pin(async { Err(Error::invalid("test", "submit is unused")) })
        }
    }

    #[tokio::test]
    async fn elapsed_cooldown_resumes_without_regenerating_prerequisites() {
        let (_d, s) = store();
        let profile = ProtocolProfile::android_reference().unwrap();
        let flow = Arc::new(CooldownFlow::default());
        let svc = RegistrationService::new(s, &profile, flow.clone());

        let first = svc.begin(phone(), ChallengeMethod::Sms).await.unwrap();
        assert!(matches!(
            first,
            RegistrationStep::Action(RequiredAction::WaitForCooldown { .. })
        ));
        let second = svc.begin(phone(), ChallengeMethod::Sms).await.unwrap();
        assert!(matches!(
            second,
            RegistrationStep::Action(RequiredAction::EnterCode(_))
        ));
        assert_eq!(
            flow.prepare_calls.load(std::sync::atomic::Ordering::SeqCst),
            1
        );
        assert_eq!(
            flow.begin_calls.load(std::sync::atomic::Ordering::SeqCst),
            2
        );
    }

    #[derive(Debug)]
    struct ComparisonRequests;

    impl ComparisonRequestFactory for ComparisonRequests {
        fn prepare(&self, tx: &mut WriteTx<'_>, attempt: &RegistrationAttempt) -> Result<()> {
            AccountRecord {
                generation: attempt.generation,
                phone: Some(attempt.phone.clone()),
                ..AccountRecord::default()
            }
            .save(tx)
        }

        fn code_request(
            &self,
            _attempt: &RegistrationAttempt,
        ) -> Result<wire::EncodedRegistrationRequest> {
            wire::WHATSAPP_2_26_37_70.request(
                wire::RegistrationEndpoint::Code,
                &wire::RegistrationForm::new(),
            )
        }

        fn complete_request(
            &self,
            _attempt: &RegistrationAttempt,
            code: &Zeroizing<String>,
        ) -> Result<wire::EncodedRegistrationRequest> {
            let mut form = wire::RegistrationForm::new();
            form.push("code", code.as_str())?;
            wire::WHATSAPP_2_26_37_70.request(wire::RegistrationEndpoint::Register, &form)
        }

        fn security_request(
            &self,
            _attempt: &RegistrationAttempt,
            pin: &Zeroizing<String>,
        ) -> Result<wire::EncodedRegistrationRequest> {
            let mut form = wire::RegistrationForm::new();
            form.push("code", pin.as_str())?;
            wire::WHATSAPP_2_26_37_70.request(wire::RegistrationEndpoint::Security, &form)
        }

        fn challenge_request(
            &self,
            _attempt: &RegistrationAttempt,
            params: &wire::RegistrationChallengeParams,
        ) -> Result<wire::EncodedRegistrationRequest> {
            let mut form = wire::RegistrationForm::new();
            params.append_to_form(&mut form)?;
            wire::WHATSAPP_2_26_37_70.request(wire::RegistrationEndpoint::Challenge, &form)
        }

        fn email_code_request(
            &self,
            _attempt: &RegistrationAttempt,
        ) -> Result<wire::EncodedRegistrationRequest> {
            let mut form = wire::RegistrationForm::new();
            form.push("method", "email_otp")?;
            wire::WHATSAPP_2_26_37_70.request(wire::RegistrationEndpoint::Code, &form)
        }
    }

    /// `ComparisonRequests` with the `/v2/exist` preflight enabled.
    #[derive(Debug)]
    struct PreflightRequests;

    impl ComparisonRequestFactory for PreflightRequests {
        fn prepare(&self, tx: &mut WriteTx<'_>, attempt: &RegistrationAttempt) -> Result<()> {
            ComparisonRequests.prepare(tx, attempt)
        }
        fn preflight_request(
            &self,
            _attempt: &RegistrationAttempt,
        ) -> Result<Option<wire::EncodedRegistrationRequest>> {
            wire::WHATSAPP_2_26_37_70
                .request(
                    wire::RegistrationEndpoint::Exist,
                    &wire::RegistrationForm::new(),
                )
                .map(Some)
        }
        fn code_request(
            &self,
            attempt: &RegistrationAttempt,
        ) -> Result<wire::EncodedRegistrationRequest> {
            ComparisonRequests.code_request(attempt)
        }
        fn complete_request(
            &self,
            attempt: &RegistrationAttempt,
            code: &Zeroizing<String>,
        ) -> Result<wire::EncodedRegistrationRequest> {
            ComparisonRequests.complete_request(attempt, code)
        }
        fn security_request(
            &self,
            attempt: &RegistrationAttempt,
            pin: &Zeroizing<String>,
        ) -> Result<wire::EncodedRegistrationRequest> {
            ComparisonRequests.security_request(attempt, pin)
        }
        fn challenge_request(
            &self,
            attempt: &RegistrationAttempt,
            params: &wire::RegistrationChallengeParams,
        ) -> Result<wire::EncodedRegistrationRequest> {
            ComparisonRequests.challenge_request(attempt, params)
        }
        fn email_code_request(
            &self,
            attempt: &RegistrationAttempt,
        ) -> Result<wire::EncodedRegistrationRequest> {
            ComparisonRequests.email_code_request(attempt)
        }
    }

    /// Once the preflight has left, a failed code request is not "not sent":
    /// the attempt records an unknown outcome instead of being restored.
    #[tokio::test]
    async fn a_failure_after_a_sent_preflight_is_not_reported_as_not_sent() {
        let (_d, s) = store();
        let profile = ProtocolProfile::android_reference().unwrap();
        let transport = Arc::new(MaybeSentTransport {
            steps: std::sync::Mutex::new(std::collections::VecDeque::from([
                (
                    "/v2/exist",
                    Some(
                        br#"{"status":"fail","reason":"incorrect","recommended_method":["sms"]}"#
                            .as_slice(),
                    ),
                ),
                ("/v2/code", None),
            ])),
        });
        let service = RegistrationService::new(
            s.clone(),
            &profile,
            Arc::new(ComparisonHttpFlow::new(
                Arc::new(PreflightRequests),
                transport.clone(),
            )),
        );
        let error = service
            .begin(phone(), ChallengeMethod::Sms)
            .await
            .unwrap_err();
        assert!(!matches!(error, Error::RequestNotSent(_)), "{error}");
        let saved = service.current().await.unwrap().unwrap();
        assert!(
            matches!(
                saved.state,
                RegistrationState::RegistrationOutcomeUnknown { .. }
            ),
            "{:?}",
            saved.state
        );
        assert!(transport.steps.lock().unwrap().is_empty());
    }

    #[derive(Debug)]
    struct ComparisonTransport {
        responses: std::sync::Mutex<std::collections::VecDeque<(&'static str, &'static [u8])>>,
    }

    impl wire::RegistrationTransport for ComparisonTransport {
        fn send<'a>(
            &'a self,
            request: wire::EncodedRegistrationRequest,
        ) -> wire::RegistrationTransportFuture<'a> {
            Box::pin(async move {
                let (path, body) = self.responses.lock().unwrap().pop_front().unwrap();
                assert!(request.url().ends_with(path));
                wire::RegistrationHttpResponse::new(200, Zeroizing::new(body.to_vec()))
            })
        }
    }

    /// Each step answers with a response, or `None` for a request that
    /// provably did not leave (a connection failure before writing).
    type MaybeSentSteps =
        std::sync::Mutex<std::collections::VecDeque<(&'static str, Option<&'static [u8]>)>>;

    #[derive(Debug)]
    struct MaybeSentTransport {
        steps: MaybeSentSteps,
    }

    impl wire::RegistrationTransport for MaybeSentTransport {
        fn send<'a>(
            &'a self,
            request: wire::EncodedRegistrationRequest,
        ) -> wire::RegistrationTransportFuture<'a> {
            Box::pin(async move {
                let (path, body) = self.steps.lock().unwrap().pop_front().unwrap();
                assert!(request.url().ends_with(path));
                match body {
                    Some(body) => {
                        wire::RegistrationHttpResponse::new(200, Zeroizing::new(body.to_vec()))
                    }
                    None => Err(Error::RequestNotSent(Box::new(Error::Io(
                        std::io::Error::other("connection refused before writing"),
                    )))),
                }
            })
        }
    }

    /// Snapshots the material but cannot restore it (the trait default).
    #[cfg(feature = "experimental-registration-https")]
    #[derive(Debug)]
    struct SnapshotOnlyRequests;

    #[cfg(feature = "experimental-registration-https")]
    impl ComparisonRequestFactory for SnapshotOnlyRequests {
        fn prepare(&self, tx: &mut WriteTx<'_>, attempt: &RegistrationAttempt) -> Result<()> {
            ComparisonRequests.prepare(tx, attempt)
        }
        fn prepared_snapshot(&self, _tx: &WriteTx<'_>) -> Result<Option<Zeroizing<Vec<u8>>>> {
            Ok(Some(Zeroizing::new(vec![1])))
        }
        fn code_request(
            &self,
            attempt: &RegistrationAttempt,
        ) -> Result<wire::EncodedRegistrationRequest> {
            ComparisonRequests.code_request(attempt)
        }
        fn complete_request(
            &self,
            attempt: &RegistrationAttempt,
            code: &Zeroizing<String>,
        ) -> Result<wire::EncodedRegistrationRequest> {
            ComparisonRequests.complete_request(attempt, code)
        }
        fn security_request(
            &self,
            attempt: &RegistrationAttempt,
            pin: &Zeroizing<String>,
        ) -> Result<wire::EncodedRegistrationRequest> {
            ComparisonRequests.security_request(attempt, pin)
        }
    }

    /// A restore that fails keeps the attempt where it was (refused by every
    /// command, safe) and says both why the request was not sent and why the
    /// attempt is stuck, instead of only the store error (review L5).
    #[cfg(feature = "experimental-registration-https")]
    #[tokio::test]
    async fn a_failed_restore_reports_the_cause_and_the_stuck_state() {
        let (_d, s) = store();
        let mut profile = ProtocolProfile::android_reference().unwrap();
        profile.id = request::LAB_PROFILE_ID.into();
        let transport = Arc::new(MaybeSentTransport {
            steps: std::sync::Mutex::new(std::collections::VecDeque::from([
                (
                    "/v2/code",
                    Some(br#"{"status":"fail","reason":"bad_param"}"#.as_slice()),
                ),
                ("/v2/code", None),
            ])),
        });
        let service = RegistrationService::new(
            s.clone(),
            &profile,
            Arc::new(ComparisonHttpFlow::new(
                Arc::new(SnapshotOnlyRequests),
                transport,
            )),
        );
        service.begin(phone(), ChallengeMethod::Sms).await.unwrap();
        let error = service
            .diagnose_bad_parameter_once(phone())
            .await
            .unwrap_err()
            .to_string();
        assert!(error.contains("request not sent"), "{error}");
        assert!(error.contains("restoring the attempt failed"), "{error}");
        assert_eq!(
            service.current().await.unwrap().unwrap().state,
            RegistrationState::Preparing
        );
    }

    /// A request that provably did not leave restores the attempt instead of
    /// stranding it in `outcome_unknown`: a fresh attempt rests `unregistered`
    /// and `begin` resumes it with the same attempt and keys, and an
    /// OTP submission returns to its challenge so the same code can be sent.
    #[tokio::test]
    async fn unsent_requests_restore_the_attempt_instead_of_an_unknown_outcome() {
        let (_d, s) = store();
        let profile = ProtocolProfile::android_reference().unwrap();
        let transport = Arc::new(MaybeSentTransport {
            steps: std::sync::Mutex::new(std::collections::VecDeque::from([
                ("/v2/code", None),
                ("/v2/code", Some(br#"{"status":"sent"}"#.as_slice())),
                ("/v2/code", None),
                ("/v2/register", None),
                (
                    "/v2/register",
                    Some(br#"{"status":"ok","login":"mock-login"}"#.as_slice()),
                ),
            ])),
        });
        let service = RegistrationService::new(
            s.clone(),
            &profile,
            Arc::new(ComparisonHttpFlow::new(
                Arc::new(ComparisonRequests),
                transport.clone(),
            )),
        );
        let error = service
            .begin(phone(), ChallengeMethod::Sms)
            .await
            .unwrap_err();
        assert!(matches!(error, Error::RequestNotSent(_)), "{error}");
        let first = service.current().await.unwrap().unwrap();
        // Not a cooldown: nothing was asked of the server, and status must not
        // suggest a server-directed wait (review L2).
        assert_eq!(first.state, RegistrationState::Unregistered);
        // Restored exactly: a request that never left leaves no history, so
        // it is not counted as a preparation either (review L4).
        assert!(first.history.is_empty(), "{:?}", first.history);
        assert_eq!(first.code_preparation_count(), Some(0));
        // The saved attempt keeps its method; resuming with another is
        // refused rather than silently ignored (review L2).
        let error = service
            .begin(phone(), ChallengeMethod::Voice)
            .await
            .unwrap_err();
        assert!(
            error.to_string().contains("resume it with that method"),
            "{error}"
        );
        let RegistrationStep::Action(RequiredAction::EnterCode(handle)) =
            service.begin(phone(), ChallengeMethod::Sms).await.unwrap()
        else {
            panic!("the unsent attempt resumes")
        };
        assert_eq!(service.current().await.unwrap().unwrap().id, first.id);
        // A resend switching to voice that does not leave keeps the pending
        // challenge and the attempt's original method.
        let error = service
            .resend_code(phone(), ChallengeMethod::Voice)
            .await
            .unwrap_err();
        assert!(matches!(error, Error::RequestNotSent(_)), "{error}");
        let saved = service.current().await.unwrap().unwrap();
        assert_eq!(saved.method, ChallengeMethod::Sms);
        assert!(matches!(
            saved.state,
            RegistrationState::ChallengePending { .. }
        ));
        let code = || Answer::Code(Zeroizing::new("123456".into()));
        let error = service.submit(handle.clone(), code()).await.unwrap_err();
        assert!(matches!(error, Error::RequestNotSent(_)), "{error}");
        let saved = service.current().await.unwrap().unwrap();
        assert!(
            matches!(saved.state, RegistrationState::ChallengePending { .. }),
            "{:?}",
            saved.state
        );
        // The restored handle still answers the same step.
        let RegistrationState::ChallengePending { handle: restored } = saved.state else {
            unreachable!()
        };
        assert_eq!(
            service.submit(restored, code()).await.unwrap(),
            RegistrationStep::Completed
        );
        assert!(transport.steps.lock().unwrap().is_empty());
        // `accounts status` reads the saved state rather than assuming none.
        assert_eq!(current_state_name(&s).unwrap(), Some("registered"));
    }

    /// Another number's `begin` and another profile's `reset` refuse instead
    /// of replacing or deleting the saved attempt.
    #[tokio::test]
    async fn begin_and_reset_never_replace_another_attempt() {
        let (_d, s) = store();
        let profile = ProtocolProfile::android_reference().unwrap();
        let transport = Arc::new(ComparisonTransport {
            responses: std::sync::Mutex::new(std::collections::VecDeque::from([(
                "/v2/code",
                br#"{"status":"fail","reason":"blocked"}"#.as_slice(),
            )])),
        });
        let service = RegistrationService::new(
            s.clone(),
            &profile,
            Arc::new(ComparisonHttpFlow::new(
                Arc::new(ComparisonRequests),
                transport.clone(),
            )),
        );
        service.begin(phone(), ChallengeMethod::Sms).await.unwrap();
        let saved = service.current().await.unwrap().unwrap();
        assert!(saved.state.is_terminal());
        let other = PhoneNumber::new(1, "5557654321").unwrap();
        let error = service
            .begin(other, ChallengeMethod::Sms)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("nothing is replaced"), "{error}");
        let mut lab = profile.clone();
        lab.id = "another-profile".into();
        let foreign = RegistrationService::new(
            s.clone(),
            &lab,
            Arc::new(UnsupportedFlow {
                missing: Vec::new(),
            }),
        );
        assert!(foreign.reset().await.is_err());
        assert_eq!(service.current().await.unwrap().unwrap(), saved);
        // The owning profile may still discard its own terminal attempt.
        service.reset().await.unwrap();
        assert!(service.current().await.unwrap().is_none());
    }

    /// Another profile's flow cannot answer a saved challenge: a flow that
    /// sends nothing would save a local block over the server's pending code,
    /// which `reset` would then accept. The reset exemption also requires a
    /// history that never got past the first local block (review L3).
    #[tokio::test]
    async fn another_profile_cannot_answer_or_reset_a_pending_challenge() {
        let (_d, s) = store();
        let profile = ProtocolProfile::android_reference().unwrap();
        let transport = Arc::new(ComparisonTransport {
            responses: std::sync::Mutex::new(std::collections::VecDeque::from([(
                "/v2/code",
                br#"{"status":"sent"}"#.as_slice(),
            )])),
        });
        let owner = RegistrationService::new(
            s.clone(),
            &profile,
            Arc::new(ComparisonHttpFlow::new(
                Arc::new(ComparisonRequests),
                transport,
            )),
        );
        let RegistrationStep::Action(RequiredAction::EnterCode(handle)) =
            owner.begin(phone(), ChallengeMethod::Sms).await.unwrap()
        else {
            panic!("expected code")
        };
        let mut other = profile.clone();
        other.id = "android-primary/2.0.0.0".into();
        let foreign = RegistrationService::new(
            s.clone(),
            &other,
            Arc::new(UnsupportedFlow {
                missing: Vec::new(),
            }),
        );
        let error = foreign
            .submit(handle, Answer::Code(Zeroizing::new("123456".into())))
            .await
            .unwrap_err();
        assert!(error.to_string().contains("another profile"), "{error}");
        assert!(matches!(
            owner.current().await.unwrap().unwrap().state,
            RegistrationState::ChallengePending { .. }
        ));
        // Even blocked locally, an attempt whose history reached a challenge
        // is not another profile's to reset.
        let mut saved = owner.current().await.unwrap().unwrap();
        saved
            .transition(RegistrationState::Verifying { step: 1 }, Timestamp::now())
            .unwrap();
        saved
            .transition(
                RegistrationState::Blocked {
                    reason: BlockedReason::ProfileIncomplete {
                        missing: Vec::new(),
                    },
                },
                Timestamp::now(),
            )
            .unwrap();
        s.write(|tx| RegistrationService::save_current(tx, &saved))
            .unwrap();
        assert!(foreign.reset().await.is_err());
        assert!(owner.current().await.unwrap().is_some());
    }

    /// An attempt blocked locally by a flow that sends nothing, saved under an
    /// earlier pinned version, can be reset from the current one; a server
    /// refusal saved under another profile still cannot (review L3).
    #[tokio::test]
    async fn a_local_block_from_an_older_version_can_be_reset() {
        let (_d, s) = store();
        let mut old = ProtocolProfile::android_reference().unwrap();
        old.id = "android-primary/2.0.0.0".into();
        let unsupported = || {
            Arc::new(UnsupportedFlow {
                missing: vec!["registration".into()],
            })
        };
        let previous = RegistrationService::new(s.clone(), &old, unsupported());
        assert!(matches!(
            previous.begin(phone(), ChallengeMethod::Sms).await.unwrap(),
            RegistrationStep::Blocked(BlockedReason::ProfileIncomplete { .. })
        ));
        let current_profile = ProtocolProfile::android_reference().unwrap();
        let current = RegistrationService::new(s.clone(), &current_profile, unsupported());
        // Not replaced implicitly by `begin`; an explicit reset clears it.
        assert!(current.begin(phone(), ChallengeMethod::Sms).await.is_err());
        current.reset().await.unwrap();
        assert!(current.current().await.unwrap().is_none());
    }

    /// An unclaimed store whose account material names another number is
    /// not claimed for this one (review L6).
    #[test]
    fn a_store_with_another_numbers_material_is_not_claimed() {
        let (_d, s) = store();
        s.write(|tx| {
            AccountRecord {
                generation: tx.generation(),
                phone: Some(phone()),
                ..AccountRecord::default()
            }
            .save(tx)
        })
        .unwrap();
        let other = PhoneNumber::new(1, "5557654321").unwrap();
        let error = claim_store(&s, &other).unwrap_err();
        assert!(error.to_string().contains("another number"), "{error}");
        assert_eq!(store_owner(&s).unwrap(), None);
        claim_store(&s, &phone()).unwrap();
        assert_eq!(store_owner(&s).unwrap(), Some(phone()));
    }

    #[test]
    fn a_store_without_an_attempt_has_no_state_name() {
        let (_d, s) = store();
        assert_eq!(current_state_name(&s).unwrap(), None);
    }

    #[test]
    fn build_failures_are_classified_as_not_sent_once() {
        let built = not_built(Error::invalid("registration_config", "config changed"));
        assert!(matches!(built, Error::RequestNotSent(_)));
        assert!(built.to_string().contains("config changed"));
        // Already classified: not wrapped twice.
        let Error::RequestNotSent(inner) = not_built(built) else {
            panic!()
        };
        assert!(!matches!(*inner, Error::RequestNotSent(_)));
    }

    #[tokio::test]
    async fn refusal_details_persist_and_begin_does_not_retry() {
        let (_d, s) = store();
        let profile = ProtocolProfile::android_reference().unwrap();
        let transport = Arc::new(ComparisonTransport {
            responses: std::sync::Mutex::new(std::collections::VecDeque::from([(
                "/v2/code",
                br#"{"status":"fail","reason":"bad_token"}"#.as_slice(),
            )])),
        });
        let service = RegistrationService::new(
            s.clone(),
            &profile,
            Arc::new(ComparisonHttpFlow::new(
                Arc::new(ComparisonRequests),
                transport.clone(),
            )),
        );
        let first = service.begin(phone(), ChallengeMethod::Sms).await.unwrap();
        let saved = service.current().await.unwrap().unwrap();
        assert!(matches!(
            saved.state,
            RegistrationState::Blocked {
                reason: BlockedReason::ServerRefused {
                    code: None,
                    http_status: Some(200),
                    reason: Some(wire::RegistrationResponseReason::BadToken),
                    ..
                }
            }
        ));
        let decoded: RegistrationAttempt =
            serde_json::from_slice(&serde_json::to_vec(&saved).unwrap()).unwrap();
        assert_eq!(decoded.state, saved.state);
        let resumed = RegistrationService::new(
            s,
            &profile,
            Arc::new(ComparisonHttpFlow::new(
                Arc::new(ComparisonRequests),
                transport.clone(),
            )),
        );
        assert_eq!(
            resumed.begin(phone(), ChallengeMethod::Sms).await.unwrap(),
            first
        );
        assert!(transport.responses.lock().unwrap().is_empty());
    }

    #[test]
    fn old_refusals_decode_and_unknown_server_text_is_not_persisted() {
        let old: BlockedReason =
            serde_json::from_str(r#"{"ServerRefused":{"code":null}}"#).unwrap();
        assert!(matches!(
            old,
            BlockedReason::ServerRefused {
                code: None,
                http_status: None,
                reason: None,
                ..
            }
        ));
        let response = wire::RegistrationHttpResponse::new(
            403,
            Zeroizing::new(
                br#"{"status":"fail","reason":"private_server_text","login":"secret-login"}"#
                    .to_vec(),
            ),
        )
        .unwrap()
        .parse_json()
        .unwrap();
        let output = ComparisonHttpFlow::blocked_response(403, &response);
        let encoded = serde_json::to_string(&output.state).unwrap();
        assert!(!encoded.contains("private_server_text"));
        assert!(!encoded.contains("secret-login"));
        assert!(matches!(
            output.state,
            RegistrationState::Blocked {
                reason: BlockedReason::ServerRefused {
                    code: Some(403),
                    http_status: Some(403),
                    reason: Some(wire::RegistrationResponseReason::Other),
                    ..
                }
            }
        ));
    }

    #[tokio::test]
    async fn consent_context_survives_restart_without_acceptance_or_url_leaks() {
        let (_d, s) = store();
        let profile = ProtocolProfile::android_reference().unwrap();
        let transport = Arc::new(ComparisonTransport {
            responses: std::sync::Mutex::new(std::collections::VecDeque::from([
                ("/v2/code", br#"{"status":"sent"}"#.as_slice()),
                ("/v2/register", br#"{"status":"fail","reason":"consent","consent_id":3,"consent_version":2,"parent_consent_url":"https://example.invalid/private-consent-token","retry_after":30,"sms_wait":60,"voice_wait":90}"#.as_slice()),
            ])),
        });
        let flow = Arc::new(ComparisonHttpFlow::new(
            Arc::new(ComparisonRequests),
            transport.clone(),
        ));
        let svc = RegistrationService::new(s.clone(), &profile, flow.clone());
        let RegistrationStep::Action(RequiredAction::EnterCode(h)) =
            svc.begin(phone(), ChallengeMethod::Sms).await.unwrap()
        else {
            panic!("expected code")
        };
        let result = svc
            .submit(h, Answer::Code(Zeroizing::new("123456".into())))
            .await
            .unwrap();
        let RegistrationStep::Blocked(BlockedReason::ConsentRequired {
            details,
            retry_after,
            sms_retry_after,
            voice_retry_after,
        }) = &result
        else {
            panic!("expected consent")
        };
        assert_eq!(details.consent_id, Some(3));
        assert_eq!(details.consent_version, Some(2));
        assert!(details.parent_consent_url.is_some());
        assert!(retry_after < sms_retry_after && sms_retry_after < voice_retry_after);
        assert!(!format!("{result:?}").contains("private-consent-token"));
        let restarted = RegistrationService::new(s.clone(), &profile, flow);
        assert_eq!(
            restarted
                .begin(phone(), ChallengeMethod::Sms)
                .await
                .unwrap(),
            result
        );
        assert!(transport.responses.lock().unwrap().is_empty());
        let raw = std::fs::read(s.path()).unwrap();
        assert!(
            !raw.windows(b"private-consent-token".len())
                .any(|v| v == b"private-consent-token")
        );
    }

    #[tokio::test]
    async fn post_otp_refusals_and_unknown_results_never_replay_answers() {
        for body in [
            br#"{"status":"fail","reason":"incorrect"}"#.as_slice(),
            br#"{"status":"fail","reason":"stale"}"#,
            br#"{"status":"fail","reason":"device_confirm_or_second_code","sms_wait":1500,"voice_wait":1500}"#,
            br#"{"status":"fail","reason":"second_code"}"#,
            br#"{"status":"fail","reason":"challenge"}"#,
            br#"{"status":"fail","reason":"consent"}"#,
            br#"{"status":"fail","reason":"too_many_guesses"}"#,
            br#"{"status":"fail","reason":"guessed_too_fast","retry_after":60}"#,
            br#"{"status":"fail","reason":"blocked"}"#,
            br#"{"status":"fail","reason":"future_reason"}"#,
            b"invalid JSON",
        ] {
            let (_d, s) = store();
            let profile = ProtocolProfile::android_reference().unwrap();
            let transport = Arc::new(ComparisonTransport {
                responses: std::sync::Mutex::new(std::collections::VecDeque::from([
                    ("/v2/code", br#"{"status":"sent"}"#.as_slice()),
                    ("/v2/register", body),
                ])),
            });
            let svc = RegistrationService::new(s.clone(), &profile, Arc::new(
                ComparisonHttpFlow::new(Arc::new(ComparisonRequests), transport.clone())));
            let RegistrationStep::Action(RequiredAction::EnterCode(handle)) = svc.begin(phone(), ChallengeMethod::Sms).await.unwrap() else { panic!("expected code") };
            let result = svc.submit(handle.clone(), Answer::Code(Zeroizing::new("123456".into()))).await;
            if body == b"invalid JSON" {
                assert!(result.is_err());
                assert!(matches!(svc.current().await.unwrap().unwrap().state, RegistrationState::RegistrationOutcomeUnknown { .. }));
            } else if body.starts_with(br#"{"status":"fail","reason":"challenge""#) {
                // The email second factor is a supported next step, not a replay.
                assert!(matches!(result.unwrap(), RegistrationStep::Action(RequiredAction::RequestEmailCode(_))));
            } else if body.starts_with(br#"{"status":"fail","reason":"stale""#)
                || body.starts_with(br#"{"status":"fail","reason":"incorrect""#)
            {
                // An expired code returns to code entry on a new step
                // (2.26.37.71 `VerifyPhoneNumber.A5O` case 9); the old
                // handle is still refused below, so nothing is replayed.
                assert!(matches!(result.unwrap(), RegistrationStep::Action(RequiredAction::EnterCode(_))));
            } else {
                assert!(matches!(result.unwrap(), RegistrationStep::Blocked(_) | RegistrationStep::Action(RequiredAction::WaitForCooldown { .. })));
            }
            assert!(svc.submit(handle, Answer::Code(Zeroizing::new("123456".into()))).await.is_err());
            let reopened = RegistrationService::new(s.clone(), &profile, Arc::new(
                ComparisonHttpFlow::new(Arc::new(ComparisonRequests), transport.clone())));
            assert_ne!(reopened.begin(phone(), ChallengeMethod::Sms).await.unwrap(), RegistrationStep::Completed);
            assert!(transport.responses.lock().unwrap().is_empty());
            assert_ne!(s.read(AccountRecord::load).unwrap().unwrap().registration, RegistrationStatus::Registered);
        }
    }

    /// 2.26.37.71 `VerifyPhoneNumber.A5O` case 9: an expired code (`stale`)
    /// keeps the attempt on code entry, so a resend and a new code finish it.
    #[tokio::test]
    async fn stale_code_returns_to_code_entry_and_resends_on_the_same_attempt() {
        let (_d, s) = store();
        let profile = ProtocolProfile::android_reference().unwrap();
        let transport = Arc::new(ComparisonTransport {
            responses: std::sync::Mutex::new(std::collections::VecDeque::from([
                ("/v2/code", br#"{"status":"sent"}"#.as_slice()),
                (
                    "/v2/register",
                    br#"{"status":"fail","login":"15551234567","reason":"stale"}"#.as_slice(),
                ),
                (
                    "/v2/code",
                    br#"{"status":"sent","sms_wait":125}"#.as_slice(),
                ),
                (
                    "/v2/register",
                    br#"{"status":"ok","login":"15551234567","edge_routing_info":"AQ=="}"#
                        .as_slice(),
                ),
            ])),
        });
        let service = RegistrationService::new(
            s.clone(),
            &profile,
            Arc::new(ComparisonHttpFlow::new(
                Arc::new(ComparisonRequests),
                transport.clone(),
            )),
        );
        let RegistrationStep::Action(RequiredAction::EnterCode(h)) =
            service.begin(phone(), ChallengeMethod::Sms).await.unwrap()
        else {
            panic!("expected code")
        };
        let id = service.current().await.unwrap().unwrap().id;
        let RegistrationStep::Action(RequiredAction::EnterCode(h2)) = service
            .submit(h.clone(), Answer::Code(Zeroizing::new("123456".into())))
            .await
            .unwrap()
        else {
            panic!("an expired code returns to code entry, not a block")
        };
        assert_eq!(h2.step, h.step + 1);
        assert_eq!(h2.method, ChallengeMethod::Sms);
        let RegistrationStep::Action(RequiredAction::EnterCode(h3)) = service
            .resend_code(phone(), ChallengeMethod::Sms)
            .await
            .unwrap()
        else {
            panic!("the resend asks for a new code")
        };
        assert_eq!(
            service.current().await.unwrap().unwrap().id,
            id,
            "same attempt"
        );
        assert_eq!(
            service
                .submit(h3, Answer::Code(Zeroizing::new("654321".into())))
                .await
                .unwrap(),
            RegistrationStep::Completed
        );
        assert!(transport.responses.lock().unwrap().is_empty());
    }

    /// 2.26.37.71 `VerifyPhoneNumber.A1v`: a `mismatch` returns to code entry
    /// but gates the next answer for `retry_after`, else 60 s.
    #[tokio::test]
    async fn mismatch_returns_to_code_entry_with_an_answer_wait() {
        let (_d, s) = store();
        let profile = ProtocolProfile::android_reference().unwrap();
        let transport = Arc::new(ComparisonTransport {
            responses: std::sync::Mutex::new(std::collections::VecDeque::from([
                ("/v2/code", br#"{"status":"sent"}"#.as_slice()),
                (
                    "/v2/register",
                    br#"{"status":"fail","reason":"mismatch"}"#.as_slice(),
                ),
            ])),
        });
        let service = RegistrationService::new(
            s.clone(),
            &profile,
            Arc::new(ComparisonHttpFlow::new(
                Arc::new(ComparisonRequests),
                transport.clone(),
            )),
        );
        let RegistrationStep::Action(RequiredAction::EnterCode(h)) =
            service.begin(phone(), ChallengeMethod::Sms).await.unwrap()
        else {
            panic!("expected code")
        };
        let RegistrationStep::Action(RequiredAction::EnterCode(h2)) = service
            .submit(h, Answer::Code(Zeroizing::new("111111".into())))
            .await
            .unwrap()
        else {
            panic!("a mismatch returns to code entry")
        };
        let earliest = Timestamp::now().saturating_add(std::time::Duration::from_secs(55));
        assert!(
            h2.answer_not_before.is_some_and(|at| at >= earliest),
            "60 s default wait"
        );
        // The next code inside the wait is refused without consuming the step.
        let error = service
            .submit(h2.clone(), Answer::Code(Zeroizing::new("222222".into())))
            .await
            .unwrap_err();
        assert!(matches!(error, Error::RateLimited { .. }), "{error}");
        assert!(matches!(
            service.current().await.unwrap().unwrap().state,
            RegistrationState::ChallengePending { .. }
        ));
        assert!(
            transport.responses.lock().unwrap().is_empty(),
            "nothing sent"
        );
    }

    /// An attempt saved as blocked by `stale` before the fix resends too, and
    /// still honours a saved wait; other server refusals stay blocked.
    #[tokio::test]
    async fn saved_stale_block_resends_but_other_blocks_do_not() {
        let (_d, s) = store();
        let profile = ProtocolProfile::android_reference().unwrap();
        let transport = Arc::new(ComparisonTransport {
            responses: std::sync::Mutex::new(std::collections::VecDeque::from([
                ("/v2/code", br#"{"status":"sent"}"#.as_slice()),
                ("/v2/code", br#"{"status":"sent"}"#.as_slice()),
            ])),
        });
        let service = RegistrationService::new(
            s.clone(),
            &profile,
            Arc::new(ComparisonHttpFlow::new(
                Arc::new(ComparisonRequests),
                transport.clone(),
            )),
        );
        let RegistrationStep::Action(RequiredAction::EnterCode(_)) =
            service.begin(phone(), ChallengeMethod::Sms).await.unwrap()
        else {
            panic!("expected code")
        };
        async fn block(
            service: &RegistrationService,
            s: &Arc<ProtocolStore>,
            reason: wire::RegistrationResponseReason,
            not_before: Option<Timestamp>,
        ) {
            let mut saved = service.current().await.unwrap().unwrap();
            saved.state = RegistrationState::Blocked {
                reason: BlockedReason::ServerRefused {
                    code: None,
                    http_status: Some(200),
                    reason: Some(reason),
                    not_before,
                },
            };
            s.write(|tx| RegistrationService::save_current(tx, &saved))
                .unwrap();
        }
        block(
            &service,
            &s,
            wire::RegistrationResponseReason::Blocked,
            None,
        )
        .await;
        let error = service
            .resend_code(phone(), ChallengeMethod::Sms)
            .await
            .unwrap_err();
        assert!(
            error.to_string().contains("expired-code refusal"),
            "{error}"
        );
        let later = Timestamp::now().saturating_add(std::time::Duration::from_secs(3600));
        block(
            &service,
            &s,
            wire::RegistrationResponseReason::Stale,
            Some(later),
        )
        .await;
        let error = service
            .resend_code(phone(), ChallengeMethod::Sms)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("wait"), "{error}");
        block(&service, &s, wire::RegistrationResponseReason::Stale, None).await;
        let RegistrationStep::Action(RequiredAction::EnterCode(_)) = service
            .resend_code(phone(), ChallengeMethod::Sms)
            .await
            .unwrap()
        else {
            panic!("a saved expired-code block resends")
        };
        assert_eq!(transport.responses.lock().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn register_challenge_runs_the_email_second_factor_to_completion() {
        let (_d, s) = store();
        let profile = ProtocolProfile::android_reference().unwrap();
        let transport = Arc::new(ComparisonTransport {
            responses: std::sync::Mutex::new(std::collections::VecDeque::from([
                ("/v2/code", br#"{"status":"sent","sms_wait":10}"#.as_slice()),
                (
                    "/v2/register",
                    br#"{"status":"fail","reason":"challenge","second_factor_methods":["email_otp","wa_old"],"is_device_trusted":false}"#.as_slice(),
                ),
                (
                    "/v2/code",
                    br#"{"status":"sent","email_otp_wait":5,"retry_after":900,"email_otp_length":6,"email":"a***@example.com"}"#.as_slice(),
                ),
                (
                    "/v2/register",
                    br#"{"status":"ok","login":"15551234567","edge_routing_info":"AQ=="}"#.as_slice(),
                ),
            ])),
        });
        let service = RegistrationService::new(
            s.clone(),
            &profile,
            Arc::new(ComparisonHttpFlow::new(
                Arc::new(ComparisonRequests),
                transport.clone(),
            )),
        );
        let RegistrationStep::Action(RequiredAction::EnterCode(h)) =
            service.begin(phone(), ChallengeMethod::Sms).await.unwrap()
        else {
            panic!("expected code")
        };
        let RegistrationStep::Action(RequiredAction::RequestEmailCode(h2)) = service
            .submit(h, Answer::Code(Zeroizing::new("123456".into())))
            .await
            .unwrap()
        else {
            panic!("expected the email challenge")
        };
        assert!(matches!(
            service.current().await.unwrap().unwrap().state,
            RegistrationState::EmailChallengeRequired { .. }
        ));
        // A wrong answer kind is rejected without consuming anything.
        assert!(
            service
                .submit(h2.clone(), Answer::EmailCode(Zeroizing::new("1".into())))
                .await
                .is_err()
        );
        let RegistrationStep::Action(RequiredAction::EnterEmailCode(h3)) =
            service.request_email_code(h2, None).await.unwrap()
        else {
            panic!("expected email code entry")
        };
        // The email wait applies to re-requesting, not to entering the code.
        // The longer of the two server waits is kept, not the first present.
        let earliest = Timestamp::now().saturating_add(std::time::Duration::from_secs(800));
        assert!(h3.retry_after.is_some_and(|at| at >= earliest));
        assert!(
            service
                .submit(h3.clone(), Answer::Pin(Zeroizing::new("1234".into())))
                .await
                .is_err(),
            "a PIN is not an email code"
        );
        assert_eq!(
            service
                .submit(h3, Answer::EmailCode(Zeroizing::new("654321".into())))
                .await
                .unwrap(),
            RegistrationStep::Completed
        );
        assert!(transport.responses.lock().unwrap().is_empty());
        let saved = service.current().await.unwrap().unwrap();
        assert!(matches!(saved.state, RegistrationState::Registered));
    }

    #[tokio::test]
    async fn saved_challenge_refusal_resumes_into_the_email_step_once() {
        let (_d, s) = store();
        let profile = ProtocolProfile::android_reference().unwrap();
        let transport = Arc::new(ComparisonTransport {
            responses: std::sync::Mutex::new(std::collections::VecDeque::from([
                ("/v2/code", br#"{"status":"sent"}"#.as_slice()),
                // A response written before this branch existed: no methods listed
                // and, for this test, a refusal we then treat as `challenge` blocked.
                (
                    "/v2/register",
                    br#"{"status":"fail","reason":"challenge","second_factor_methods":["wa_old"]}"#
                        .as_slice(),
                ),
                (
                    "/v2/code",
                    br#"{"status":"fail","reason":"too_many_guesses","retry_after":60}"#.as_slice(),
                ),
            ])),
        });
        let service = RegistrationService::new(
            s.clone(),
            &profile,
            Arc::new(ComparisonHttpFlow::new(
                Arc::new(ComparisonRequests),
                transport.clone(),
            )),
        );
        let RegistrationStep::Action(RequiredAction::EnterCode(h)) =
            service.begin(phone(), ChallengeMethod::Sms).await.unwrap()
        else {
            panic!("expected code")
        };
        // Only wa_old offered: stays blocked with the challenge refusal.
        let RegistrationStep::Blocked(BlockedReason::ServerRefused {
            reason: Some(wire::RegistrationResponseReason::Challenge),
            ..
        }) = service
            .submit(h, Answer::Code(Zeroizing::new("123456".into())))
            .await
            .unwrap()
        else {
            panic!("expected a challenge refusal")
        };
        // Operator recovery moves it to the email step without any request.
        let RegistrationStep::Action(RequiredAction::RequestEmailCode(h2)) =
            service.resume_email_challenge().await.unwrap()
        else {
            panic!("expected the email challenge")
        };
        assert!(service.resume_email_challenge().await.is_err(), "not twice");
        // An address is never sent: the server emails the one on file.
        assert!(
            service
                .request_email_code(h2.clone(), Some("a@example.com".into()))
                .await
                .is_err()
        );
        // A refused email request cools down and keeps the attempt.
        assert!(matches!(
            service.request_email_code(h2, None).await.unwrap(),
            RegistrationStep::Action(RequiredAction::WaitForCooldown { .. })
        ));
        assert!(transport.responses.lock().unwrap().is_empty());
    }

    /// A refusal without `retry_after` keeps the longest method wait it
    /// carried: the resumed email step enforces it before any request, and a
    /// refusal with no wait serialises exactly as before (no `not_before`).
    #[tokio::test]
    async fn a_blocked_refusal_keeps_its_method_wait() {
        let (_d, s) = store();
        let profile = ProtocolProfile::android_reference().unwrap();
        let transport = Arc::new(ComparisonTransport {
            responses: std::sync::Mutex::new(std::collections::VecDeque::from([
                ("/v2/code", br#"{"status":"sent"}"#.as_slice()),
                (
                    "/v2/register",
                    br#"{"status":"fail","reason":"challenge","second_factor_methods":["wa_old"],"email_otp_wait":900,"sms_wait":30}"#
                        .as_slice(),
                ),
            ])),
        });
        let service = RegistrationService::new(
            s.clone(),
            &profile,
            Arc::new(ComparisonHttpFlow::new(
                Arc::new(ComparisonRequests),
                transport.clone(),
            )),
        );
        let RegistrationStep::Action(RequiredAction::EnterCode(h)) =
            service.begin(phone(), ChallengeMethod::Sms).await.unwrap()
        else {
            panic!("expected code")
        };
        let before = Timestamp::now();
        service
            .submit(h, Answer::Code(Zeroizing::new("123456".into())))
            .await
            .unwrap();
        let saved = service.current().await.unwrap().unwrap().state;
        let not_before = saved.refusal_not_before().expect("the wait is kept");
        assert!(not_before >= before.saturating_add(std::time::Duration::from_secs(900)));
        let RegistrationStep::Action(RequiredAction::RequestEmailCode(h2)) =
            service.resume_email_challenge().await.unwrap()
        else {
            panic!("expected the email challenge")
        };
        assert_eq!(h2.retry_after, Some(not_before));
        assert!(matches!(
            service.request_email_code(h2, None).await,
            Err(Error::RateLimited { .. })
        ));
        assert!(
            transport.responses.lock().unwrap().is_empty(),
            "nothing sent"
        );
        // Older records and wait-less refusals keep their exact JSON shape.
        let plain = BlockedReason::ServerRefused {
            code: None,
            http_status: Some(200),
            reason: None,
            not_before: None,
        };
        let json = serde_json::to_value(&plain).unwrap();
        assert!(json["ServerRefused"].get("not_before").is_none(), "{json}");
        assert_eq!(
            serde_json::from_value::<BlockedReason>(json).unwrap(),
            plain
        );
    }

    /// A shorter method-specific wait never shortens a longer generic one,
    /// and a `challenge` refusal keeps its `email_otp_wait`.
    #[tokio::test]
    async fn server_waits_keep_the_longest_value() {
        let (_d, s) = store();
        let profile = ProtocolProfile::android_reference().unwrap();
        let transport = Arc::new(ComparisonTransport {
            responses: std::sync::Mutex::new(std::collections::VecDeque::from([
                (
                    "/v2/code",
                    br#"{"status":"sent","sms_wait":30,"retry_after":600}"#.as_slice(),
                ),
                (
                    "/v2/register",
                    br#"{"status":"fail","reason":"challenge","second_factor_methods":["email_otp"],"email_otp_wait":900}"#
                        .as_slice(),
                ),
            ])),
        });
        let service = RegistrationService::new(
            s.clone(),
            &profile,
            Arc::new(ComparisonHttpFlow::new(
                Arc::new(ComparisonRequests),
                transport.clone(),
            )),
        );
        let before = Timestamp::now();
        let RegistrationStep::Action(RequiredAction::EnterCode(h)) =
            service.begin(phone(), ChallengeMethod::Sms).await.unwrap()
        else {
            panic!("expected code")
        };
        let at_least = |until: Option<Timestamp>, seconds: u64| {
            until.is_some_and(|until| {
                until >= before.saturating_add(std::time::Duration::from_secs(seconds))
            })
        };
        assert!(at_least(h.retry_after, 600), "{:?}", h.retry_after);
        service
            .submit(h, Answer::Code(Zeroizing::new("123456".into())))
            .await
            .unwrap();
        let saved = service.current().await.unwrap().unwrap();
        let RegistrationState::EmailChallengeRequired { handle } = saved.state else {
            panic!("expected the email challenge, got {:?}", saved.state)
        };
        assert!(
            at_least(handle.retry_after, 900),
            "{:?}",
            handle.retry_after
        );
        assert_eq!(
            longest([None, Some(std::time::Duration::from_secs(5)), None]),
            Some(std::time::Duration::from_secs(5))
        );
        assert_eq!(longest::<2>([None, None]), None);
    }

    /// Each step answers with a body, or fails after the request may have been
    /// written (an ordinary I/O error, not "not sent").
    #[derive(Debug)]
    struct LostAfterWriteTransport {
        steps: MaybeSentSteps,
    }

    impl wire::RegistrationTransport for LostAfterWriteTransport {
        fn send<'a>(
            &'a self,
            request: wire::EncodedRegistrationRequest,
        ) -> wire::RegistrationTransportFuture<'a> {
            Box::pin(async move {
                let (path, body) = self.steps.lock().unwrap().pop_front().unwrap();
                assert!(request.url().ends_with(path));
                match body {
                    Some(body) => {
                        wire::RegistrationHttpResponse::new(200, Zeroizing::new(body.to_vec()))
                    }
                    None => Err(Error::Io(std::io::Error::other("reset after writing"))),
                }
            })
        }
    }

    /// An email-code request that may have reached the server (lost after
    /// writing) is never resumed; only an answer that arrived unreadable is.
    #[tokio::test]
    async fn an_email_code_request_lost_after_writing_is_not_resumed() {
        let (_d, s) = store();
        let profile = ProtocolProfile::android_reference().unwrap();
        let transport = Arc::new(LostAfterWriteTransport {
            steps: std::sync::Mutex::new(std::collections::VecDeque::from([
                ("/v2/code", Some(br#"{"status":"sent"}"#.as_slice())),
                (
                    "/v2/register",
                    Some(
                        br#"{"status":"fail","reason":"challenge","second_factor_methods":["email_otp"]}"#
                            .as_slice(),
                    ),
                ),
                ("/v2/code", None),
            ])),
        });
        let service = RegistrationService::new(
            s.clone(),
            &profile,
            Arc::new(ComparisonHttpFlow::new(
                Arc::new(ComparisonRequests),
                transport.clone(),
            )),
        );
        let RegistrationStep::Action(RequiredAction::EnterCode(h)) =
            service.begin(phone(), ChallengeMethod::Sms).await.unwrap()
        else {
            panic!("expected code")
        };
        let RegistrationStep::Action(RequiredAction::RequestEmailCode(email)) = service
            .submit(h, Answer::Code(Zeroizing::new("123456".into())))
            .await
            .unwrap()
        else {
            panic!("expected the email challenge")
        };
        assert!(service.request_email_code(email, None).await.is_err());
        let saved = service.current().await.unwrap().unwrap();
        assert!(matches!(
            saved.state,
            RegistrationState::RegistrationOutcomeUnknown { .. }
        ));
        let error = service.resume_email_challenge().await.unwrap_err();
        assert!(
            error.to_string().contains("only a `challenge` refusal"),
            "{error}"
        );
        assert!(transport.steps.lock().unwrap().is_empty());
    }

    /// Each email-challenge recovery is allowed once per attempt: a lost
    /// email-code request can be resumed once, and a second loss stays
    /// uncertain instead of being re-sent again.
    #[tokio::test]
    async fn lost_email_code_requests_resume_only_once() {
        let (_d, s) = store();
        let profile = ProtocolProfile::android_reference().unwrap();
        let transport = Arc::new(StatusTransport {
            steps: std::sync::Mutex::new(std::collections::VecDeque::from([
                ("/v2/code", 200, br#"{"status":"sent"}"#.as_slice()),
                (
                    "/v2/register",
                    200,
                    br#"{"status":"fail","reason":"challenge","second_factor_methods":["wa_old"]}"#
                        .as_slice(),
                ),
                // Two email-code requests refused with an HTTP error status
                // and a body that cannot be read (the observed 404 shape).
                ("/v2/code", 404, b"not json".as_slice()),
                ("/v2/code", 404, b"not json".as_slice()),
            ])),
        });
        let service = RegistrationService::new(
            s.clone(),
            &profile,
            Arc::new(ComparisonHttpFlow::new(
                Arc::new(ComparisonRequests),
                transport.clone(),
            )),
        );
        let RegistrationStep::Action(RequiredAction::EnterCode(h)) =
            service.begin(phone(), ChallengeMethod::Sms).await.unwrap()
        else {
            panic!("expected code")
        };
        service
            .submit(h, Answer::Code(Zeroizing::new("123456".into())))
            .await
            .unwrap();
        for round in 0..2 {
            let RegistrationStep::Action(RequiredAction::RequestEmailCode(handle)) =
                service.resume_email_challenge().await.unwrap()
            else {
                panic!("expected the email challenge in round {round}")
            };
            let _ = service.request_email_code(handle, None).await;
            let saved = service.current().await.unwrap().unwrap();
            assert!(
                matches!(
                    saved.state,
                    RegistrationState::RegistrationOutcomeUnknown { .. }
                ),
                "round {round}: {:?}",
                saved.state
            );
        }
        let error = service.resume_email_challenge().await.unwrap_err();
        assert!(error.to_string().contains("already used"), "{error}");
        assert!(transport.steps.lock().unwrap().is_empty());
    }

    /// Each step answers with an HTTP status and a body.
    #[derive(Debug)]
    struct StatusTransport {
        steps: std::sync::Mutex<std::collections::VecDeque<(&'static str, u16, &'static [u8])>>,
    }

    impl wire::RegistrationTransport for StatusTransport {
        fn send<'a>(
            &'a self,
            request: wire::EncodedRegistrationRequest,
        ) -> wire::RegistrationTransportFuture<'a> {
            Box::pin(async move {
                let (path, status, body) = self.steps.lock().unwrap().pop_front().unwrap();
                assert!(request.url().ends_with(path), "{}", request.url());
                wire::RegistrationHttpResponse::new(status, Zeroizing::new(body.to_vec()))
            })
        }
    }

    /// An email-code request answered with a body we cannot read under HTTP
    /// 2xx, 5xx, 408 or 429 may have sent an email (or carries a wait we
    /// cannot read): it stays unknown and is never resumed, so the request
    /// cannot be sent again without the wait it may have set (M1 and its
    /// follow-up). Only a definite 4xx refusal is resumable (the test above).
    #[tokio::test]
    async fn an_unreadable_uncertain_answer_to_an_email_request_is_not_resumed() {
        for (status, body) in [
            // A negative wait fails strict parsing under HTTP 200.
            (200, br#"{"status":"sent","email_otp_wait":-1}"#.as_slice()),
            (500, b"".as_slice()),
            (502, b"<html>bad gateway</html>".as_slice()),
            (504, b"<html>gateway timeout</html>".as_slice()),
            (408, b"timeout".as_slice()),
            (429, b"slow down".as_slice()),
        ] {
            let (_d, s) = store();
            let profile = ProtocolProfile::android_reference().unwrap();
            let transport = Arc::new(StatusTransport {
                steps: std::sync::Mutex::new(std::collections::VecDeque::from([
                    ("/v2/code", 200, br#"{"status":"sent"}"#.as_slice()),
                    (
                        "/v2/register",
                        200,
                        br#"{"status":"fail","reason":"challenge","second_factor_methods":["wa_old"]}"#
                            .as_slice(),
                    ),
                    ("/v2/code", status, body),
                ])),
            });
            let service = RegistrationService::new(
                s.clone(),
                &profile,
                Arc::new(ComparisonHttpFlow::new(
                    Arc::new(ComparisonRequests),
                    transport.clone(),
                )),
            );
            let RegistrationStep::Action(RequiredAction::EnterCode(h)) =
                service.begin(phone(), ChallengeMethod::Sms).await.unwrap()
            else {
                panic!("expected code")
            };
            service
                .submit(h, Answer::Code(Zeroizing::new("123456".into())))
                .await
                .unwrap();
            let RegistrationStep::Action(RequiredAction::RequestEmailCode(handle)) =
                service.resume_email_challenge().await.unwrap()
            else {
                panic!("expected the email challenge")
            };
            assert!(service.request_email_code(handle, None).await.is_err());
            assert!(
                matches!(
                    service.current().await.unwrap().unwrap().state,
                    RegistrationState::RegistrationOutcomeUnknown { .. }
                ),
                "HTTP {status}"
            );
            let error = service.resume_email_challenge().await.unwrap_err();
            assert!(
                error.to_string().contains("can resume"),
                "HTTP {status}: {error}"
            );
            assert!(transport.steps.lock().unwrap().is_empty());
        }
    }

    /// The default flow never produces a challenge; answering one that
    /// another flow left under the same profile id sends nothing and keeps
    /// the challenge, instead of writing a local block over it.
    #[tokio::test]
    async fn the_default_flow_restores_a_challenge_it_did_not_create() {
        let (_d, s) = store();
        let profile = ProtocolProfile::android_reference().unwrap();
        let real = RegistrationService::new(
            s.clone(),
            &profile,
            Arc::new(ComparisonHttpFlow::new(
                Arc::new(ComparisonRequests),
                Arc::new(ComparisonTransport {
                    responses: std::sync::Mutex::new(std::collections::VecDeque::from([(
                        "/v2/code",
                        br#"{"status":"sent"}"#.as_slice(),
                    )])),
                }),
            )),
        );
        let RegistrationStep::Action(RequiredAction::EnterCode(handle)) =
            real.begin(phone(), ChallengeMethod::Sms).await.unwrap()
        else {
            panic!("expected code")
        };
        let saved = real.current().await.unwrap().unwrap();
        let default = RegistrationService::new(
            s.clone(),
            &profile,
            Arc::new(UnsupportedFlow {
                missing: vec!["registration".into()],
            }),
        );
        let error = default
            .submit(handle, Answer::Code(Zeroizing::new("123456".into())))
            .await
            .unwrap_err();
        assert!(matches!(error, Error::RequestNotSent(_)), "{error}");
        assert_eq!(default.current().await.unwrap().unwrap(), saved);
    }

    #[tokio::test]
    async fn security_code_requires_successful_http_and_fail_status() {
        #[derive(Debug)]
        struct Response(u16, &'static [u8]);
        impl wire::RegistrationTransport for Response {
            fn send<'a>(
                &'a self,
                _: wire::EncodedRegistrationRequest,
            ) -> wire::RegistrationTransportFuture<'a> {
                Box::pin(async move {
                    wire::RegistrationHttpResponse::new(self.0, Zeroizing::new(self.1.to_vec()))
                })
            }
        }
        for (status, body) in [
            (
                403,
                br#"{"status":"fail","reason":"security_code"}"#.as_slice(),
            ),
            (
                500,
                br#"{"status":"fail","reason":"security_code"}"#.as_slice(),
            ),
            (
                200,
                br#"{"status":"unknown","reason":"security_code"}"#.as_slice(),
            ),
        ] {
            let (_d, s) = store();
            let profile = ProtocolProfile::android_reference().unwrap();
            let initial = RegistrationService::new(s.clone(), &profile, Arc::new(ScriptedFlow));
            let RegistrationStep::Action(RequiredAction::EnterCode(h)) =
                initial.begin(phone(), ChallengeMethod::Sms).await.unwrap()
            else {
                panic!("expected code")
            };
            let svc = RegistrationService::new(
                s,
                &profile,
                Arc::new(ComparisonHttpFlow::new(
                    Arc::new(ComparisonRequests),
                    Arc::new(Response(status, body)),
                )),
            );
            assert!(matches!(
                svc.submit(h, Answer::Code(Zeroizing::new("123456".into())))
                    .await
                    .unwrap(),
                RegistrationStep::Blocked(_)
            ));
        }
    }

    #[tokio::test]
    async fn pin_wait_is_enforced_without_consuming_challenge() {
        let (_d, s) = store();
        let profile = ProtocolProfile::android_reference().unwrap();
        let transport = Arc::new(ComparisonTransport {
            responses: std::sync::Mutex::new(std::collections::VecDeque::from([
                ("/v2/code", br#"{"status":"sent","sms_wait":60}"#.as_slice()),
                (
                    "/v2/register",
                    br#"{"status":"fail","reason":"security_code","retry_after":60}"#.as_slice(),
                ),
            ])),
        });
        let svc = RegistrationService::new(
            s.clone(),
            &profile,
            Arc::new(ComparisonHttpFlow::new(
                Arc::new(ComparisonRequests),
                transport.clone(),
            )),
        );
        let RegistrationStep::Action(RequiredAction::EnterCode(h)) =
            svc.begin(phone(), ChallengeMethod::Sms).await.unwrap()
        else {
            panic!("expected code")
        };
        let RegistrationStep::Action(RequiredAction::EnterPin(h)) = svc
            .submit(h, Answer::Code(Zeroizing::new("123456".into())))
            .await
            .unwrap()
        else {
            panic!("expected PIN")
        };
        assert!(matches!(
            svc.submit(h, Answer::Pin(Zeroizing::new("6157".into())))
                .await,
            Err(Error::RateLimited { .. })
        ));
        assert!(matches!(
            svc.current().await.unwrap().unwrap().state,
            RegistrationState::NeedsUserAction { .. }
        ));
        assert!(transport.responses.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn comparison_http_flow_drives_happy_path_and_commits_credentials() {
        let (_d, s) = store();
        let profile = ProtocolProfile::android_reference().unwrap();
        let flow = ComparisonHttpFlow::new(
            Arc::new(ComparisonRequests),
            Arc::new(ComparisonTransport {
                responses: std::sync::Mutex::new(std::collections::VecDeque::from([
                    (
                        "/v2/code",
                        br#"{"status":"sent","sms_wait":"30"}"#.as_slice(),
                    ),
                    (
                        "/v2/register",
                        br#"{"status":"fail","reason":"security_code","security_code_set":true}"#
                            .as_slice(),
                    ),
                    (
                        "/v2/security",
                        br#"{"status":"ok","login":"login-secret"}"#.as_slice(),
                    ),
                ])),
            }),
        );
        let svc = RegistrationService::new(s.clone(), &profile, Arc::new(flow));

        let step = svc.begin(phone(), ChallengeMethod::Sms).await.unwrap();
        let RegistrationStep::Action(RequiredAction::EnterCode(handle)) = step else {
            panic!("expected code challenge, got {step:?}");
        };
        assert!(handle.retry_after.is_some());
        let pin_step = svc
            .submit(handle, Answer::Code(Zeroizing::new("123456".into())))
            .await
            .unwrap();
        let RegistrationStep::Action(RequiredAction::EnterPin(pin_handle)) = pin_step else {
            panic!("expected account PIN challenge, got {pin_step:?}");
        };
        let done = svc
            .submit(pin_handle, Answer::Pin(Zeroizing::new("6157".into())))
            .await
            .unwrap();
        assert_eq!(done, RegistrationStep::Completed);
        let account = s.read(AccountRecord::load).unwrap().unwrap();
        assert_eq!(account.registration, RegistrationStatus::Registered);
        assert_eq!(account.credentials.len(), 1);
        assert_eq!(account.credentials[0].name, "login");
        assert_eq!(
            account.credentials[0].source_step.as_deref(),
            Some("/v2/security")
        );
        assert_eq!(account.credentials[0].value.as_slice(), b"login-secret");
        let raw = std::fs::read(s.path()).unwrap();
        assert!(
            !raw.windows(b"login-secret".len())
                .any(|window| window == b"login-secret")
        );
    }

    /// A test flow that hands out a challenge and accepts one specific code,
    /// to exercise handle binding and the verifying path.
    #[derive(Debug)]
    struct ScriptedFlow;

    impl RegistrationFlow for ScriptedFlow {
        fn prepare(&self, _tx: &mut WriteTx<'_>, _attempt: &RegistrationAttempt) -> Result<()> {
            Ok(())
        }

        fn begin<'a>(&'a self, attempt: &'a RegistrationAttempt) -> RegistrationFuture<'a> {
            Box::pin(async move {
                Ok(RegistrationState::ChallengePending {
                    handle: ChallengeHandle {
                        answer_not_before: None,
                        attempt: attempt.id,
                        generation: attempt.generation,
                        step: 1,
                        method: attempt.method.clone(),
                        expires_at: None,
                        retry_after: None,
                    },
                }
                .into())
            })
        }

        fn submit<'a>(
            &'a self,
            attempt: &'a RegistrationAttempt,
            handle: &'a ChallengeHandle,
            answer: &'a Answer,
        ) -> RegistrationFuture<'a> {
            Box::pin(async move {
                match answer {
                    Answer::Code(c) if c.as_str() == "928374" && handle.step == 1 => {
                        Ok(RegistrationState::NeedsUserAction {
                            handle: ChallengeHandle {
                                answer_not_before: None,
                                attempt: attempt.id,
                                generation: attempt.generation,
                                step: 2,
                                method: ChallengeMethod::Other("pin".into()),
                                expires_at: None,
                                retry_after: None,
                            },
                        }
                        .into())
                    }
                    Answer::Pin(p) if p.as_str() == "6157" && handle.step == 2 => {
                        let completion = RegistrationCompletion::new()
                            .with_own_jid(Jid::parse("15551234567@s.whatsapp.net")?)
                            .with_credential(ObservedCredential {
                                name: "login".into(),
                                value: Zeroizing::new(b"server-login-secret".to_vec()),
                                observed_at: Timestamp::now(),
                                source_step: Some("register".into()),
                            })?;
                        Ok(RegistrationFlowOutput::registered(completion))
                    }
                    _ => Ok(RegistrationState::Blocked {
                        reason: BlockedReason::ServerRefused {
                            code: Some(403),
                            http_status: Some(403),
                            reason: None,
                            not_before: None,
                        },
                    }
                    .into()),
                }
            })
        }
    }

    #[derive(Debug)]
    struct PausingFlow {
        entered: Arc<tokio::sync::Notify>,
        release: Arc<tokio::sync::Notify>,
    }

    impl RegistrationFlow for PausingFlow {
        fn prepare(&self, tx: &mut WriteTx<'_>, attempt: &RegistrationAttempt) -> Result<()> {
            AccountRecord {
                generation: attempt.generation,
                phone: Some(attempt.phone.clone()),
                ..AccountRecord::default()
            }
            .save(tx)
        }

        fn begin<'a>(&'a self, _attempt: &'a RegistrationAttempt) -> RegistrationFuture<'a> {
            Box::pin(async move {
                self.entered.notify_one();
                self.release.notified().await;
                Ok(RegistrationState::Blocked {
                    reason: BlockedReason::UnsupportedStep {
                        description: "paused test flow".into(),
                    },
                }
                .into())
            })
        }

        fn submit<'a>(
            &'a self,
            _attempt: &'a RegistrationAttempt,
            _handle: &'a ChallengeHandle,
            _answer: &'a Answer,
        ) -> RegistrationFuture<'a> {
            Box::pin(async {
                Ok(RegistrationState::Blocked {
                    reason: BlockedReason::UnsupportedStep {
                        description: "submit unused in paused test flow".into(),
                    },
                }
                .into())
            })
        }
    }

    #[tokio::test]
    async fn driver_wait_does_not_hold_the_store_transaction() {
        let (_d, s) = store();
        let profile = ProtocolProfile::android_reference().unwrap();
        let entered = Arc::new(tokio::sync::Notify::new());
        let release = Arc::new(tokio::sync::Notify::new());
        let svc = RegistrationService::new(
            s.clone(),
            &profile,
            Arc::new(PausingFlow {
                entered: entered.clone(),
                release: release.clone(),
            }),
        );

        let begin = tokio::spawn({
            let svc = svc.clone();
            async move { svc.begin(phone(), ChallengeMethod::Sms).await }
        });
        entered.notified().await;

        let current = svc.current().await.unwrap().unwrap();
        assert_eq!(current.state, RegistrationState::Preparing);
        let prepared = s.read(AccountRecord::load).unwrap().unwrap();
        assert_eq!(prepared.generation, current.generation);
        assert_eq!(prepared.phone, Some(phone()));
        tokio::time::timeout(
            std::time::Duration::from_secs(1),
            s.run_blocking(|store| store.write(|_| Ok(()))),
        )
        .await
        .expect("registration driver held the write transaction")
        .unwrap();

        release.notify_one();
        assert!(matches!(
            begin.await.unwrap().unwrap(),
            RegistrationStep::Blocked(BlockedReason::UnsupportedStep { .. })
        ));
    }

    #[derive(Debug)]
    struct FailingFlow;

    impl RegistrationFlow for FailingFlow {
        fn prepare(&self, _tx: &mut WriteTx<'_>, _attempt: &RegistrationAttempt) -> Result<()> {
            Ok(())
        }

        fn begin<'a>(&'a self, _attempt: &'a RegistrationAttempt) -> RegistrationFuture<'a> {
            Box::pin(async { Err(Error::Io(std::io::Error::other("synthetic failure"))) })
        }

        fn submit<'a>(
            &'a self,
            _attempt: &'a RegistrationAttempt,
            _handle: &'a ChallengeHandle,
            _answer: &'a Answer,
        ) -> RegistrationFuture<'a> {
            Box::pin(async { Err(Error::Io(std::io::Error::other("synthetic failure"))) })
        }
    }

    #[tokio::test]
    async fn driver_error_is_persisted_as_outcome_unknown() {
        let (_d, s) = store();
        let profile = ProtocolProfile::android_reference().unwrap();
        let svc = RegistrationService::new(s, &profile, Arc::new(FailingFlow));

        assert!(svc.begin(phone(), ChallengeMethod::Sms).await.is_err());
        let current = svc.current().await.unwrap().unwrap();
        assert_eq!(
            current.state,
            RegistrationState::RegistrationOutcomeUnknown { step: 0 }
        );
        assert_eq!(current.history_len(), 2);
    }

    #[derive(Debug)]
    struct ReconcilingFlow;

    impl RegistrationFlow for ReconcilingFlow {
        fn prepare(&self, _tx: &mut WriteTx<'_>, _attempt: &RegistrationAttempt) -> Result<()> {
            Ok(())
        }

        fn begin<'a>(&'a self, _attempt: &'a RegistrationAttempt) -> RegistrationFuture<'a> {
            Box::pin(async { Err(Error::Io(std::io::Error::other("lost response"))) })
        }

        fn submit<'a>(
            &'a self,
            _attempt: &'a RegistrationAttempt,
            _handle: &'a ChallengeHandle,
            _answer: &'a Answer,
        ) -> RegistrationFuture<'a> {
            Box::pin(async { Err(Error::invalid("test", "submit is unused")) })
        }

        fn reconcile<'a>(&'a self, attempt: &'a RegistrationAttempt) -> RegistrationFuture<'a> {
            Box::pin(async move {
                Ok(RegistrationState::ChallengePending {
                    handle: ChallengeHandle {
                        answer_not_before: None,
                        attempt: attempt.id,
                        generation: attempt.generation,
                        step: 1,
                        method: attempt.method.clone(),
                        expires_at: None,
                        retry_after: None,
                    },
                }
                .into())
            })
        }
    }

    #[tokio::test]
    async fn outcome_unknown_can_be_reconciled_without_repeating_mutation() {
        let (_d, s) = store();
        let profile = ProtocolProfile::android_reference().unwrap();
        let svc = RegistrationService::new(s, &profile, Arc::new(ReconcilingFlow));

        assert!(svc.begin(phone(), ChallengeMethod::Sms).await.is_err());
        assert!(matches!(
            svc.current().await.unwrap().unwrap().state,
            RegistrationState::RegistrationOutcomeUnknown { step: 0 }
        ));
        let step = svc.reconcile().await.unwrap();
        assert!(matches!(
            step,
            RegistrationStep::Action(RequiredAction::EnterCode(_))
        ));
        assert!(matches!(
            svc.current().await.unwrap().unwrap().state,
            RegistrationState::ChallengePending { .. }
        ));
        assert!(svc.reconcile().await.is_err());
    }

    #[derive(Debug)]
    struct FailingPreparationFlow;

    impl RegistrationFlow for FailingPreparationFlow {
        fn prepare(&self, tx: &mut WriteTx<'_>, attempt: &RegistrationAttempt) -> Result<()> {
            AccountRecord {
                generation: attempt.generation,
                phone: Some(attempt.phone.clone()),
                ..AccountRecord::default()
            }
            .save(tx)?;
            Err(Error::Io(std::io::Error::other(
                "synthetic preparation failure",
            )))
        }

        fn begin<'a>(&'a self, _attempt: &'a RegistrationAttempt) -> RegistrationFuture<'a> {
            Box::pin(async {
                Ok(RegistrationState::Blocked {
                    reason: BlockedReason::UnsupportedStep {
                        description: "unreachable after failed preparation".into(),
                    },
                }
                .into())
            })
        }

        fn submit<'a>(
            &'a self,
            _attempt: &'a RegistrationAttempt,
            _handle: &'a ChallengeHandle,
            _answer: &'a Answer,
        ) -> RegistrationFuture<'a> {
            Box::pin(async {
                Ok(RegistrationState::Blocked {
                    reason: BlockedReason::UnsupportedStep {
                        description: "submit unused after failed preparation".into(),
                    },
                }
                .into())
            })
        }
    }

    #[tokio::test]
    async fn preparation_failure_rolls_back_attempt_and_prerequisites() {
        let (_d, s) = store();
        let profile = ProtocolProfile::android_reference().unwrap();
        let svc = RegistrationService::new(s.clone(), &profile, Arc::new(FailingPreparationFlow));

        assert!(svc.begin(phone(), ChallengeMethod::Sms).await.is_err());
        assert!(svc.current().await.unwrap().is_none());
        assert!(s.read(AccountRecord::load).unwrap().is_none());
    }

    #[tokio::test]
    async fn handles_are_bound_to_attempt_generation_and_step() {
        let (_d, s) = store();
        let profile = ProtocolProfile::android_reference().unwrap();
        let svc = RegistrationService::new(s.clone(), &profile, Arc::new(ScriptedFlow));
        let step = svc.begin(phone(), ChallengeMethod::Sms).await.unwrap();
        let RegistrationStep::Action(RequiredAction::EnterCode(h1)) = step else {
            panic!("expected code challenge, got {step:?}");
        };
        // A handle for a different attempt is rejected without consuming the step.
        for invalid in [
            Answer::Pin(Zeroizing::new("6157".into())),
            Answer::Code(Zeroizing::new("bad-code".into())),
        ] {
            assert!(svc.submit(h1.clone(), invalid).await.is_err());
            assert!(matches!(
                svc.current().await.unwrap().unwrap().state,
                RegistrationState::ChallengePending { .. }
            ));
        }
        let foreign = ChallengeHandle {
            attempt: AttemptId([9; 16]),
            ..h1.clone()
        };
        assert!(
            svc.submit(foreign, Answer::Code(Zeroizing::new("928374".into())))
                .await
                .is_err()
        );
        let step2 = svc
            .submit(h1.clone(), Answer::Code(Zeroizing::new("928374".into())))
            .await
            .unwrap();
        let RegistrationStep::Action(RequiredAction::EnterPin(h2)) = step2 else {
            panic!("expected pin step, got {step2:?}");
        };
        assert_eq!(h2.step, 2);
        assert!(
            svc.submit(h2.clone(), Answer::Code(Zeroizing::new("928374".into())))
                .await
                .is_err()
        );
        // The old step-1 handle can no longer be answered.
        assert!(
            svc.submit(h1, Answer::Code(Zeroizing::new("928374".into())))
                .await
                .is_err()
        );
        let done = svc
            .submit(h2, Answer::Pin(Zeroizing::new("6157".into())))
            .await
            .unwrap();
        assert_eq!(done, RegistrationStep::Completed);
        let account = s.read(AccountRecord::load).unwrap().unwrap();
        assert_eq!(account.registration, RegistrationStatus::Registered);
        assert_eq!(account.phone, Some(phone()));
        assert_eq!(
            account.own_jid.unwrap().to_string(),
            "15551234567@s.whatsapp.net"
        );
        assert_eq!(account.credentials.len(), 1);
        // Completed attempts are never restarted implicitly.
        let again = svc.begin(phone(), ChallengeMethod::Sms).await.unwrap();
        assert_eq!(again, RegistrationStep::Completed);
        let other = PhoneNumber::new(1, "5559876543").unwrap();
        assert!(svc.begin(other, ChallengeMethod::Sms).await.is_err());
        // The persisted record never contains the code or PIN.
        let raw = std::fs::read(s.path()).unwrap();
        assert!(!raw.windows(6).any(|w| w == b"928374"));
        assert!(
            !raw.windows(b"server-login-secret".len())
                .any(|window| window == b"server-login-secret")
        );
        let cur = svc.current().await.unwrap().unwrap();
        let json = serde_json::to_string(&cur).unwrap();
        assert!(!json.contains("928374") && !json.contains("6157"));
        assert!(!format!("{cur:?}").contains("5551234567"));
    }
}
