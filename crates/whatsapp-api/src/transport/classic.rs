//! APK classical XX sequencing over existing cryptographic primitives.
//! Unlike standard Noise message APIs, LX/1NZ.A06 hashes only the first
//! ephemeral: no empty EncryptAndHash payload follows ClientHello.

use super::{
    ConnectionEpoch, TransportError,
    comparison::COMPARISON_CONNECTION_HEADER,
    noise::{MAX_RECORD_LEN, StaticKeypair},
};
use hkdf::Hkdf;
use sha2::{Digest, Sha256};
use snow::{
    params::{CipherChoice, DHChoice},
    resolvers::{CryptoResolver, DefaultResolver},
    types::Cipher,
};
use zeroize::Zeroizing;

fn error() -> TransportError {
    TransportError::Noise("comparison classical cryptographic operation failed".into())
}

fn cipher(key: &[u8; 32]) -> Result<Box<dyn Cipher>, TransportError> {
    let mut cipher = DefaultResolver
        .resolve_cipher(&CipherChoice::AESGCM)
        .ok_or_else(error)?;
    cipher.set(key);
    Ok(cipher)
}

fn dh(private: &[u8; 32], public: &[u8]) -> Result<Zeroizing<[u8; 32]>, TransportError> {
    if public.len() != 32 {
        return Err(error());
    }
    let mut dh = DefaultResolver
        .resolve_dh(&DHChoice::Curve25519)
        .ok_or_else(error)?;
    dh.set(private);
    let mut shared = Zeroizing::new([0; 32]);
    dh.dh(public, shared.as_mut()).map_err(|_| error())?;
    if shared.iter().all(|b| *b == 0) {
        return Err(error());
    }
    Ok(shared)
}

struct Symmetric {
    hash: [u8; 32],
    chain: Zeroizing<[u8; 32]>,
    key: Zeroizing<[u8; 32]>,
    nonce: u64,
}

impl Symmetric {
    fn new() -> Self {
        let mut initial = [0; 32];
        let name = b"Noise_XX_25519_AESGCM_SHA256";
        initial[..name.len()].copy_from_slice(name);
        let mut state = Self {
            hash: initial,
            chain: Zeroizing::new(initial),
            key: Zeroizing::new([0; 32]),
            nonce: 0,
        };
        state.mix_hash(&COMPARISON_CONNECTION_HEADER);
        state
    }
    fn mix_hash(&mut self, bytes: &[u8]) {
        let mut digest = Sha256::new();
        digest.update(self.hash);
        digest.update(bytes);
        self.hash = digest.finalize().into();
    }
    fn derive(&self, input: &[u8]) -> Result<Zeroizing<[u8; 64]>, TransportError> {
        let mut out = Zeroizing::new([0; 64]);
        Hkdf::<Sha256>::new(Some(self.chain.as_ref()), input)
            .expand(&[], out.as_mut())
            .map_err(|_| error())?;
        Ok(out)
    }
    fn mix_key(&mut self, shared: &[u8]) -> Result<(), TransportError> {
        let out = self.derive(shared)?;
        self.chain.copy_from_slice(&out[..32]);
        self.key.copy_from_slice(&out[32..]);
        self.nonce = 0;
        Ok(())
    }
    fn decrypt(&mut self, record: &[u8]) -> Result<Zeroizing<Vec<u8>>, TransportError> {
        if record.len() < 16 || record.len() > MAX_RECORD_LEN {
            return Err(error());
        }
        let mut out = Zeroizing::new(vec![0; record.len()]);
        let n = cipher(&self.key)?
            .decrypt(self.nonce, &self.hash, record, &mut out)
            .map_err(|_| error())?;
        self.nonce = self.nonce.checked_add(1).ok_or_else(error)?;
        out.truncate(n);
        self.mix_hash(record);
        Ok(out)
    }
    fn encrypt(&mut self, plain: &[u8]) -> Result<Vec<u8>, TransportError> {
        if plain.len() > MAX_RECORD_LEN - 16 {
            return Err(error());
        }
        let mut out = vec![0; plain.len() + 16];
        let n = cipher(&self.key)?.encrypt(self.nonce, &self.hash, plain, &mut out);
        self.nonce = self.nonce.checked_add(1).ok_or_else(error)?;
        out.truncate(n);
        self.mix_hash(&out);
        Ok(out)
    }
}

/// Internal single-use XX handshake; ephemeral keys are never persisted.
pub(super) struct ClassicHandshake {
    state: Symmetric,
    ephemeral: StaticKeypair,
    local: StaticKeypair,
    epoch: ConnectionEpoch,
}

type ServerHandshake = (ClassicFinish, Zeroizing<Vec<u8>>, [u8; 32]);

