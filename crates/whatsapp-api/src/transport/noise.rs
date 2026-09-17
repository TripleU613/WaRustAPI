//! Noise handshake/transport adapter over `snow` (experimental).
//!
//! Compiled only with `experimental-noise`. `snow` is used with its Rust
//! crypto resolver and **without** its `std` feature, because that feature
//! activates the optional `ring` dependency (ADR 0004).
//!
//! Rules enforced here (README section 8.3):
//!
//! * a handshake produces a transport bound to one [`ConnectionEpoch`];
//! * transport keys/counters are never persisted or reused on a new socket;
//! * after any failure the state is consumed and the connection must close;
//! * records above the backend maximum are rejected, never fragmented.

use zeroize::Zeroizing;

use super::{ConnectionEpoch, TransportError};

/// Maximum Noise record the `snow` backend accepts.
pub const MAX_RECORD_LEN: usize = 65_535;

/// Parameters for one handshake.
#[derive(Debug, Clone)]
pub struct NoiseConfig {
    /// Noise protocol name, e.g. `Noise_XX_25519_AESGCM_SHA256`.
    pub pattern: String,
    /// Prologue bytes mixed into the handshake hash.
    pub prologue: Vec<u8>,
}

/// A static Noise keypair. The private half is zeroized on drop.
pub struct StaticKeypair {
    private: Zeroizing<[u8; 32]>,
    public: [u8; 32],
}

impl std::fmt::Debug for StaticKeypair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StaticKeypair(pub={})", hex::encode(self.public))
    }
}

impl StaticKeypair {
    /// Generate a fresh keypair using the backend's DH implementation.
    pub fn generate(config: &NoiseConfig) -> Result<Self, TransportError> {
        let params = parse_params(&config.pattern)?;
        let kp = snow::Builder::new(params)
            .generate_keypair()
            .map_err(|e| TransportError::Noise(format!("keygen: {e:?}")))?;
        let mut private = Zeroizing::new([0u8; 32]);
        private.copy_from_slice(&kp.private);
        let mut public = [0u8; 32];
        public.copy_from_slice(&kp.public);
        Ok(Self { private, public })
    }

    /// Reconstruct from persisted halves.
    pub fn from_parts(private: [u8; 32], public: [u8; 32]) -> Self {
        Self {
            private: Zeroizing::new(private),
            public,
        }
    }

    /// Public key bytes.
    pub fn public(&self) -> &[u8; 32] {
        &self.public
    }

    /// Private key bytes (for persistence inside an encrypted envelope only).
    pub fn private(&self) -> &[u8; 32] {
        &self.private
    }
}

fn parse_params(pattern: &str) -> Result<snow::params::NoiseParams, TransportError> {
    pattern
        .parse()
        .map_err(|_| TransportError::Config("unparseable noise pattern"))
}

/// An in-progress handshake as initiator.
pub struct Handshake {
    state: snow::HandshakeState,
    epoch: ConnectionEpoch,
}

impl std::fmt::Debug for Handshake {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Handshake")
            .field("epoch", &self.epoch)
            .field("finished", &self.state.is_handshake_finished())
            .finish()
    }
}

impl Handshake {
    /// Start an initiator handshake for a new connection epoch.
    pub fn initiator(
        config: &NoiseConfig,
        local_static: &StaticKeypair,
        epoch: ConnectionEpoch,
    ) -> Result<Self, TransportError> {
        let params = parse_params(&config.pattern)?;
        let state = snow::Builder::new(params)
            .prologue(&config.prologue)
            .map_err(|e| TransportError::Noise(format!("prologue: {e:?}")))?
            .local_private_key(local_static.private())
            .map_err(|e| TransportError::Noise(format!("static key: {e:?}")))?
            .build_initiator()
            .map_err(|e| TransportError::Noise(format!("build: {e:?}")))?;
        Ok(Self { state, epoch })
    }

    /// Epoch this handshake belongs to.
    pub fn epoch(&self) -> ConnectionEpoch {
        self.epoch
    }

    /// Write the next handshake message with `payload`.
    pub fn write_message(&mut self, payload: &[u8]) -> Result<Vec<u8>, TransportError> {
        let mut out = vec![0u8; payload.len() + 256];
        let n = self
            .state
            .write_message(payload, &mut out)
            .map_err(|e| TransportError::Noise(format!("write: {e:?}")))?;
        out.truncate(n);
        Ok(out)
    }

    /// Read the next handshake message and return its payload.
    pub fn read_message(&mut self, message: &[u8]) -> Result<Vec<u8>, TransportError> {
        if message.len() > MAX_RECORD_LEN {
            return Err(TransportError::RecordTooLarge {
                len: message.len(),
                max: MAX_RECORD_LEN,
            });
        }
        let mut out = vec![0u8; message.len()];
        let n = self
            .state
            .read_message(message, &mut out)
            .map_err(|e| TransportError::Noise(format!("read: {e:?}")))?;
        out.truncate(n);
        Ok(out)
    }

    /// The remote static key, once the handshake has revealed it.
    pub fn remote_static(&self) -> Option<[u8; 32]> {
        self.state
            .get_remote_static()
            .and_then(|s| s.try_into().ok())
    }

    /// Whether the handshake is complete.
    pub fn is_finished(&self) -> bool {
        self.state.is_handshake_finished()
    }

    /// Finish into transport mode. Consumes the handshake; there is no way to
    /// obtain the transport twice.
    pub fn into_transport(self) -> Result<NoiseTransport, TransportError> {
        let state = self
            .state
            .into_transport_mode()
            .map_err(|e| TransportError::Noise(format!("transport: {e:?}")))?;
        Ok(NoiseTransport {
            state: Some(state),
            epoch: self.epoch,
        })
    }
}

