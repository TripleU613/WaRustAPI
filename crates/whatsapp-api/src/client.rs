//! Public facade (README section 14).
//!
//! `WhatsAppClient` is a cloneable handle over one account actor and one
//! encrypted protocol store. Every operation either works offline against
//! durable state or returns a typed error naming the missing capability or
//! profile fact. Nothing here contacts a server for the current profile.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::watch;

use crate::capabilities::{Capability, CapabilityMatrix};
use crate::error::{Error, Result};
use crate::model::ids::{AccountId, ContactId, GroupId, LocalMessageId};
use crate::model::phone::PhoneNumber;
use crate::model::receipt::Receipt;
use crate::model::send::{SendRequest, SendStatus, SendTicket};
use crate::outbox::Outbox;
use crate::profile::{Limits, ProtocolProfile};
use crate::registration::{
    Answer, ChallengeHandle, ChallengeMethod, RegistrationFlow, RegistrationService,
    RegistrationStep, UnsupportedFlow,
};
use crate::runtime::{ConnectionStatus, RuntimeHandle};
use crate::store::keys::KeySource;
use crate::store::{ConsumerId, OpenOptions, ProtocolStore};
use crate::sync::SyncEngine;

/// Pagination request for list operations.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PageRequest {
    /// Opaque continuation from a previous page.
    pub continuation: Option<String>,
    /// Maximum items.
    pub limit: u32,
}

/// A bounded page with continuation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page<T> {
    /// Items.
    pub items: Vec<T>,
    /// Continuation for the next page, if any.
    pub continuation: Option<String>,
}

/// Group creation request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateGroupRequest {
    /// Caller idempotency key (group creation is not idempotent server-side).
    pub client_request_id: crate::model::ids::ClientRequestId,
    /// Subject.
    pub subject: String,
    /// Initial participants.
    pub participants: Vec<ContactId>,
}

/// Explicit read/played receipt policy (README section 13.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReceiptPolicy {
    /// Only send receipts the application explicitly requests.
    #[default]
    ExplicitOnly,
}

/// Builder for [`WhatsAppClient`].
pub struct ClientBuilder {
    profile: Option<ProtocolProfile>,
    store_path: Option<PathBuf>,
    account_id: Option<AccountId>,
    key_source: Option<Arc<dyn KeySource>>,
    limits: Option<Limits>,
    namespace: String,
    receipt_policy: ReceiptPolicy,
    registration_flow: Option<Arc<dyn RegistrationFlow>>,
    create_store: bool,
}

impl std::fmt::Debug for ClientBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientBuilder")
            .field("profile", &self.profile.as_ref().map(|p| &p.id))
            .field("store_path", &self.store_path)
            .field("account_id", &self.account_id)
            .field("has_key_source", &self.key_source.is_some())
            .field("namespace", &self.namespace)
            .finish_non_exhaustive()
    }
}

impl ClientBuilder {
    fn new() -> Self {
        Self {
            profile: None,
            store_path: None,
            account_id: None,
            key_source: None,
            limits: None,
            namespace: "default".into(),
            receipt_policy: ReceiptPolicy::default(),
            registration_flow: None,
            create_store: true,
        }
    }

    /// Protocol profile (defaults to the Android reference profile).
    #[must_use]
    pub fn profile(mut self, profile: ProtocolProfile) -> Self {
        self.profile = Some(profile);
        self
    }

