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

/// Load the signed prekey actually registered for this account. Never generates
/// or rotates identity; serialized key material must remain encrypted at rest.
pub fn registered_signed_prekey(store: &ProtocolStore) -> Result<Zeroizing<Vec<u8>>> {
    use wacore_libsignal::protocol::{
        GenericSignedPreKey, PrivateKey, SignedPreKeyRecord, Timestamp,
    };
    store.read(|tx| {
        let record: Material = tx
            .get_json(Table::Account, MATERIAL_KEY, RECORD_VERSION)?
            .ok_or_else(|| invalid("registered signed prekey unavailable"))?;
        record.check_account(AccountRecord::load(tx)?)?;
        let public = PublicKey::from_djb_public_key_bytes(&record.signed_pre_key.public)
            .map_err(|_| invalid("invalid signed prekey"))?;
        let identity = PublicKey::from_djb_public_key_bytes(&record.identity.public)
            .map_err(|_| invalid("invalid identity"))?;
        if !identity.verify_signature(&public.serialize(), &record.signature) {
            return Err(invalid("signed prekey signature invalid"));
        }
        let pair = KeyPair {
            public_key: public,
            private_key: PrivateKey::deserialize(&record.signed_pre_key.private)
                .map_err(|_| invalid("invalid signed private key"))?,
        };
        let signed = SignedPreKeyRecord::new(
            record.config.signed_pre_key_id.into(),
            Timestamp::from_epoch_millis(0),
            &pair,
            &record.signature,
        );
        Ok(Zeroizing::new(signed.serialize().map_err(|_| {
            invalid("cannot serialize saved signed prekey")
        })?))
    })
}

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

/// Test-only: persist generated registration material and a matching
/// registered account so transport tests can load the signed prekey.
#[cfg(test)]
#[cfg_attr(
    not(all(feature = "experimental-noise", feature = "experimental-transport")),
    allow(dead_code)
)]
pub(crate) fn install_test_material(
    store: &ProtocolStore,
    config: &LabRegistrationConfig,
) -> Result<()> {
    use crate::model::time::Timestamp;
    let attempt = RegistrationAttempt {
        id: AttemptId([3; 16]),
        generation: store.generation(),
        profile_id: LAB_PROFILE_ID.into(),
        phone: config.phone.clone(),
        method: ChallengeMethod::Sms,
        state: super::RegistrationState::Preparing,
        created_at: Timestamp::now(),
        updated_at: Timestamp::now(),
        history: Vec::new(),
    };
    let record = Material::generate(&attempt, config)?;
    store.write(|tx| {
        tx.put_json(Table::Account, MATERIAL_KEY, RECORD_VERSION, &record)?;
        AccountRecord {
            generation: attempt.generation,
            phone: Some(config.phone.clone()),
            own_jid: Some(crate::model::jid::Jid::parse("15555550100@s.whatsapp.net")?),
            noise_static: Some(NoiseStaticKey(record.noise.clone())),
            signal_identity: Some(SignalIdentityKey(record.identity.clone())),
            signal_registration_id: Some(config.registration_id),
            registration: crate::session::RegistrationStatus::Registered,
            ..AccountRecord::default()
        }
        .save(tx)
    })
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
    correct_token: bool,
    correct_legacy_request: bool,
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
        Ok(Self {
            store,
            config,
            correct_token: false,
            correct_legacy_request: false,
        })
    }

    /// Explicit token-only correction for `retry_corrected_token_once`.
    /// Preparation rejects an unchanged token or any other profile change.
    pub fn new_for_token_correction(
        store: Arc<ProtocolStore>,
        config: LabRegistrationConfig,
    ) -> Result<Self> {
        let mut factory = Self::new(store, config)?;
        factory.correct_token = true;
        Ok(factory)
    }

    /// Explicit legacy request migration, paired with the service's one-shot
    /// legacy-refusal action. Only token, User-Agent, body encoding and extra
    /// endpoint fields may change; identity and all other settings stay frozen.
    pub fn new_for_legacy_request_correction(
        store: Arc<ProtocolStore>,
        config: LabRegistrationConfig,
    ) -> Result<Self> {
        let mut factory = Self::new_for_token_correction(store, config)?;
        factory.correct_legacy_request = true;
        Ok(factory)
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

    fn code_request_with_method(
        &self,
        attempt: &RegistrationAttempt,
        method: &str,
    ) -> Result<EncodedRegistrationRequest> {
        let record = self.load(attempt)?;
        let mut form = RegistrationForm::new();
        record.common()?.append_to_form(&mut form)?;
        RegistrationCodeParams::new(record.config.token.as_str(), method)?
            .append_to_form(&mut form)?;
        super::request::append_fields(&mut form, &record.config.fields.code)?;
        super::request::append_byte_fields(&mut form, &record.config.fields.code_bytes)?;
        record.bundle()?.append_to_form(&mut form)?;
        WHATSAPP_2_26_37_70.request(RegistrationEndpoint::Code, &form)
    }
}

