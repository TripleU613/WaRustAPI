//! Offline-capable, explicitly lab-scoped registration request factory.
//!
//! Uses Signal's type-prefixed signed-prekey signature convention, with raw
//! public keys on the registration wire. Android acceptance of this convention
//! and caller-selected IDs still needs independent evidence. Never installed
//! by the default client; constructing requests performs no network I/O.

use std::{fmt, sync::Arc};

use serde::{Deserialize, Serialize};
use wacore_libsignal::core::curve::{KeyPair, PublicKey};
use zeroize::Zeroizing;

use super::prerequisites::ComparisonRegistrationPrerequisites;
use super::wire::*;
use super::{AttemptId, ChallengeMethod, ComparisonRequestFactory, RegistrationAttempt};
use crate::model::ids::AccountGeneration;
use crate::session::{AccountRecord, KeyMaterial, NoiseStaticKey, SignalIdentityKey};
use crate::store::{ProtocolStore, Table, WriteTx};
use crate::{Error, Result};

pub use super::request::{LAB_PROFILE_ID, LabRegistrationConfig};
const MATERIAL_KEY: &[u8] = b"registration-lab-material";
const RECORD_VERSION: u16 = 1;

#[derive(Serialize, Deserialize)]
struct Material {
    attempt_id: AttemptId,
    method: ChallengeMethod,
    generation: AccountGeneration,
    config: LabRegistrationConfig,
    prerequisites: ComparisonRegistrationPrerequisites,
    noise: KeyMaterial,
    identity: KeyMaterial,
    signed_pre_key: KeyMaterial,
    signature: Vec<u8>,
}

fn invalid(message: &'static str) -> Error {
    Error::invalid("registration_lab", message)
}

fn material(pair: &KeyPair) -> KeyMaterial {
    let mut public = [0; 32];
    public.copy_from_slice(pair.public_key.public_key_bytes());
    KeyMaterial {
        public,
        private: *pair.private_key.serialize(),
    }
}

impl Material {
    fn generate(attempt: &RegistrationAttempt, config: &LabRegistrationConfig) -> Result<Self> {
        let mut rng = rand::rng();
        let noise = KeyPair::generate(&mut rng);
        let identity = KeyPair::generate(&mut rng);
        let signed = KeyPair::generate(&mut rng);
        let signature = identity
            .calculate_signature(&signed.public_key.serialize(), &mut rng)
            .map_err(|_| invalid("signed-prekey signature failed"))?;
        Ok(Self {
            attempt_id: attempt.id,
            method: attempt.method.clone(),
            generation: attempt.generation,
            config: config.clone(),
            prerequisites: ComparisonRegistrationPrerequisites::generate_with(&config.device)?,
            noise: material(&noise),
            identity: material(&identity),
            signed_pre_key: material(&signed),
            signature: signature.to_vec(),
        })
    }

    fn check(&self, attempt: &RegistrationAttempt, config: &LabRegistrationConfig) -> Result<()> {
        if self.attempt_id != attempt.id
            || self.method != attempt.method
            || self.generation != attempt.generation
            || &self.config != config
        {
            return Err(invalid(
                "persisted material belongs to a different attempt or configuration",
            ));
        }
        let identity = PublicKey::from_djb_public_key_bytes(&self.identity.public)
            .map_err(|_| invalid("invalid stored identity"))?;
        let signed = PublicKey::from_djb_public_key_bytes(&self.signed_pre_key.public)
            .map_err(|_| invalid("invalid stored signed prekey"))?;
        if !identity.verify_signature(&signed.serialize(), &self.signature) {
            return Err(invalid(
                "stored signed-prekey signature failed verification",
            ));
        }
        Ok(())
    }

    fn check_account(&self, account: Option<AccountRecord>) -> Result<()> {
        let account = account.ok_or_else(|| invalid("persisted account is missing"))?;
        if account.generation != self.generation
            || account.phone.as_ref() != Some(&self.config.phone)
            || account.signal_registration_id != Some(self.config.registration_id)
            || account.noise_static.as_ref().map(|key| key.0.public) != Some(self.noise.public)
            || account.signal_identity.as_ref().map(|key| key.0.public)
                != Some(self.identity.public)
        {
            return Err(invalid(
                "persisted account does not match registration material",
            ));
        }
        Ok(())
    }