    /// Path of the account's `protocol.redb`.
    #[must_use]
    pub fn store_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.store_path = Some(path.into());
        self
    }

    /// Local account identifier.
    #[must_use]
    pub fn account_id(mut self, id: AccountId) -> Self {
        self.account_id = Some(id);
        self
    }

    /// Master key source.
    #[must_use]
    pub fn key_source(mut self, source: Arc<dyn KeySource>) -> Self {
        self.key_source = Some(source);
        self
    }

    /// Local limits (default: `Limits::default()` or the profile's).
    #[must_use]
    pub fn limits(mut self, limits: Limits) -> Self {
        self.limits = Some(limits);
        self
    }

    /// Idempotency namespace for `ClientRequestId`s.
    #[must_use]
    pub fn namespace(mut self, ns: impl Into<String>) -> Self {
        self.namespace = ns.into();
        self
    }

    /// Receipt policy.
    #[must_use]
    pub fn receipt_policy(mut self, policy: ReceiptPolicy) -> Self {
        self.receipt_policy = policy;
        self
    }

    /// Override the registration flow (tests, future profile drivers).
    #[must_use]
    pub fn registration_flow(mut self, flow: Arc<dyn RegistrationFlow>) -> Self {
        self.registration_flow = Some(flow);
        self
    }

    /// Whether to create the store if missing (default true).
    #[must_use]
    pub fn create_store(mut self, create: bool) -> Self {
        self.create_store = create;
        self
    }

    /// Open the store and start the account actor. No registration and no
    /// network activity happen here.
    pub async fn build(self) -> Result<WhatsAppClient> {
        let mut profile = match self.profile {
            Some(p) => p,
            None => ProtocolProfile::android_reference()?,
        };
        if let Some(l) = self.limits {
            profile.limits = l;
        }
        let path = self
            .store_path
            .ok_or_else(|| Error::invalid("store_path", "required"))?;
        let account_id = self
            .account_id
            .ok_or_else(|| Error::invalid("account_id", "required"))?;
        let key_source = self
            .key_source
            .ok_or_else(|| Error::invalid("key_source", "required"))?;
        let create = self.create_store;
        let store = tokio::task::spawn_blocking(move || {
            ProtocolStore::open(OpenOptions {
                path,
                account_id,
                key_source,
                create,
            })
        })
        .await
        .map_err(|_| Error::Cancelled)??;
        let store = Arc::new(store);
        let profile = Arc::new(profile);
        let flow = self
            .registration_flow
            .unwrap_or_else(|| Arc::new(UnsupportedFlow::for_profile(&profile)));
        let registration = RegistrationService::new(store.clone(), &profile, flow);
        let runtime = crate::runtime::spawn(store.clone(), profile.clone());
        Ok(WhatsAppClient {
            inner: Arc::new(Inner {
                capabilities: CapabilityMatrix::current(profile.id.clone()),
                outbox: Outbox::new(self.namespace, profile.limits),
                store,
                profile,
                registration,
                runtime,
                receipt_policy: self.receipt_policy,
            }),
        })
    }
}

struct Inner {
    store: Arc<ProtocolStore>,
    profile: Arc<ProtocolProfile>,
    registration: RegistrationService,
    runtime: RuntimeHandle,
    outbox: Outbox,
    capabilities: CapabilityMatrix,
    receipt_policy: ReceiptPolicy,
}

/// Cloneable handle to one account.
#[derive(Clone)]
pub struct WhatsAppClient {
    inner: Arc<Inner>,
}

impl std::fmt::Debug for WhatsAppClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WhatsAppClient")
            .field("account_id", self.inner.store.account_id())
            .field("profile", &self.inner.profile.id)
            .finish_non_exhaustive()
    }
}

impl WhatsAppClient {
    /// Start building a client.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// The protocol profile in use.
    pub fn profile(&self) -> &ProtocolProfile {
        &self.inner.profile
    }

    /// The protocol store (for tooling; application code should use the API).
    pub fn store(&self) -> &Arc<ProtocolStore> {
        &self.inner.store
    }

    /// Registration operations.
    pub fn registration(&self) -> &RegistrationService {
        &self.inner.registration
    }

    /// Capability matrix and tested profile information.
    pub fn capabilities(&self) -> &CapabilityMatrix {
        &self.inner.capabilities
    }

    /// Configured receipt policy.
    pub fn receipt_policy(&self) -> ReceiptPolicy {
        self.inner.receipt_policy
    }

    /// Convenience over `registration().begin` for an SMS challenge.
    pub async fn request_otp(&self, phone: PhoneNumber) -> Result<RegistrationStep> {
        self.inner
            .registration
            .begin(phone, ChallengeMethod::Sms)
            .await
    }