impl ComparisonRequestFactory for StoredComparisonRequestFactory {
    fn consent_request(
        &self,
        attempt: &RegistrationAttempt,
        decision: Option<&super::consent::ConsentDecision>,
    ) -> Result<EncodedRegistrationRequest> {
        let record = self.load(attempt)?;
        let mut form = RegistrationForm::new();
        record.common()?.append_to_form(&mut form)?;
        let endpoint = if let Some(value) = decision {
            value.validate()?;
            if let super::RegistrationState::Blocked {
                reason: super::BlockedReason::ConsentRequired { details, .. },
            } = &attempt.state
                && let Some(login) = &details.login
            {
                form.push("login", login.as_str())?;
            }
            if let Some(dob) = &value.dob {
                form.push("context", "dob")?;
                form.push("dob", dob.as_str())?;
            } else {
                form.push("context", "consent")?;
                form.push(
                    "consent_id",
                    value
                        .consent_id
                        .ok_or_else(|| invalid("missing consent ID"))?
                        .to_string(),
                )?;
                form.push(
                    "consent_version",
                    value
                        .consent_version
                        .ok_or_else(|| invalid("missing consent version"))?
                        .to_string(),
                )?;
                form.push(
                    "consent_result",
                    value
                        .consent_result
                        .ok_or_else(|| invalid("missing consent result"))?
                        .to_string(),
                )?;
            }
            RegistrationEndpoint::Consent
        } else {
            form.push("token", record.config.token.as_str())?;
            // A dedicated inspection does not reuse code/preflight telemetry.
            RegistrationEndpoint::Exist
        };
        record.bundle()?.append_to_form(&mut form)?;
        WHATSAPP_2_26_37_70.request(endpoint, &form)
    }
    fn preflight_request(
        &self,
        attempt: &RegistrationAttempt,
    ) -> Result<Option<EncodedRegistrationRequest>> {
        if !self.config.preflight {
            return Ok(None);
        }
        self.exist_request(attempt).map(Some)
    }

    /// The same-identity `/v2/exist` request, independent of the preflight
    /// opt-in (2.26.37.71 `KotlinRegistrationBridge.A0B` performSameDeviceCheck).
    fn exist_request(&self, attempt: &RegistrationAttempt) -> Result<EncodedRegistrationRequest> {
        let record = self.load(attempt)?;
        let mut form = RegistrationForm::new();
        record.common()?.append_to_form(&mut form)?;
        form.push("token", record.config.token.as_str())?;
        super::request::append_fields(&mut form, &record.config.fields.exist)?;
        let mut fields = record.config.fields.exist_bytes.clone();
        if let Some(metrics) = fields.get_mut("client_metrics") {
            let count = attempt
                .code_preparation_count()
                .ok_or_else(|| invalid("incomplete preflight preparation history"))?;
            let mut value: serde_json::Value = serde_json::from_slice(metrics)
                .map_err(|_| invalid("invalid preflight metrics JSON"))?;
            let object = value
                .as_object_mut()
                .ok_or_else(|| invalid("preflight metrics must be an object"))?;
            object.insert("attempts".into(), serde_json::json!(count));
            *metrics = Zeroizing::new(
                serde_json::to_vec(&value)
                    .map_err(|_| invalid("cannot encode preflight metrics"))?,
            );
        }
        super::request::append_byte_fields(&mut form, &fields)?;
        record.bundle()?.append_to_form(&mut form)?;
        WHATSAPP_2_26_37_70.request(RegistrationEndpoint::Exist, &form)
    }

    fn prepare(&self, tx: &mut WriteTx<'_>, attempt: &RegistrationAttempt) -> Result<()> {
        self.check_attempt(attempt)?;
        if tx.generation() != attempt.generation {
            return Err(invalid("stale account generation"));
        }
        if let Some(mut record) =
            tx.get_json::<Material>(Table::Account, MATERIAL_KEY, RECORD_VERSION)?
        {
            if self.correct_token {
                if record.config.token == self.config.token {
                    return Err(invalid("corrected token must differ from the saved token"));
                }
                record.config.token = self.config.token.clone();
                if self.correct_legacy_request {
                    record.config.client.user_agent = self.config.client.user_agent.clone();
                    record.config.transport.body_mode = self.config.transport.body_mode;
                    record.config.fields = self.config.fields.clone();
                }
                record.check(attempt, &self.config)?;
                record.check_account(AccountRecord::load_in_write(tx)?)?;
                return tx.put_json(Table::Account, MATERIAL_KEY, RECORD_VERSION, &record);
            }
            record.check(attempt, &self.config)?;
            return record.check_account(AccountRecord::load_in_write(tx)?);
        }
        if self.correct_token {
            return Err(invalid("token correction requires existing material"));
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
        let method = match attempt.method {
            ChallengeMethod::Sms => "sms",
            ChallengeMethod::Voice => "voice",
            _ => return Err(invalid("unsupported challenge method")),
        };
        self.code_request_with_method(attempt, method)
    }

    /// `/v2/code` with `method=email_otp` (2.26.37.71 `LX/0GB.A01` maps the
    /// email second factor to that delivery method; the reply carries
    /// `email_otp_wait`/`email_otp_length`).
    fn email_code_request(
        &self,
        attempt: &RegistrationAttempt,
    ) -> Result<EncodedRegistrationRequest> {
        self.code_request_with_method(attempt, "email_otp")
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
        super::request::append_byte_fields(&mut form, &record.config.fields.register_bytes)?;
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
        super::request::append_byte_fields(&mut form, &record.config.fields.security_bytes)?;
        record.bundle()?.append_to_form(&mut form)?;
        WHATSAPP_2_26_37_70.request(RegistrationEndpoint::Security, &form)
    }

    /// `/v2/challenge` (2.26.37.71 `KotlinRegistrationBridge.A0I`): the shared
    /// identity parameters, the challenge fields, configured extras and the
    /// same key bundle the register request carries ("e2e keys not available"
    /// aborts the client's request).
    fn challenge_request(
        &self,
        attempt: &RegistrationAttempt,
        params: &RegistrationChallengeParams,
    ) -> Result<EncodedRegistrationRequest> {
        let record = self.load(attempt)?;
        let mut form = RegistrationForm::new();
        record.common()?.append_to_form(&mut form)?;
        params.append_to_form(&mut form)?;
        super::request::append_fields(&mut form, &record.config.fields.challenge)?;
        record.bundle()?.append_to_form(&mut form)?;
        WHATSAPP_2_26_37_70.request(RegistrationEndpoint::Challenge, &form)
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
            preflight: false,
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
            .code_bytes
            .insert("client_metrics".into(), Zeroizing::new(b"test ~*".to_vec()));
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
        assert!(body.contains("client_metrics=test%20~%2A"));
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

    /// Every request fails before leaving (a connection refused before writing).
    #[cfg(feature = "experimental-registration-https")]
    #[derive(Debug)]
    struct UnsentTransport(std::sync::Mutex<u32>);

    #[cfg(feature = "experimental-registration-https")]
    impl RegistrationTransport for UnsentTransport {
        fn send<'a>(&'a self, _: EncodedRegistrationRequest) -> RegistrationTransportFuture<'a> {
            Box::pin(async move {
                *self.0.lock().unwrap() += 1;
                Err(Error::RequestNotSent(Box::new(Error::Io(
                    std::io::Error::other("connection refused before writing"),
                ))))
            })
        }
    }