    fn bundle(&self) -> Result<RegistrationKeyBundle> {
        RegistrationKeyBundle::new(
            self.noise.public,
            self.identity.public.to_vec(),
            self.config.registration_id,
            self.config.signed_pre_key_id,
            self.signed_pre_key.public.to_vec(),
            self.signature.clone(),
        )
    }

    fn common(&self) -> Result<RegistrationCommonParams> {
        self.prerequisites.common_params(
            self.config.phone.clone(),
            self.config.language.clone(),
            self.config.locale.clone(),
        )
    }
}

/// Durable lab request factory. Keys, identifiers and account material commit
/// in the service's existing preparation transaction, before any HTTP request.
/// Restart requires exactly the same config. Existing material is never silently
/// replaced: a new attempt requires a fresh store/account generation.
pub struct StoredComparisonRequestFactory {
    store: Arc<ProtocolStore>,
    config: LabRegistrationConfig,
}

impl fmt::Debug for StoredComparisonRequestFactory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StoredComparisonRequestFactory")
            .finish_non_exhaustive()
    }
}

impl StoredComparisonRequestFactory {
    /// Validate configuration without generating keys or contacting the server.
    pub fn new(store: Arc<ProtocolStore>, config: LabRegistrationConfig) -> Result<Self> {
        config.validate()?;
        Ok(Self { store, config })
    }

    fn check_attempt(&self, attempt: &RegistrationAttempt) -> Result<()> {
        if attempt.profile_id != LAB_PROFILE_ID || attempt.phone() != &self.config.phone {
            return Err(invalid(
                "requires the explicit comparison lab profile and configured phone",
            ));
        }
        match attempt.method {
            ChallengeMethod::Sms | ChallengeMethod::Voice => Ok(()),
            _ => Err(invalid("unsupported challenge method")),
        }
    }

    fn load(&self, attempt: &RegistrationAttempt) -> Result<Material> {
        self.check_attempt(attempt)?;
        self.store.read(|tx| {
            if tx.generation() != attempt.generation {
                return Err(invalid("stale account generation"));
            }
            let record: Material = tx
                .get_json(Table::Account, MATERIAL_KEY, RECORD_VERSION)?
                .ok_or_else(|| invalid("registration material has not been committed"))?;
            record.check(attempt, &self.config)?;
            record.check_account(AccountRecord::load(tx)?)?;
            Ok(record)
        })
    }
}

impl ComparisonRequestFactory for StoredComparisonRequestFactory {
    fn prepare(&self, tx: &mut WriteTx<'_>, attempt: &RegistrationAttempt) -> Result<()> {
        self.check_attempt(attempt)?;
        if tx.generation() != attempt.generation {
            return Err(invalid("stale account generation"));
        }
        if let Some(record) =
            tx.get_json::<Material>(Table::Account, MATERIAL_KEY, RECORD_VERSION)?
        {
            record.check(attempt, &self.config)?;
            return record.check_account(AccountRecord::load_in_write(tx)?);
        }
        if AccountRecord::load_in_write(tx)?.is_some() {
            return Err(invalid("refusing to replace existing account material"));
        }
        let record = Material::generate(attempt, &self.config)?;
        record.common()?;
        record.bundle()?;
        AccountRecord {
            generation: attempt.generation,
            phone: Some(attempt.phone().clone()),
            noise_static: Some(NoiseStaticKey(record.noise.clone())),
            signal_identity: Some(SignalIdentityKey(record.identity.clone())),
            signal_registration_id: Some(self.config.registration_id),
            ..AccountRecord::default()
        }
        .save(tx)?;
        tx.put_json(Table::Account, MATERIAL_KEY, RECORD_VERSION, &record)
    }

    fn code_request(&self, attempt: &RegistrationAttempt) -> Result<EncodedRegistrationRequest> {
        let record = self.load(attempt)?;
        let method = match attempt.method {
            ChallengeMethod::Sms => "sms",
            ChallengeMethod::Voice => "voice",
            _ => return Err(invalid("unsupported challenge method")),
        };
        let mut form = RegistrationForm::new();
        record.common()?.append_to_form(&mut form)?;
        RegistrationCodeParams::new(record.config.token.as_str(), method)?
            .append_to_form(&mut form)?;
        super::request::append_fields(&mut form, &record.config.fields.code)?;
        record.bundle()?.append_to_form(&mut form)?;
        WHATSAPP_2_26_37_70.request(RegistrationEndpoint::Code, &form)
    }

