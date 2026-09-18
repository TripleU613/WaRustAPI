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
pub mod factory;
pub mod prerequisites;
pub mod request;
mod request_config;
pub mod token;
pub mod wire;

use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

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
    /// The protocol profile lacks required facts (ledger references listed).
    ProfileIncomplete {
        /// Missing ledger entries.
        missing: Vec<String>,
    },
    /// The server refused the flow with a sanitized code.
    ServerRefused {
        /// Sanitized code.
        code: Option<u32>,
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
}

impl fmt::Debug for Answer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Code(_) => f.write_str("Answer::Code(<redacted>)"),
            Self::Pin(_) => f.write_str("Answer::Pin(<redacted>)"),
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
                    S::Verifying { .. } | S::CoolingDown { .. } | S::Blocked { .. }
                )
                | (
                    S::NeedsUserAction { .. },
                    S::Verifying { .. } | S::Blocked { .. }
                )
                | (
                    S::Verifying { .. },
                    S::Registered
                        | S::NeedsUserAction { .. }
                        | S::ChallengePending { .. }
                        | S::RegistrationOutcomeUnknown { .. }
                        | S::CoolingDown { .. }
                        | S::Blocked { .. },
                )
                | (
                    S::RegistrationOutcomeUnknown { .. },
                    S::Registered | S::ChallengePending { .. } | S::Blocked { .. }
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Transition {
    at: Timestamp,
    from: String,
    to: String,
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

    /// Number of recorded transitions.
    pub fn history_len(&self) -> usize {
        self.history.len()
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
    /// Generate and persist profile prerequisites inside the same transaction
    /// as the `Preparing` transition. This method must be bounded and perform
    /// no network I/O. If it fails, neither the attempt nor its prerequisites
    /// are committed.
    fn prepare(&self, tx: &mut WriteTx<'_>, attempt: &RegistrationAttempt) -> Result<()>;

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

    fn begin<'a>(&'a self, _attempt: &'a RegistrationAttempt) -> RegistrationFuture<'a> {
        Box::pin(async move {
            Ok(RegistrationState::Blocked {
                reason: BlockedReason::ProfileIncomplete {
                    missing: self.missing.clone(),
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
        Box::pin(async move {
            Ok(RegistrationState::Blocked {
                reason: BlockedReason::ProfileIncomplete {
                    missing: self.missing.clone(),
                },
            }
            .into())
        })
    }
}

/// Supplies exact comparison-build requests and persists their prerequisites.
///
/// Implementations normally use [`wire::assemble_code_request`] and
/// [`wire::assemble_complete_request`]. Keeping this boundary separate lets
/// callers source device identifiers and key material from their durable
/// store without exposing them through the high-level service.
pub trait ComparisonRequestFactory: Send + Sync + fmt::Debug {
    /// Persist keys and identifiers needed by later request construction.
    fn prepare(&self, tx: &mut WriteTx<'_>, attempt: &RegistrationAttempt) -> Result<()>;

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
        Ok((status_code, response.parse_json()?))
    }

    fn blocked_response(
        status_code: u16,
        response: &wire::RegistrationResponse,
    ) -> RegistrationFlowOutput {
        if let Some(wait) = response.retry_after {
            return RegistrationState::CoolingDown {
                until: Timestamp::now().saturating_add(wait),
            }
            .into();
        }
        if response.status == wire::RegistrationResponseStatus::Fail || status_code >= 400 {
            return RegistrationState::Blocked {
                reason: BlockedReason::ServerRefused {
                    code: (status_code >= 400).then_some(u32::from(status_code)),
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
                    source_step: Some("/v2/register".into()),
                })?;
            }
        }
        Ok(completion)
    }
}

impl RegistrationFlow for ComparisonHttpFlow {
    fn prepare(&self, tx: &mut WriteTx<'_>, attempt: &RegistrationAttempt) -> Result<()> {
        self.requests.prepare(tx, attempt)
    }

    fn begin<'a>(&'a self, attempt: &'a RegistrationAttempt) -> RegistrationFuture<'a> {
        Box::pin(async move {
            let request = self.requests.code_request(attempt)?;
            let (status_code, response) = self.send(request).await?;
            if (200..300).contains(&status_code)
                && response.status == wire::RegistrationResponseStatus::Sent
            {
                let retry_wait = match attempt.method {
                    ChallengeMethod::Sms => response.sms_wait.or(response.retry_after),
                    ChallengeMethod::Voice => response.voice_wait.or(response.retry_after),
                    ChallengeMethod::Other(_) => response.retry_after,
                };
                return Ok(RegistrationState::ChallengePending {
                    handle: ChallengeHandle {
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

    fn submit<'a>(
        &'a self,
        attempt: &'a RegistrationAttempt,
        handle: &'a ChallengeHandle,
        answer: &'a Answer,
    ) -> RegistrationFuture<'a> {
        Box::pin(async move {
            let request = match answer {
                Answer::Code(code) => self.requests.complete_request(attempt, code)?,
                Answer::Pin(pin) => self.requests.security_request(attempt, pin)?,
            };
            let (status_code, mut response) = self.send(request).await?;
            if (200..300).contains(&status_code)
                && response.status == wire::RegistrationResponseStatus::Ok
            {
                return Ok(RegistrationFlowOutput::registered(Self::completion_from(
                    &mut response,
                )?));
            }
            if matches!(answer, Answer::Code(_))
                && response.reason == Some(wire::RegistrationResponseReason::SecurityCode)
            {
                return Ok(RegistrationState::NeedsUserAction {
                    handle: ChallengeHandle {
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
            Ok(Self::blocked_response(status_code, &response))
        })
    }
}

enum BeginPlan {
    Existing(RegistrationStep),
    Start(RegistrationAttempt),
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

    async fn finish_transition(
        &self,
        attempt_id: AttemptId,
        generation: AccountGeneration,
        expected_state: &'static str,
        output: RegistrationFlowOutput,
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
                            if let RegistrationState::CoolingDown { until } = &existing.state
                                && now >= *until
                            {
                                existing.transition(RegistrationState::Preparing, now)?;
                                Self::save_current(tx, &existing)?;
                                return Ok(BeginPlan::Start(existing));
                            }
                            return Ok(BeginPlan::Existing(step_for(&existing.state)));
                        }
                        if matches!(existing.state, RegistrationState::Registered) {
                            return Err(Error::invalid(
                                "phone",
                                "an account is already registered in this store; logout/reset first",
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
                    attempt.transition(RegistrationState::Preparing, now)?;
                    // Prerequisites and Preparing commit atomically before
                    // any request can be sent (docs/design/README.md section 7).
                    flow.prepare(tx, &attempt)?;
                    Self::save_current(tx, &attempt)?;
                    Ok(BeginPlan::Start(attempt))
                })
            })
            .await?;

        let attempt = match plan {
            BeginPlan::Existing(step) => return Ok(step),
            BeginPlan::Start(attempt) => attempt,
        };
        let result = self.flow.begin(&attempt).await;
        match result {
            Ok(next) => {
                self.finish_transition(attempt.id, attempt.generation, "preparing", next)
                    .await
            }
            Err(error) => {
                self.finish_transition(
                    attempt.id,
                    attempt.generation,
                    "preparing",
                    RegistrationState::RegistrationOutcomeUnknown { step: 0 }.into(),
                )
                .await?;
                Err(error)
            }
        }
    }

    /// Submit a typed answer for a specific challenge step.
    pub async fn submit(
        &self,
        handle: ChallengeHandle,
        answer: Answer,
    ) -> Result<RegistrationStep> {
        let (attempt, bound) = self
            .store
            .run_blocking(move |s| {
                s.write(|tx| {
                    let mut attempt = Self::load_current(tx)?.ok_or_else(|| {
                        Error::invalid("challenge", "no registration attempt in progress")
                    })?;
                    let bound = match &attempt.state {
                        RegistrationState::ChallengePending { handle: h }
                        | RegistrationState::NeedsUserAction { handle: h } => h.clone(),
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
                    attempt.transition(RegistrationState::Verifying { step: bound.step }, now)?;
                    Self::save_current(tx, &attempt)?;
                    Ok((attempt, bound))
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
                self.finish_transition(
                    attempt.id,
                    attempt.generation,
                    "verifying",
                    RegistrationState::RegistrationOutcomeUnknown { step: bound.step }.into(),
                )
                .await?;
                Err(error)
            }
        }
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
        self.store
            .run_blocking(|s| {
                s.write(|tx| {
                    if let Some(mut a) = Self::load_current(tx)? {
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

    #[test]
    fn transitions_follow_the_table() {
        use RegistrationState as S;
        let h = ChallengeHandle {
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
            !S::Blocked {
                reason: BlockedReason::ServerRefused { code: None }
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
                        reason: BlockedReason::ServerRefused { code: Some(403) },
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