    /// A corrected-token retry that never left restores the `bad_token`
    /// refusal, but its one-shot stays spent: the reservation already saved the
    /// corrected token, so the same correction could not be repeated anyway.
    #[tokio::test]
    #[cfg(feature = "experimental-registration-https")]
    async fn a_not_sent_correction_restores_the_refusal_and_keeps_its_one_shot() {
        use crate::registration::BlockedReason;
        let dir = tempfile::tempdir().unwrap();
        let store = open(dir.path());
        let original = StoredComparisonRequestFactory::new(store.clone(), config()).unwrap();
        let mut a = attempt(&store);
        store.write(|tx| original.prepare(tx, &a)).unwrap();
        let refusal = RegistrationState::Blocked {
            reason: BlockedReason::ServerRefused {
                code: None,
                http_status: Some(200),
                reason: Some(RegistrationResponseReason::BadToken),
            },
        };
        a.transition(refusal.clone(), Timestamp::now()).unwrap();
        store
            .write(|tx| RegistrationService::save_current(tx, &a))
            .unwrap();
        let mut profile = crate::ProtocolProfile::android_reference().unwrap();
        profile.id = LAB_PROFILE_ID.into();
        let transport = Arc::new(UnsentTransport(std::sync::Mutex::new(0)));
        let mut corrected = config();
        corrected.token = Zeroizing::new("corrected-token".into());
        let service = RegistrationService::new(
            store.clone(),
            &profile,
            Arc::new(ComparisonHttpFlow::new(
                Arc::new(
                    StoredComparisonRequestFactory::new_for_token_correction(
                        store.clone(),
                        corrected,
                    )
                    .unwrap(),
                ),
                transport.clone(),
            )),
        );
        let error = service
            .retry_corrected_token_once(config().phone)
            .await
            .unwrap_err();
        assert!(matches!(error, Error::RequestNotSent(_)), "{error}");
        let saved = service.current().await.unwrap().unwrap();
        assert_eq!(saved.state, refusal);
        assert!(saved.refused_at_code_request());
        let again = service
            .retry_corrected_token_once(config().phone)
            .await
            .unwrap_err()
            .to_string();
        assert!(again.contains("one-shot"), "{again}");
        assert_eq!(
            *transport.0.lock().unwrap(),
            1,
            "one attempt, which never left"
        );
    }

    /// The same saved refusal shape reached from an OTP submission (`verifying
    /// -> blocked`) must not make a code-request diagnostic or the corrected
    /// token resend eligible: either would ask for a new SMS or call.
    #[tokio::test]
    #[cfg(feature = "experimental-registration-https")]
    async fn code_request_actions_refuse_a_refusal_from_another_request() {
        use crate::registration::BlockedReason;
        let dir = tempfile::tempdir().unwrap();
        let store = open(dir.path());
        let factory = StoredComparisonRequestFactory::new(store.clone(), config()).unwrap();
        let mut profile = crate::ProtocolProfile::android_reference().unwrap();
        profile.id = LAB_PROFILE_ID.into();
        let transport = Arc::new(MockTransport::default());
        for reason in [
            RegistrationResponseReason::BadToken,
            RegistrationResponseReason::BadParam,
        ] {
            let mut a = attempt(&store);
            store.write(|tx| factory.prepare(tx, &a)).unwrap();
            a.state = RegistrationState::Verifying { step: 1 };
            a.transition(
                RegistrationState::Blocked {
                    reason: BlockedReason::ServerRefused {
                        code: None,
                        http_status: Some(200),
                        reason: Some(reason),
                    },
                },
                Timestamp::now(),
            )
            .unwrap();
            assert!(!a.refused_at_code_request());
            store
                .write(|tx| RegistrationService::save_current(tx, &a))
                .unwrap();
            let mut corrected = config();
            corrected.token = Zeroizing::new("corrected-token".into());
            let correction = RegistrationService::new(
                store.clone(),
                &profile,
                Arc::new(ComparisonHttpFlow::new(
                    Arc::new(
                        StoredComparisonRequestFactory::new_for_token_correction(
                            store.clone(),
                            corrected,
                        )
                        .unwrap(),
                    ),
                    transport.clone(),
                )),
            );
            let error = correction
                .retry_corrected_token_once(config().phone)
                .await
                .unwrap_err()
                .to_string();
            assert!(
                error.contains("eligible") || error.contains("did not come from a code request"),
                "{error}"
            );
            let diagnostic = RegistrationService::new(
                store.clone(),
                &profile,
                Arc::new(ComparisonHttpFlow::new(
                    Arc::new(StoredComparisonRequestFactory::new(store.clone(), config()).unwrap()),
                    transport.clone(),
                )),
            );
            assert!(
                diagnostic
                    .diagnose_bad_parameter_once(config().phone)
                    .await
                    .is_err()
            );
        }
        assert!(transport.0.lock().unwrap().is_empty(), "nothing was sent");
    }

