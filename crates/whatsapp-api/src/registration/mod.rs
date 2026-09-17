//! Primary registration as a durable, branching state machine (README section 7).
//!
//! The Android registration protocol remains incomplete for the active
//! profile. A version-scoped request-construction layer for a newer signed
//! comparison build lives in [`wire`], but verification steps and a reviewed
//! HTTPS backend are still missing. The state machine, persistence, challenge
//! binding and typed steps are implemented and tested; the profile-specific driver is a
//! [`RegistrationFlow`] whose only current implementation reports
//! [`BlockedReason::ProfileIncomplete`]. No request is ever sent and no
//! OTP is ever requested by this code.

pub mod wire;

use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::error::{Error, Result};
use crate::model::ids::AccountGeneration;
use crate::model::phone::PhoneNumber;
use crate::model::time::Timestamp;
use crate::profile::ProtocolProfile;
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

/// Durable registration state (README section 7 table).
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
    Pin<Box<dyn Future<Output = Result<RegistrationState>> + Send + 'a>>;

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

    /// Prepare prerequisites and request the first challenge. Returns the
    /// next state (typically `ChallengePending`, `CoolingDown` or `Blocked`).
    fn begin<'a>(&'a self, attempt: &'a RegistrationAttempt) -> RegistrationFuture<'a>;

    /// Submit an answer for the bound handle. Returns the next state.
    fn submit<'a>(
        &'a self,
        attempt: &'a RegistrationAttempt,
        handle: &'a ChallengeHandle,
        answer: &'a Answer,
    ) -> RegistrationFuture<'a>;
}

/// The only flow available while the Android registration protocol is
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
            })
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
            })
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
        next: RegistrationState,
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
                    attempt.transition(next, Timestamp::now())?;
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
    /// restarted implicitly (README section 7).
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
                    if let Some(existing) = Self::load_current(tx)? {
                        if existing.generation == tx.generation()
                            && *existing.phone() == phone
                            && existing.profile_id == profile_id
                            && (existing.state.is_terminal()
                                || !matches!(existing.state, RegistrationState::Blocked { .. }))
                        {
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
                    // any request can be sent (README section 7).
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
                    RegistrationState::RegistrationOutcomeUnknown { step: 0 },
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
                    RegistrationState::RegistrationOutcomeUnknown { step: bound.step },
                )
                .await?;
                Err(error)
            }
        }
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
                })
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
                        })
                    }
                    Answer::Pin(p) if p.as_str() == "6157" && handle.step == 2 => {
                        Ok(RegistrationState::Registered)
                    }
                    _ => Ok(RegistrationState::Blocked {
                        reason: BlockedReason::ServerRefused { code: Some(403) },
                    }),
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
                })
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
                })
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
                })
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
                })
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
        // Completed attempts are never restarted implicitly.
        let again = svc.begin(phone(), ChallengeMethod::Sms).await.unwrap();
        assert_eq!(again, RegistrationStep::Completed);
        let other = PhoneNumber::new(1, "5559876543").unwrap();
        assert!(svc.begin(other, ChallengeMethod::Sms).await.is_err());
        // The persisted record never contains the code or PIN.
        let raw = std::fs::read(s.path()).unwrap();
        assert!(!raw.windows(6).any(|w| w == b"928374"));
        let cur = svc.current().await.unwrap().unwrap();
        let json = serde_json::to_string(&cur).unwrap();
        assert!(!json.contains("928374") && !json.contains("6157"));
        assert!(!format!("{cur:?}").contains("5551234567"));
    }
}