    /// Convenience over `registration().submit` for a code. Surfaces further
    /// steps rather than pretending the flow ends here.
    pub async fn verify_otp(
        &self,
        handle: ChallengeHandle,
        code: zeroize::Zeroizing<String>,
    ) -> Result<RegistrationStep> {
        self.inner
            .registration
            .submit(handle, Answer::Code(code))
            .await
    }

    /// Idempotently start one runtime connection and wait for readiness.
    pub async fn connect(&self, timeout: Duration) -> Result<()> {
        if let crate::runtime::ConnectionState::Ready { .. } = self.inner.runtime.current().state {
            return Ok(());
        }
        self.inner.runtime.connect(timeout).await
    }

    /// Live connection state watch.
    pub fn connection_state(&self) -> watch::Receiver<ConnectionStatus> {
        self.inner.runtime.status()
    }

    /// Stop reconnecting and shut down the connection. Not a logout.
    pub async fn disconnect(&self) -> Result<()> {
        self.inner.runtime.disconnect().await
    }

    /// Durably enqueue a send and return its ticket.
    pub async fn send_message(&self, request: SendRequest) -> Result<SendTicket> {
        let (ticket, _new) = self
            .inner
            .outbox
            .enqueue_async(&self.inner.store, request)
            .await?;
        self.inner.runtime.nudge();
        Ok(ticket)
    }

    /// Current aggregate and per-recipient facts for a ticket.
    pub async fn message_status(&self, ticket: &SendTicket) -> Result<SendStatus> {
        let outbox = self.inner.outbox.clone();
        let id = ticket.local_id;
        self.inner
            .store
            .run_blocking(move |s| {
                s.read(|tx| outbox.status(tx, id))?
                    .ok_or_else(|| Error::invalid("ticket", "unknown local message id"))
            })
            .await
    }

    /// Cancel a queued send. Returns `false` for an unknown ticket.
    pub async fn cancel_send(&self, local_id: LocalMessageId) -> Result<bool> {
        let outbox = self.inner.outbox.clone();
        self.inner
            .store
            .run_blocking(move |s| s.write(|tx| outbox.cancel(tx, local_id)))
            .await
    }

    /// Queue an explicit receipt under the configured privacy policy.
    pub async fn send_receipt(&self, _receipt: Receipt) -> Result<()> {
        self.inner.capabilities.require(Capability::Receipts)
    }

    /// Bounded page of groups.
    pub async fn groups(&self, _page: PageRequest) -> Result<Page<crate::sync::Group>> {
        self.inner.capabilities.require(Capability::Groups)?;
        Err(Error::unsupported(Capability::Groups, "unreachable"))
    }

    /// Create a group with request identity tracking.
    pub async fn create_group(&self, _request: CreateGroupRequest) -> Result<GroupId> {
        self.inner.capabilities.require(Capability::Groups)?;
        Err(Error::unsupported(Capability::Groups, "unreachable"))
    }

    /// Bounded staged upload of caller-provided encoded content.
    pub async fn upload_media(
        &self,
        _source: PathBuf,
    ) -> Result<crate::model::message::MediaReference> {
        self.inner.capabilities.require(Capability::Media)?;
        Err(Error::unsupported(Capability::Media, "unreachable"))
    }

    /// Verified download into a sink path.
    pub async fn download_media(
        &self,
        _reference: &crate::model::message::MediaReference,
        _sink: PathBuf,
    ) -> Result<u64> {
        self.inner.capabilities.require(Capability::Media)?;
        Err(Error::unsupported(Capability::Media, "unreachable"))
    }

    /// Durable event replay for a named consumer.
    pub fn events(&self, consumer: ConsumerId) -> SyncEngine {
        SyncEngine::new(self.inner.store.clone(), consumer)
    }