    #[tokio::test]
    #[cfg(feature = "experimental-registration-https")]
    async fn token_correction_is_atomic_token_only_and_once_per_account() {
        use crate::registration::BlockedReason;
        let dir = tempfile::tempdir().unwrap();
        let store = open(dir.path());
        let original_factory =
            StoredComparisonRequestFactory::new(store.clone(), config()).unwrap();
        let mut a = attempt(&store);
        store.write(|tx| original_factory.prepare(tx, &a)).unwrap();
        let original = original_factory.code_request(&a).unwrap().body().to_vec();
        a.transition(
            RegistrationState::Blocked {
                reason: BlockedReason::ServerRefused {
                    code: None,
                    http_status: Some(200),
                    reason: Some(RegistrationResponseReason::BadToken),
                },
            },
            Timestamp::now(),
        )
        .unwrap();
        store
            .write(|tx| RegistrationService::save_current(tx, &a))
            .unwrap();
        let mut profile = crate::ProtocolProfile::android_reference().unwrap();
        profile.id = LAB_PROFILE_ID.into();
        let transport = Arc::new(MockTransport::default());
        for unrelated_change in [false, true] {
            let mut cfg = config();
            if unrelated_change {
                cfg.token = Zeroizing::new("corrected-token".into());
                cfg.locale = "CA".into();
            }
            let factory = Arc::new(
                StoredComparisonRequestFactory::new_for_token_correction(store.clone(), cfg)
                    .unwrap(),
            );
            let service = RegistrationService::new(
                store.clone(),
                &profile,
                Arc::new(ComparisonHttpFlow::new(factory, transport.clone())),
            );
            assert!(
                service
                    .retry_corrected_token_once(config().phone)
                    .await
                    .is_err()
            );
            assert_eq!(original_factory.code_request(&a).unwrap().body(), original);
            assert!(transport.0.lock().unwrap().is_empty());
        }
        let mut cfg = config();
        cfg.token = Zeroizing::new("corrected-token".into());
        let factory = Arc::new(
            StoredComparisonRequestFactory::new_for_token_correction(store.clone(), cfg).unwrap(),
        );
        let service = RegistrationService::new(
            store.clone(),
            &profile,
            Arc::new(ComparisonHttpFlow::new(factory.clone(), transport.clone())),
        );
        assert!(
            service
                .retry_corrected_token_once(PhoneNumber::new(1, "5557654321").unwrap())
                .await
                .is_err()
        );
        service
            .retry_corrected_token_once(config().phone)
            .await
            .unwrap();
        let saved = service.current().await.unwrap().unwrap();
        assert_eq!(saved.id, a.id);
        assert_eq!(saved.generation, a.generation);
        assert!(saved.history_len() > a.history_len());
        let updated = factory.code_request(&saved).unwrap();
        assert_eq!(
            String::from_utf8(original)
                .unwrap()
                .replace("token=offline-test-token-only", "token=corrected-token"),
            std::str::from_utf8(updated.body()).unwrap()
        );
        assert_eq!(transport.0.lock().unwrap().len(), 1);
        store
            .write(|tx| RegistrationService::save_current(tx, &a))
            .unwrap();
        assert!(
            service
                .retry_corrected_token_once(config().phone)
                .await
                .is_err()
        );
        assert_eq!(transport.0.lock().unwrap().len(), 1);
    }