/// Established Noise transport for one connection epoch.
pub struct NoiseTransport {
    state: Option<snow::TransportState>,
    epoch: ConnectionEpoch,
}

impl std::fmt::Debug for NoiseTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NoiseTransport")
            .field("epoch", &self.epoch)
            .field("closed", &self.state.is_none())
            .finish()
    }
}

impl NoiseTransport {
    /// Epoch this transport belongs to.
    pub fn epoch(&self) -> ConnectionEpoch {
        self.epoch
    }

    fn state(&mut self) -> Result<&mut snow::TransportState, TransportError> {
        self.state
            .as_mut()
            .ok_or_else(|| TransportError::Noise("transport closed after failure".into()))
    }

    /// Encrypt one record. On failure the transport is closed permanently.
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<Vec<u8>, TransportError> {
        if plaintext.len() + 16 > MAX_RECORD_LEN {
            return Err(TransportError::RecordTooLarge {
                len: plaintext.len() + 16,
                max: MAX_RECORD_LEN,
            });
        }
        let mut out = vec![0u8; plaintext.len() + 16];
        match self.state()?.write_message(plaintext, &mut out) {
            Ok(n) => {
                out.truncate(n);
                Ok(out)
            }
            Err(e) => {
                self.state = None;
                Err(TransportError::Noise(format!("encrypt: {e:?}")))
            }
        }
    }

    /// Decrypt one record. On failure the transport is closed permanently.
    pub fn decrypt(&mut self, record: &[u8]) -> Result<Vec<u8>, TransportError> {
        if record.len() > MAX_RECORD_LEN {
            return Err(TransportError::RecordTooLarge {
                len: record.len(),
                max: MAX_RECORD_LEN,
            });
        }
        let mut out = vec![0u8; record.len()];
        match self.state()?.read_message(record, &mut out) {
            Ok(n) => {
                out.truncate(n);
                Ok(out)
            }
            Err(e) => {
                self.state = None;
                Err(TransportError::Noise(format!("decrypt: {e:?}")))
            }
        }
    }

    /// Whether the transport is still usable.
    pub fn is_open(&self) -> bool {
        self.state.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> NoiseConfig {
        NoiseConfig {
            pattern: "Noise_XX_25519_AESGCM_SHA256".into(),
            prologue: b"WA\x06\x03".to_vec(),
        }
    }

    /// Full XX handshake against a snow responder, then bidirectional
    /// records. This proves the adapter, not WhatsApp compatibility.
    #[test]
    fn xx_handshake_and_records_roundtrip() {
        let cfg = config();
        let client_static = StaticKeypair::generate(&cfg).unwrap();
        let server_static = StaticKeypair::generate(&cfg).unwrap();
        let mut client = Handshake::initiator(&cfg, &client_static, ConnectionEpoch(1)).unwrap();
        let params: snow::params::NoiseParams = cfg.pattern.parse().unwrap();
        let mut server = snow::Builder::new(params)
            .prologue(&cfg.prologue)
            .unwrap()
            .local_private_key(server_static.private())
            .unwrap()
            .build_responder()
            .unwrap();
        let mut buf = vec![0u8; 1024];

        // -> e
        let m1 = client.write_message(b"hello").unwrap();
        let n = server.read_message(&m1, &mut buf).unwrap();
        assert_eq!(&buf[..n], b"hello");
        // <- e, ee, s, es
        let n = server.write_message(b"cert", &mut buf).unwrap();
        let p = client.read_message(&buf[..n]).unwrap();
        assert_eq!(p, b"cert");
        assert_eq!(client.remote_static(), Some(*server_static.public()));
        // -> s, se
        let m3 = client.write_message(b"login").unwrap();
        let n = server.read_message(&m3, &mut buf).unwrap();
        assert_eq!(&buf[..n], b"login");
        assert!(client.is_finished());

        let mut ct = client.into_transport().unwrap();
        let mut st = server.into_transport_mode().unwrap();
        let rec = ct.encrypt(b"ping").unwrap();
        let n = st.read_message(&rec, &mut buf).unwrap();
        assert_eq!(&buf[..n], b"ping");
        let n = st.write_message(b"pong", &mut buf).unwrap();
        assert_eq!(ct.decrypt(&buf[..n]).unwrap(), b"pong");

        // A corrupted record closes the transport permanently.
        let mut bad = st
            .write_message(b"x", &mut buf)
            .map(|n| buf[..n].to_vec())
            .unwrap();
        bad[0] ^= 0xFF;
        assert!(ct.decrypt(&bad).is_err());
        assert!(!ct.is_open());
        assert!(ct.encrypt(b"y").is_err());
    }

    #[test]
    fn oversized_records_are_rejected_not_fragmented() {
        let cfg = config();
        let ks = StaticKeypair::generate(&cfg).unwrap();
        let mut hs = Handshake::initiator(&cfg, &ks, ConnectionEpoch(1)).unwrap();
        let big = vec![0u8; MAX_RECORD_LEN + 1];
        assert!(matches!(
            hs.read_message(&big),
            Err(TransportError::RecordTooLarge { .. })
        ));
    }

    #[test]
    fn keypair_debug_hides_private() {
        let cfg = config();
        let ks = StaticKeypair::generate(&cfg).unwrap();
        let d = format!("{ks:?}");
        assert!(!d.contains(&hex::encode(ks.private())));
        let again = StaticKeypair::from_parts(*ks.private(), *ks.public());
        assert_eq!(again.public(), ks.public());
    }
}