impl ClassicHandshake {
    pub(super) fn new(
        local: &StaticKeypair,
        epoch: ConnectionEpoch,
    ) -> Result<Self, TransportError> {
        let ephemeral = StaticKeypair::generate(&super::handshake::comparison_noise_config())?;
        let mut state = Symmetric::new();
        state.mix_hash(ephemeral.public());
        Ok(Self {
            state,
            ephemeral,
            local: StaticKeypair::from_parts(*local.private(), *local.public()),
            epoch,
        })
    }
    pub(super) fn ephemeral(&self) -> &[u8; 32] {
        self.ephemeral.public()
    }
    pub(super) fn receive(
        mut self,
        ephemeral: &[u8],
        encrypted_static: &[u8],
        payload: &[u8],
    ) -> Result<ServerHandshake, TransportError> {
        self.state.mix_hash(ephemeral);
        self.state
            .mix_key(dh(self.ephemeral.private(), ephemeral)?.as_ref())?;
        let remote = self.state.decrypt(encrypted_static)?;
        let remote: [u8; 32] = remote.as_slice().try_into().map_err(|_| error())?;
        self.state
            .mix_key(dh(self.ephemeral.private(), &remote)?.as_ref())?;
        let certificates = self.state.decrypt(payload)?;
        let remote_ephemeral = ephemeral.try_into().map_err(|_| error())?;
        Ok((
            ClassicFinish {
                state: self.state,
                local: self.local,
                remote_ephemeral,
                epoch: self.epoch,
            },
            certificates,
            remote,
        ))
    }
}

pub(super) struct ClassicFinish {
    state: Symmetric,
    local: StaticKeypair,
    remote_ephemeral: [u8; 32],
    epoch: ConnectionEpoch,
}

impl ClassicFinish {
    pub(super) fn epoch(&self) -> ConnectionEpoch {
        self.epoch
    }
    pub(super) fn finish(
        mut self,
        login: &[u8],
    ) -> Result<(Vec<u8>, Vec<u8>, ClassicTransport), TransportError> {
        let encrypted_static = self.state.encrypt(self.local.public())?;
        self.state
            .mix_key(dh(self.local.private(), &self.remote_ephemeral)?.as_ref())?;
        let payload = self.state.encrypt(login)?;
        let split = self.state.derive(&[])?;
        let mut send = Zeroizing::new([0; 32]);
        let mut receive = Zeroizing::new([0; 32]);
        send.copy_from_slice(&split[..32]);
        receive.copy_from_slice(&split[32..]);
        Ok((
            encrypted_static,
            payload,
            ClassicTransport {
                send,
                receive,
                send_nonce: 0,
                receive_nonce: 0,
                epoch: self.epoch,
            },
        ))
    }
}

/// Owned by the framed channel, which consumes it on every error/cancellation.
pub(super) struct ClassicTransport {
    send: Zeroizing<[u8; 32]>,
    receive: Zeroizing<[u8; 32]>,
    send_nonce: u64,
    receive_nonce: u64,
    epoch: ConnectionEpoch,
}

impl ClassicTransport {
    pub(super) fn epoch(&self) -> ConnectionEpoch {
        self.epoch
    }
    pub(super) fn encrypt(&mut self, plain: &[u8]) -> Result<Vec<u8>, TransportError> {
        if plain.len() > MAX_RECORD_LEN - 16 || self.send_nonce == u64::MAX {
            return Err(error());
        }
        let mut out = vec![0; plain.len() + 16];
        let n = cipher(&self.send)?.encrypt(self.send_nonce, &[], plain, &mut out);
        self.send_nonce += 1;
        out.truncate(n);
        Ok(out)
    }
    pub(super) fn decrypt(&mut self, record: &[u8]) -> Result<Zeroizing<Vec<u8>>, TransportError> {
        if record.len() < 16 || record.len() > MAX_RECORD_LEN || self.receive_nonce == u64::MAX {
            return Err(error());
        }
        let mut out = Zeroizing::new(vec![0; record.len()]);
        let n = cipher(&self.receive)?
            .decrypt(self.receive_nonce, &[], record, &mut out)
            .map_err(|_| error())?;
        self.receive_nonce += 1;
        out.truncate(n);
        Ok(out)
    }
}

#[cfg(test)]
pub(super) struct TestResponder {
    state: Symmetric,
    ephemeral: StaticKeypair,
}

#[cfg(test)]
impl TestResponder {
    pub(super) fn hello(local: &StaticKeypair, remote: &[u8], cert: &[u8]) -> (Self, Vec<u8>) {
        let ephemeral =
            StaticKeypair::generate(&super::handshake::comparison_noise_config()).unwrap();
        let mut state = Symmetric::new();
        state.mix_hash(remote);
        state.mix_hash(ephemeral.public());
        state
            .mix_key(dh(ephemeral.private(), remote).unwrap().as_ref())
            .unwrap();
        let encrypted_static = state.encrypt(local.public()).unwrap();
        state
            .mix_key(dh(local.private(), remote).unwrap().as_ref())
            .unwrap();
        let payload = state.encrypt(cert).unwrap();
        let record = [ephemeral.public().as_slice(), &encrypted_static, &payload].concat();
        (Self { state, ephemeral }, record)
    }
    pub(super) fn finish(mut self, record: &[u8]) -> (Zeroizing<Vec<u8>>, ClassicTransport) {
        let remote = self.state.decrypt(&record[..48]).unwrap();
        self.state
            .mix_key(dh(self.ephemeral.private(), &remote).unwrap().as_ref())
            .unwrap();
        let payload = self.state.decrypt(&record[48..]).unwrap();
        let split = self.state.derive(&[]).unwrap();
        let mut receive = Zeroizing::new([0; 32]);
        let mut send = Zeroizing::new([0; 32]);
        receive.copy_from_slice(&split[..32]);
        send.copy_from_slice(&split[32..]);
        (
            payload,
            ClassicTransport {
                send,
                receive,
                send_nonce: 0,
                receive_nonce: 0,
                epoch: ConnectionEpoch(1),
            },
        )
    }
}