    #[test]
    fn legacy_request_correction_preserves_identity_and_rejects_unrelated_edits() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(dir.path());
        let a = attempt(&store);
        let original = StoredComparisonRequestFactory::new(store.clone(), config()).unwrap();
        store.write(|tx| original.prepare(tx, &a)).unwrap();
        let before = original.code_request(&a).unwrap().body().to_vec();
        let mut cfg = config();
        cfg.token = Zeroizing::new("corrected-token".into());
        cfg.client.user_agent = Some(Zeroizing::new("test-agent".into()));
        cfg.transport.body_mode = super::super::request::RequestBodyMode::EncEmptyH;
        cfg.fields
            .code_bytes
            .insert("mcc".into(), Zeroizing::new(b"000".to_vec()));
        for field in ["registration_id", "signed_pre_key_id", "locale", "timeout"] {
            let mut invalid = cfg.clone();
            match field {
                "registration_id" => invalid.registration_id += 1,
                "signed_pre_key_id" => invalid.signed_pre_key_id += 1,
                "locale" => invalid.locale = "CA".into(),
                _ => invalid.transport.timeout_ms += 1,
            }
            let factory = StoredComparisonRequestFactory::new_for_legacy_request_correction(
                store.clone(),
                invalid,
            )
            .unwrap();
            assert!(store.write(|tx| factory.prepare(tx, &a)).is_err());
            assert_eq!(original.code_request(&a).unwrap().body(), before);
        }
        let factory =
            StoredComparisonRequestFactory::new_for_legacy_request_correction(store.clone(), cfg)
                .unwrap();
        store.write(|tx| factory.prepare(tx, &a)).unwrap();
        let after = factory.code_request(&a).unwrap();
        let expected = String::from_utf8(before)
            .unwrap()
            .replace("token=offline-test-token-only", "token=corrected-token")
            .replace("&method=sms", "&method=sms&mcc=000");
        assert_eq!(std::str::from_utf8(after.body()).unwrap(), expected);
        assert!(store.write(|tx| factory.prepare(tx, &a)).is_err());
    }

    #[test]
    fn preflight_uses_saved_identity_and_separate_json_fields() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(dir.path());
        let a = attempt(&store);
        let mut cfg = config();
        cfg.preflight = true;
        cfg.fields.exist_bytes.insert(
            "client_metrics".into(),
            Zeroizing::new(br#"{"attempts":1}"#.to_vec()),
        );
        cfg.fields
            .code_bytes
            .insert("reason".into(), Zeroizing::new(Vec::new()));
        let factory = StoredComparisonRequestFactory::new(store.clone(), cfg).unwrap();
        store.write(|tx| factory.prepare(tx, &a)).unwrap();
        let exist = factory.preflight_request(&a).unwrap().unwrap();
        let code = factory.code_request(&a).unwrap();
        assert!(exist.url().ends_with("/v2/exist"));
        let exist_body = std::str::from_utf8(exist.body()).unwrap();
        let code_body = std::str::from_utf8(code.body()).unwrap();
        assert!(!exist_body.contains("method="));
        assert!(!exist_body.contains("reason="));
        assert!(exist_body.contains("client_metrics="));
        assert!(!code_body.contains("client_metrics="));
        for key in [
            "cc=",
            "in=",
            "token=",
            "fdid=",
            "authkey=",
            "e_ident=",
            "e_skey_sig=",
        ] {
            assert_eq!(
                exist_body.split('&').find(|v| v.starts_with(key)),
                code_body.split('&').find(|v| v.starts_with(key))
            );
        }
    }

    #[derive(Debug)]
    struct PreflightTransport {
        response: &'static str,
        paths: std::sync::Mutex<Vec<String>>,
    }
    impl RegistrationTransport for PreflightTransport {
        fn send<'a>(
            &'a self,
            request: EncodedRegistrationRequest,
        ) -> RegistrationTransportFuture<'a> {
            Box::pin(async move {
                self.paths.lock().unwrap().push(request.url().to_owned());
                let body = if request.url().ends_with("/v2/exist") {
                    self.response
                } else {
                    r#"{"status":"sent"}"#
                };
                RegistrationHttpResponse::new(200, Zeroizing::new(body.as_bytes().to_vec()))
            })
        }
    }

    #[tokio::test]
    async fn dob_consent_reuses_login_and_does_not_complete_pending_parent_flow() {
        use crate::registration::{BlockedReason, consent::ConsentDecision};
        #[derive(Debug)]
        struct DobTransport;
        impl RegistrationTransport for DobTransport {
            fn send<'a>(
                &'a self,
                request: EncodedRegistrationRequest,
            ) -> RegistrationTransportFuture<'a> {
                Box::pin(async move {
                    assert!(request.url().ends_with("/v2/consent"));
                    let body = std::str::from_utf8(request.body()).unwrap();
                    for field in [
                        "context=dob",
                        "dob=2000-02-29",
                        "login=intermediate-login",
                        "e_ident=",
                    ] {
                        assert!(body.contains(field));
                    }
                    assert!(!body.contains("consent_result="));
                    RegistrationHttpResponse::new(200, Zeroizing::new(br#"{"status":"ok","pending":"parent_verification","parent_consent_url":"https://example.invalid/private"}"#.to_vec()))
                })
            }
        }
        let dir = tempfile::tempdir().unwrap();
        let store = open(dir.path());
        let factory =
            Arc::new(StoredComparisonRequestFactory::new(store.clone(), config()).unwrap());
        let mut a = attempt(&store);
        store.write(|tx| factory.prepare(tx, &a)).unwrap();
        let details = RegistrationHttpResponse::new(200, Zeroizing::new(br#"{"status":"fail","reason":"consent","pending":"dob","login":"intermediate-login"}"#.to_vec())).unwrap().parse_json().unwrap().consent;
        a.transition(
            RegistrationState::Blocked {
                reason: BlockedReason::ConsentRequired {
                    details,
                    retry_after: None,
                    sms_retry_after: None,
                    voice_retry_after: None,
                },
            },
            Timestamp::now(),
        )
        .unwrap();
        store
            .write(|tx| RegistrationService::save_current(tx, &a))
            .unwrap();
        let mut profile = crate::ProtocolProfile::android_reference().unwrap();
        profile.id = LAB_PROFILE_ID.into();
        let service = RegistrationService::new(
            store.clone(),
            &profile,
            Arc::new(ComparisonHttpFlow::new(factory, Arc::new(DobTransport))),
        );
        let decision: ConsentDecision =
            serde_json::from_str(r#"{"dob":"2000-02-29","confirmed":true}"#).unwrap();
        let result = service
            .continue_consent(config().phone, Some(decision))
            .await
            .unwrap();
        assert!(matches!(
            result,
            RegistrationStep::Blocked(BlockedReason::ConsentRequired { .. })
        ));
        assert_ne!(
            store
                .read(AccountRecord::load)
                .unwrap()
                .unwrap()
                .registration,
            crate::session::RegistrationStatus::Registered
        );
        assert!(!format!("{result:?}").contains("https://"));
    }

    #[tokio::test]
    async fn consent_inspection_and_submission_are_bound_and_once_only() {
        use crate::registration::{BlockedReason, consent::ConsentDecision};
        #[derive(Debug, Default)]
        struct ConsentTransport(std::sync::Mutex<Vec<String>>);
        impl RegistrationTransport for ConsentTransport {
            fn send<'a>(
                &'a self,
                request: EncodedRegistrationRequest,
            ) -> RegistrationTransportFuture<'a> {
                Box::pin(async move {
                    self.0.lock().unwrap().push(request.url().into());
                    let body = if request.url().ends_with("/v2/exist") {
                        r#"{"status":"fail","reason":"consent","consent_id":3,"consent_version":2,"pending":"youth_consent"}"#
                    } else {
                        assert!(request.url().ends_with("/v2/consent"));
                        let body = std::str::from_utf8(request.body()).unwrap();
                        for field in [
                            "context=consent",
                            "consent_id=3",
                            "consent_version=2",
                            "consent_result=5",
                        ] {
                            assert!(body.split('&').any(|v| v == field));
                        }
                        assert!(!body.contains("code="));
                        r#"{"status":"ok","login":"consent-test-login"}"#
                    };
                    RegistrationHttpResponse::new(200, Zeroizing::new(body.as_bytes().to_vec()))
                })
            }
        }
        let dir = tempfile::tempdir().unwrap();
        let store = open(dir.path());
        let factory =
            Arc::new(StoredComparisonRequestFactory::new(store.clone(), config()).unwrap());
        let mut a = attempt(&store);
        store.write(|tx| factory.prepare(tx, &a)).unwrap();
        let original = factory.code_request(&a).unwrap().body().to_vec();
        a.transition(
            RegistrationState::Blocked {
                reason: BlockedReason::ServerRefused {
                    code: None,
                    http_status: Some(200),
                    reason: Some(RegistrationResponseReason::Consent),
                },
            },
            Timestamp::now(),
        )
        .unwrap();
        store
            .write(|tx| RegistrationService::save_current(tx, &a))
            .unwrap();
        let mut profile = crate::ProtocolProfile::android_reference().unwrap();
        profile.id = LAB_PROFILE_ID.into();
        let transport = Arc::new(ConsentTransport::default());
        let service = RegistrationService::new(
            store.clone(),
            &profile,
            Arc::new(ComparisonHttpFlow::new(factory.clone(), transport.clone())),
        );
        let decision = ConsentDecision {
            consent_id: Some(3),
            consent_version: Some(2),
            consent_result: Some(5),
            dob: None,
            confirmed: true,
        };
        assert!(
            service
                .continue_consent(config().phone, Some(decision.clone()))
                .await
                .is_err()
        );
        assert!(
            service
                .continue_consent(PhoneNumber::new(1, "5557654321").unwrap(), None)
                .await
                .is_err()
        );
        let (first, second) = tokio::join!(
            service.continue_consent(config().phone, None),
            service.continue_consent(config().phone, None)
        );
        assert_eq!(usize::from(first.is_ok()) + usize::from(second.is_ok()), 1);
        let saved = service.current().await.unwrap().unwrap();
        assert_eq!(saved.id, a.id);
        assert_eq!(saved.generation, a.generation);
        assert_eq!(original, factory.code_request(&saved).unwrap().body());
        assert!(
            service
                .continue_consent(
                    config().phone,
                    Some(ConsentDecision {
                        consent_version: Some(9),
                        ..decision.clone()
                    })
                )
                .await
                .is_err()
        );
        assert!(
            service
                .continue_consent(
                    config().phone,
                    Some(ConsentDecision {
                        confirmed: false,
                        ..decision.clone()
                    })
                )
                .await
                .is_err()
        );
        let mut waiting = saved.clone();
        if let RegistrationState::Blocked {
            reason: BlockedReason::ConsentRequired { retry_after, .. },
        } = &mut waiting.state
        {
            *retry_after =
                Some(Timestamp::now().saturating_add(std::time::Duration::from_secs(3600)));
        } else {
            panic!("expected saved consent context");
        }
        store
            .write(|tx| RegistrationService::save_current(tx, &waiting))
            .unwrap();
        assert!(
            service
                .continue_consent(config().phone, Some(decision.clone()))
                .await
                .is_err()
        );
        store
            .write(|tx| RegistrationService::save_current(tx, &saved))
            .unwrap();
        assert_eq!(
            service
                .continue_consent(config().phone, Some(decision.clone()))
                .await
                .unwrap(),
            RegistrationStep::Completed
        );
        assert!(
            service
                .continue_consent(config().phone, Some(decision.clone()))
                .await
                .is_err()
        );
        store
            .write(|tx| RegistrationService::save_current(tx, &saved))
            .unwrap();
        assert!(
            service
                .continue_consent(config().phone, Some(decision))
                .await
                .is_err()
        );
        assert_eq!(transport.0.lock().unwrap().len(), 2);
        let account = store.read(AccountRecord::load).unwrap().unwrap();
        assert_eq!(
            account.credentials[0].source_step.as_deref(),
            Some("/v2/consent")
        );
    }

    #[tokio::test]
    async fn preflight_only_sends_code_after_explicit_compatible_response() {
        for (response, sends_code) in [
            (
                r#"{"status":"fail","reason":"incorrect","recommended_method":["sms"],"sms_wait":0}"#,
                true,
            ),
            (r#"{"status":"ok","recommended_method":["sms"]}"#, false),
            (r#"{"status":"fail","reason":"incorrect"}"#, false),
            (
                r#"{"status":"fail","reason":"incorrect","fallback_methods":["sms","voice"],"sms_wait":0}"#,
                true,
            ),
            (
                r#"{"status":"fail","reason":"incorrect","fallback_methods":["voice"]}"#,
                false,
            ),
            (
                r#"{"status":"fail","reason":"incorrect","recommended_method":[],"fallback_methods":["sms"]}"#,
                false,
            ),
            (
                r#"{"status":"fail","reason":"incorrect","fallback_methods":["sms"],"sms_wait":600}"#,
                false,
            ),
            (
                r#"{"status":"fail","reason":"incorrect","fallback_methods":["sms"],"verify_pn_device":1}"#,
                false,
            ),
            (
                r#"{"status":"fail","reason":"incorrect","recommended_method":["voice"],"fallback_methods":["sms"]}"#,
                false,
            ),
            (
                r#"{"status":"fail","reason":"incorrect","recommended_method":["sms"],"sms_wait":600}"#,
                false,
            ),
            (
                r#"{"status":"fail","reason":"incorrect","recommended_method":["sms"],"retry_after":600,"sms_wait":0}"#,
                false,
            ),
            (
                r#"{"status":"fail","reason":"incorrect","recommended_method":["sms"],"verify_pn_device":1}"#,
                false,
            ),
            (
                r#"{"status":"fail","reason":"device_confirm_or_second_code","recommended_method":["sms"]}"#,
                false,
            ),
            (
                r#"{"status":"fail","reason":"incorrect","recommended_method":["sms"],"security_code_set":true}"#,
                false,
            ),
            ("malformed", false),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let store = open(dir.path());
            let mut cfg = config();
            cfg.preflight = true;
            let factory =
                Arc::new(StoredComparisonRequestFactory::new(store.clone(), cfg).unwrap());
            let transport = Arc::new(PreflightTransport {
                response,
                paths: Default::default(),
            });
            let mut profile = crate::ProtocolProfile::android_reference().unwrap();
            profile.id = LAB_PROFILE_ID.into();
            let service = RegistrationService::new(
                store.clone(),
                &profile,
                Arc::new(ComparisonHttpFlow::new(factory, transport.clone())),
            );
            let result = service.begin(config().phone, ChallengeMethod::Sms).await;
            if sends_code {
                assert!(matches!(
                    result.unwrap(),
                    RegistrationStep::Action(RequiredAction::EnterCode(_))
                ));
            }
            let expected = if sends_code { 2 } else { 1 };
            assert_eq!(transport.paths.lock().unwrap().len(), expected);
            assert!(transport.paths.lock().unwrap()[0].ends_with("/v2/exist"));
            let _ = service.begin(config().phone, ChallengeMethod::Sms).await;
            assert_eq!(transport.paths.lock().unwrap().len(), expected);
        }
    }

    #[tokio::test]
    #[cfg(feature = "experimental-registration-https")]
    async fn preflight_resume_preserves_identity_and_is_once_only() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(dir.path());
        let mut cfg = config();
        cfg.preflight = true;
        let factory = Arc::new(StoredComparisonRequestFactory::new(store.clone(), cfg).unwrap());
        let mut a = attempt(&store);
        store.write(|tx| factory.prepare(tx, &a)).unwrap();
        let original = factory.code_request(&a).unwrap().body().to_vec();
        a.transition(RegistrationState::Blocked {
            reason: crate::registration::BlockedReason::UnsupportedStep {
                description: "preflight requires explicit handling; HTTP=200, status=Fail, reason=Some(Incorrect), recommended=None, fallback=Some([Sms]), device_branch=None; no code request sent".into(),
            },
        }, Timestamp::now()).unwrap();
        store
            .write(|tx| RegistrationService::save_current(tx, &a))
            .unwrap();
        let transport = Arc::new(PreflightTransport {
            response: r#"{"status":"fail","reason":"incorrect","fallback_methods":["sms"],"sms_wait":0}"#,
            paths: Default::default(),
        });
        let mut profile = crate::ProtocolProfile::android_reference().unwrap();
        profile.id = LAB_PROFILE_ID.into();
        let service = RegistrationService::new(
            store.clone(),
            &profile,
            Arc::new(ComparisonHttpFlow::new(factory.clone(), transport.clone())),
        );
        assert!(
            service
                .resume_preflight_once(PhoneNumber::new(1, "5557654321").unwrap())
                .await
                .is_err()
        );
        assert!(transport.paths.lock().unwrap().is_empty());
        assert!(matches!(
            service.resume_preflight_once(config().phone).await.unwrap(),
            RegistrationStep::Action(RequiredAction::EnterCode(_))
        ));
        let saved = service.current().await.unwrap().unwrap();
        assert_eq!(saved.id, a.id);
        assert_eq!(saved.generation, a.generation);
        assert_eq!(original, factory.code_request(&saved).unwrap().body());
        assert!(service.resume_preflight_once(config().phone).await.is_err());
        store
            .write(|tx| RegistrationService::save_current(tx, &a))
            .unwrap();
        assert!(service.resume_preflight_once(config().phone).await.is_err());
        assert_eq!(transport.paths.lock().unwrap().len(), 2);
    }

    #[tokio::test]
    #[cfg(feature = "experimental-registration-https")]
    async fn diagnostics_are_once_only_and_preserve_identity_and_body() {
        for (bad_parameter, latch) in [
            (false, b"legacy-refusal-diagnostic-used".as_slice()),
            (true, b"bad-parameter-diagnostic-used".as_slice()),
            (true, b"encrypted-parameter-diagnostic-used".as_slice()),
            (true, b"unsigned-envelope-diagnostic-used".as_slice()),
            (true, b"additional-fields-diagnostic-used".as_slice()),
            (true, b"flow-fields-diagnostic-used".as_slice()),
            (true, b"unknown-agent-diagnostic-used".as_slice()),
        ] {
            use crate::registration::BlockedReason;
            let dir = tempfile::tempdir().unwrap();
            let store = open(dir.path());
            let factory =
                Arc::new(StoredComparisonRequestFactory::new(store.clone(), config()).unwrap());
            let mut a = attempt(&store);
            store.write(|tx| factory.prepare(tx, &a)).unwrap();
            let original = factory.code_request(&a).unwrap().body().to_vec();
            a.transition(
                RegistrationState::Blocked {
                    reason: BlockedReason::ServerRefused {
                        code: None,
                        http_status: bad_parameter.then_some(200),
                        reason: bad_parameter.then_some(RegistrationResponseReason::BadParam),
                    },
                },
                Timestamp::now(),
            )
            .unwrap();
            store
                .write(|tx| RegistrationService::save_current(tx, &a))
                .unwrap();
            let transport = Arc::new(MockTransport::default());
            let mut profile = crate::ProtocolProfile::android_reference().unwrap();
            profile.id = LAB_PROFILE_ID.into();
            let service = RegistrationService::new(
                store.clone(),
                &profile,
                Arc::new(ComparisonHttpFlow::new(factory.clone(), transport.clone())),
            );
            assert!(
                service
                    .diagnose_refusal_with_latch(
                        PhoneNumber::new(1, "5557654321").unwrap(),
                        bad_parameter,
                        latch
                    )
                    .await
                    .is_err()
            );
            assert!(transport.0.lock().unwrap().is_empty());
            assert!(matches!(
                service
                    .diagnose_refusal_with_latch(config().phone, bad_parameter, latch)
                    .await
                    .unwrap(),
                RegistrationStep::Action(RequiredAction::EnterCode(_))
            ));
            let saved = service.current().await.unwrap().unwrap();
            assert_eq!(saved.id, a.id);
            assert_eq!(saved.generation, a.generation);
            assert!(saved.history_len() > a.history_len());
            assert_eq!(saved.code_preparation_count(), Some(1));
            let mut truncated = saved.clone();
            truncated
                .history
                .resize(super::super::MAX_HISTORY, truncated.history[0].clone());
            assert_eq!(truncated.code_preparation_count(), None);
            assert_eq!(original, factory.code_request(&saved).unwrap().body());
            assert!(
                service
                    .diagnose_refusal_with_latch(config().phone, bad_parameter, latch)
                    .await
                    .is_err()
            );
            // Even restoring an old eligible state cannot defeat the committed latch.
            store
                .write(|tx| RegistrationService::save_current(tx, &a))
                .unwrap();
            let resumed = RegistrationService::new(
                store,
                &profile,
                Arc::new(ComparisonHttpFlow::new(factory, transport.clone())),
            );
            assert!(
                resumed
                    .diagnose_refusal_with_latch(config().phone, bad_parameter, latch)
                    .await
                    .is_err()
            );
            assert_eq!(transport.0.lock().unwrap().len(), 1);
        }
    }

    #[tokio::test]
    #[cfg(feature = "experimental-registration-https")]
    async fn legacy_diagnostic_rejects_known_refusals_cooldowns_and_unknown_outcomes() {
        use crate::registration::BlockedReason;
        let dir = tempfile::tempdir().unwrap();
        let store = open(dir.path());
        let factory =
            Arc::new(StoredComparisonRequestFactory::new(store.clone(), config()).unwrap());
        let mut a = attempt(&store);
        store.write(|tx| factory.prepare(tx, &a)).unwrap();
        let transport = Arc::new(MockTransport::default());
        let mut profile = crate::ProtocolProfile::android_reference().unwrap();
        profile.id = LAB_PROFILE_ID.into();
        let service = RegistrationService::new(
            store.clone(),
            &profile,
            Arc::new(ComparisonHttpFlow::new(factory, transport.clone())),
        );
        for state in [
            RegistrationState::Blocked {
                reason: BlockedReason::ServerRefused {
                    code: None,
                    http_status: Some(200),
                    reason: Some(RegistrationResponseReason::FormatWrong),
                },
            },
            RegistrationState::Blocked {
                reason: BlockedReason::ServerRefused {
                    code: None,
                    http_status: Some(200),
                    reason: Some(RegistrationResponseReason::Blocked),
                },
            },
            RegistrationState::Blocked {
                reason: BlockedReason::ServerRefused {
                    code: Some(403),
                    http_status: None,
                    reason: None,
                },
            },
            RegistrationState::CoolingDown {
                until: Timestamp::now(),
            },
            RegistrationState::RegistrationOutcomeUnknown { step: 0 },
            RegistrationState::Registered,
        ] {
            a.state = state;
            store
                .write(|tx| RegistrationService::save_current(tx, &a))
                .unwrap();
            assert!(service.resume_preflight_once(config().phone).await.is_err());
            assert!(
                service
                    .retry_corrected_token_once(config().phone)
                    .await
                    .is_err()
            );
            assert!(
                service
                    .diagnose_bad_parameter_once(config().phone)
                    .await
                    .is_err()
            );
            assert!(
                service
                    .diagnose_encrypted_parameter_once(config().phone)
                    .await
                    .is_err()
            );
            assert!(
                service
                    .diagnose_unsigned_envelope_once(config().phone)
                    .await
                    .is_err()
            );
            assert!(
                service
                    .diagnose_additional_fields_once(config().phone)
                    .await
                    .is_err()
            );
            assert!(
                service
                    .diagnose_flow_fields_once(config().phone)
                    .await
                    .is_err()
            );
            assert!(
                service
                    .diagnose_unknown_agent_once(config().phone)
                    .await
                    .is_err()
            );
            assert!(
                service
                    .diagnose_legacy_refusal_once(config().phone)
                    .await
                    .is_err()
            );
        }
        assert!(transport.0.lock().unwrap().is_empty());
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