    fn complete_request(
        &self,
        attempt: &RegistrationAttempt,
        code: &Zeroizing<String>,
    ) -> Result<EncodedRegistrationRequest> {
        let record = self.load(attempt)?;
        let mut form = RegistrationForm::new();
        record.common()?.append_to_form(&mut form)?;
        RegistrationCompleteParams::new(code.as_str())?.append_to_form(&mut form)?;
        super::request::append_fields(&mut form, &record.config.fields.register)?;
        record.bundle()?.append_to_form(&mut form)?;
        WHATSAPP_2_26_37_70.request(RegistrationEndpoint::Register, &form)
    }

    fn security_request(
        &self,
        attempt: &RegistrationAttempt,
        pin: &Zeroizing<String>,
    ) -> Result<EncodedRegistrationRequest> {
        let record = self.load(attempt)?;
        let mut form = RegistrationForm::new();
        record.common()?.append_to_form(&mut form)?;
        RegistrationSecurityParams::verify(pin.as_str())?.append_to_form(&mut form)?;
        super::request::append_fields(&mut form, &record.config.fields.security)?;
        record.bundle()?.append_to_form(&mut form)?;
        WHATSAPP_2_26_37_70.request(RegistrationEndpoint::Security, &form)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::phone::PhoneNumber;
    use crate::model::{ids::AccountId, time::Timestamp};
    use crate::registration::{
        Answer, ComparisonHttpFlow, RegistrationService, RegistrationState, RegistrationStep,
        RequiredAction,
    };
    use crate::store::{MasterKey, OpenOptions, StaticKeySource};

    fn open(dir: &std::path::Path) -> Arc<ProtocolStore> {
        Arc::new(
            ProtocolStore::open(OpenOptions {
                path: dir.join("test.redb"),
                account_id: AccountId::new("lab").unwrap(),
                key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([7; 32]))),
                create: true,
            })
            .unwrap(),
        )
    }

    fn config() -> LabRegistrationConfig {
        LabRegistrationConfig {
            phone: PhoneNumber::new(1, "5551234567").unwrap(),
            language: "en".into(),
            locale: "US".into(),
            token: Zeroizing::new("offline-test-token-only".into()),
            registration_id: 17,
            signed_pre_key_id: 1,
            method: Default::default(),
            client: Default::default(),
            transport: Default::default(),
            device: Default::default(),
            fields: Default::default(),
            diagnostics: true,
        }
    }

    fn attempt(store: &ProtocolStore) -> RegistrationAttempt {
        RegistrationAttempt {
            id: AttemptId([3; 16]),
            generation: store.generation(),
            profile_id: LAB_PROFILE_ID.into(),
            phone: config().phone,
            method: ChallengeMethod::Sms,
            state: RegistrationState::Preparing,
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            history: Vec::new(),
        }
    }

    #[test]
    fn profile_device_and_endpoint_overrides_reach_only_their_forms() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(dir.path());
        let mut cfg = config();
        let fdid = "12345678-1234-4234-8234-123456789abc";
        cfg.device.facebook_device_id = Some(Zeroizing::new(fdid.into()));
        cfg.fields
            .code
            .insert("context".into(), Zeroizing::new("code only+".into()));
        cfg.fields
            .register
            .insert("context".into(), Zeroizing::new("register only".into()));
        cfg.fields
            .security
            .insert("context".into(), Zeroizing::new("security only".into()));
        let factory = StoredComparisonRequestFactory::new(store.clone(), cfg).unwrap();
        let a = attempt(&store);
        store.write(|tx| factory.prepare(tx, &a)).unwrap();
        let code = factory.code_request(&a).unwrap();
        let body = std::str::from_utf8(code.body()).unwrap();
        assert!(body.contains(&format!("fdid={fdid}")));
        assert!(body.contains("context=code+only%2B"));
        assert!(!code.diagnostic_summary().to_string().contains(fdid));
        let complete = factory
            .complete_request(&a, &Zeroizing::new("654321".into()))
            .unwrap();
        let security = factory
            .security_request(&a, &Zeroizing::new("789012".into()))
            .unwrap();
        assert!(
            std::str::from_utf8(complete.body())
                .unwrap()
                .contains("context=register+only")
        );
        assert!(
            std::str::from_utf8(security.body())
                .unwrap()
                .contains("context=security+only")
        );
    }

    #[test]
    fn committed_keys_and_requests_survive_reopen_without_rotation() {
        let dir = tempfile::tempdir().unwrap();
        let (a, body, public) = {
            let store = open(dir.path());
            let factory = StoredComparisonRequestFactory::new(store.clone(), config()).unwrap();
            let a = attempt(&store);
            assert!(factory.code_request(&a).is_err());
            store.write(|tx| factory.prepare(tx, &a)).unwrap();
            let first = factory.code_request(&a).unwrap();
            store.write(|tx| factory.prepare(tx, &a)).unwrap();
            assert_eq!(first.body(), factory.code_request(&a).unwrap().body());
            let record = factory.load(&a).unwrap();
            assert_ne!(record.noise.public, record.identity.public);
            assert_ne!(record.identity.public, record.signed_pre_key.public);
            let disk = std::fs::read(store.path()).unwrap();
            let private_json = serde_json::to_vec(&record.identity.private).unwrap();
            assert!(!disk.windows(private_json.len()).any(|w| w == private_json));
            assert!(
                !disk
                    .windows(config().token.len())
                    .any(|w| w == config().token.as_bytes())
            );
            assert!(!format!("{factory:?}").contains(config().token.as_str()));
            (a, first.body().to_vec(), record.identity.public)
        };
        let store = open(dir.path());
        let factory = StoredComparisonRequestFactory::new(store.clone(), config()).unwrap();
        store.write(|tx| factory.prepare(tx, &a)).unwrap();
        assert_eq!(body, factory.code_request(&a).unwrap().body());
        assert_eq!(public, factory.load(&a).unwrap().identity.public);
        let complete = factory
            .complete_request(&a, &Zeroizing::new("654321".into()))
            .unwrap();
        let security = factory
            .security_request(&a, &Zeroizing::new("789012".into()))
            .unwrap();
        assert!(complete.url().ends_with("/v2/register"));
        assert!(security.url().ends_with("/v2/security"));
        let stored = store
            .read(|tx| tx.get(Table::Account, MATERIAL_KEY, RECORD_VERSION))
            .unwrap()
            .unwrap();
        let text = String::from_utf8(stored).unwrap();
        assert!(!text.contains("654321"));
        assert!(!text.contains("789012"));
    }

    #[test]
    fn preparation_failure_rolls_back_keys_and_account_together() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(dir.path());
        let factory = StoredComparisonRequestFactory::new(store.clone(), config()).unwrap();
        let a = attempt(&store);
        let result: Result<()> = store.write(|tx| {
            factory.prepare(tx, &a)?;
            Err(invalid("injected rollback"))
        });
        assert!(result.is_err());
        assert!(store.read(AccountRecord::load).unwrap().is_none());
        assert!(factory.code_request(&a).is_err());
    }

    #[test]
    fn rejects_wrong_profile_stale_attempt_config_and_account_replacement() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(dir.path());
        let factory = StoredComparisonRequestFactory::new(store.clone(), config()).unwrap();
        let a = attempt(&store);
        let mut wrong = attempt(&store);
        wrong.profile_id = "android-reference".into();
        assert!(store.write(|tx| factory.prepare(tx, &wrong)).is_err());
        store.write(|tx| factory.prepare(tx, &a)).unwrap();
        let mut wrong = attempt(&store);
        wrong.id = AttemptId([4; 16]);
        assert!(factory.code_request(&wrong).is_err());
        assert!(store.write(|tx| factory.prepare(tx, &wrong)).is_err());
        let mut wrong = attempt(&store);
        wrong.generation = AccountGeneration(a.generation.0 + 1);
        assert!(factory.code_request(&wrong).is_err());
        let mut wrong = attempt(&store);
        wrong.method = ChallengeMethod::Voice;
        assert!(factory.code_request(&wrong).is_err());
        let mut changed = config();
        changed.token = Zeroizing::new("changed-token".into());
        let changed = StoredComparisonRequestFactory::new(store.clone(), changed).unwrap();
        assert!(changed.code_request(&a).is_err());
        store
            .write(|tx| {
                let mut account = AccountRecord::load_in_write(tx)?.unwrap();
                account.signal_registration_id = Some(999);
                account.save(tx)
            })
            .unwrap();
        assert!(factory.code_request(&a).is_err());
    }

    #[test]
    fn invalid_ids_and_existing_accounts_are_not_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(dir.path());
        let mut malformed = serde_json::to_value(config()).unwrap();
        malformed["phone"]["country_code"] = serde_json::json!(0);
        let malformed = serde_json::from_value(malformed).unwrap();
        assert!(StoredComparisonRequestFactory::new(store.clone(), malformed).is_err());
        let mut bad = config();
        bad.signed_pre_key_id = 0x0100_0000;
        assert!(StoredComparisonRequestFactory::new(store.clone(), bad).is_err());
        let factory = StoredComparisonRequestFactory::new(store.clone(), config()).unwrap();
        store
            .write(|tx| {
                AccountRecord {
                    generation: tx.generation(),
                    ..AccountRecord::default()
                }
                .save(tx)
            })
            .unwrap();
        assert!(
            store
                .write(|tx| factory.prepare(tx, &attempt(&store)))
                .is_err()
        );
        assert!(
            store
                .read(AccountRecord::load)
                .unwrap()
                .unwrap()
                .noise_static
                .is_none()
        );
    }

    #[derive(Debug, Default)]
    struct MockTransport(std::sync::Mutex<Vec<String>>);

    impl RegistrationTransport for MockTransport {
        fn send<'a>(
            &'a self,
            request: EncodedRegistrationRequest,
        ) -> RegistrationTransportFuture<'a> {
            Box::pin(async move {
                let body = std::str::from_utf8(request.body()).unwrap();
                for field in ["authkey=", "e_ident=", "e_skey_sig=", "e_regid="] {
                    assert!(body.contains(field));
                }
                self.0.lock().unwrap().push(request.url().to_owned());
                let response = if request.url().ends_with("/v2/code") {
                    br#"{"status":"sent"}"#.as_slice()
                } else {
                    assert!(request.url().ends_with("/v2/register"));
                    assert!(body.contains("code=123456"));
                    br#"{"status":"ok","login":"mock-login"}"#.as_slice()
                };
                RegistrationHttpResponse::new(200, Zeroizing::new(response.to_vec()))
            })
        }
    }

    #[tokio::test]
    async fn service_uses_committed_keys_for_mock_onboarding_without_duplicate_code_request() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(dir.path());
        let factory =
            Arc::new(StoredComparisonRequestFactory::new(store.clone(), config()).unwrap());
        let transport = Arc::new(MockTransport::default());
        let flow = Arc::new(ComparisonHttpFlow::new(factory, transport.clone()));
        let mut profile = crate::profile::ProtocolProfile::android_reference().unwrap();
        profile.id = LAB_PROFILE_ID.into();
        let svc = RegistrationService::new(store.clone(), &profile, flow);
        let first = svc
            .begin(config().phone, ChallengeMethod::Sms)
            .await
            .unwrap();
        let account_before = store.read(AccountRecord::load).unwrap().unwrap();
        assert_eq!(
            first,
            svc.begin(config().phone, ChallengeMethod::Sms)
                .await
                .unwrap()
        );
        assert_eq!(transport.0.lock().unwrap().len(), 1);
        let RegistrationStep::Action(RequiredAction::EnterCode(handle)) = first else {
            panic!("expected code challenge")
        };
        assert_eq!(
            svc.submit(handle, Answer::Code(Zeroizing::new("123456".into())))
                .await
                .unwrap(),
            RegistrationStep::Completed
        );
        let after = store.read(AccountRecord::load).unwrap().unwrap();
        assert!(after.is_registered());
        assert_eq!(
            account_before.noise_static.unwrap().0.public,
            after.noise_static.unwrap().0.public
        );
        assert_eq!(transport.0.lock().unwrap().len(), 2);
    }
}