    /// Shut down the runtime. The store is closed when the last handle drops.
    pub async fn shutdown(&self) {
        self.inner.runtime.shutdown().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ids::{ChatId, ClientRequestId};
    use crate::model::jid::Jid;
    use crate::model::message::MessageContent;
    use crate::model::send::AttemptProgress;
    use crate::runtime::ConnectionState;
    use crate::store::{MasterKey, StaticKeySource};
    use crate::sync::SyncEvent;

    async fn client(dir: &tempfile::TempDir) -> WhatsAppClient {
        WhatsAppClient::builder()
            .store_path(dir.path().join("protocol.redb"))
            .account_id(AccountId::new("acct").unwrap())
            .key_source(Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                [5u8; 32],
            ))))
            .build()
            .await
            .unwrap()
    }

    fn req(id: &str) -> SendRequest {
        SendRequest {
            client_request_id: ClientRequestId::new(id).unwrap(),
            chat: ChatId::from_jid(&Jid::parse("15550000000@s.whatsapp.net").unwrap()).unwrap(),
            content: MessageContent::Text {
                body: "hello".into(),
                quoted: None,
            },
        }
    }

    #[tokio::test]
    async fn builder_requires_inputs() {
        let err = WhatsAppClient::builder().build().await.unwrap_err();
        assert!(matches!(
            err,
            Error::InvalidInput {
                field: "store_path",
                ..
            }
        ));
    }

    #[tokio::test]
    async fn durable_enqueue_status_and_events() {
        let dir = tempfile::tempdir().unwrap();
        let c = client(&dir).await;
        let t1 = c.send_message(req("r1")).await.unwrap();
        let t2 = c.send_message(req("r1")).await.unwrap();
        assert_eq!(t1, t2, "identical repeat returns the same ticket");
        let st = c.message_status(&t1).await.unwrap();
        assert_eq!(st.progress, AttemptProgress::Queued);
        assert!(!st.server_accepted());
        let engine = c.events(ConsumerId::new("bridge").unwrap());
        let batch = engine.next_batch(Default::default(), 10).await.unwrap();
        assert_eq!(batch.events.len(), 1);
        assert!(matches!(
            batch.events[0].event,
            SyncEvent::SendStatusChanged(_)
        ));
        engine.ack(batch.ack_cursor).await.unwrap();
        assert!(c.cancel_send(t1.local_id).await.unwrap());
        let st = c.message_status(&t1).await.unwrap();
        assert_eq!(st.progress, AttemptProgress::Cancelled);
        c.shutdown().await;
        // State survives restart of the process (new client over same store).
        // Every handle to the store must be released first: the engine holds one.
        drop(engine);
        drop(c);
        let c2 = client(&dir).await;
        let st = c2.message_status(&t1).await.unwrap();
        assert_eq!(st.progress, AttemptProgress::Cancelled);
        let progress = c2
            .events(ConsumerId::new("bridge").unwrap())
            .sync_once(10, |_| Ok(()))
            .await
            .unwrap();
        assert_eq!(
            progress.drained, 1,
            "only the cancel event remained unacknowledged"
        );
        c2.shutdown().await;
    }

    #[tokio::test]
    async fn connect_is_typed_not_a_loop() {
        let dir = tempfile::tempdir().unwrap();
        let c = client(&dir).await;
        let err = c.connect(Duration::from_secs(5)).await.unwrap_err();
        assert!(matches!(err, Error::NotRegistered), "{err:?}");
        let status = c.connection_state().borrow().clone();
        assert_eq!(status.state, ConnectionState::NeedsRegistration);
        assert_eq!(status.epoch.0, 1);
        // Registration itself is blocked for this profile, with the missing
        // ledger facts named.
        let step = c
            .request_otp(PhoneNumber::new(1, "5551234567").unwrap())
            .await
            .unwrap();
        assert!(matches!(step, RegistrationStep::Blocked(_)));
        c.disconnect().await.unwrap();
        c.shutdown().await;
    }

    #[tokio::test]
    async fn unsupported_operations_are_typed() {
        let dir = tempfile::tempdir().unwrap();
        let c = client(&dir).await;
        assert!(matches!(
            c.groups(PageRequest::default()).await,
            Err(Error::Unsupported {
                capability: Capability::Groups,
                ..
            })
        ));
        assert!(matches!(
            c.upload_media(PathBuf::from("/dev/null")).await,
            Err(Error::Unsupported {
                capability: Capability::Media,
                ..
            })
        ));
        assert_eq!(
            c.capabilities().level(Capability::LocalStore),
            crate::capabilities::SupportLevel::Supported
        );
        c.shutdown().await;
    }
}
