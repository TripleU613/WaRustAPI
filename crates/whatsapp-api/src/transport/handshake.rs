//! Comparison APK classic XX envelope and certificate-checked framed session.
//! Source: LX/1N7 constructor, A03/A04 and LX/Aw1 in 2.26.37.70.

use bytes::Bytes;
use prost::Message;
use tokio::io::{AsyncRead, AsyncWrite};
use zeroize::Zeroizing;

use super::cert::{ChainPolicy, SignatureVerifier, VerifiedChain, verify_chain};
use super::classic::{ClassicFinish, ClassicHandshake, ClassicTransport};
use super::comparison::COMPARISON_CONNECTION_HEADER;
use super::noise::{MAX_RECORD_LEN, NoiseConfig, StaticKeypair};
use super::socket::FramedConnection;
use super::{ConnectionEpoch, TransportError};
use crate::model::time::Timestamp;
use crate::proto::wa_web_protobufs_wa6::{
    HandshakeMessage,
    handshake_message::{ClientFinish, ClientHello},
};

/// Comparison LX/Aw1.<clinit>, WhatsAppLongTerm1; LX/1N7.A03 requires issuer 0.
pub const COMPARISON_CHAIN_POLICY: ChainPolicy = ChainPolicy {
    trust_root: [
        0x14, 0x23, 0x75, 0x57, 0x4d, 0x0a, 0x58, 0x71, 0x66, 0xaa, 0xe7, 0x1e, 0xbe, 0x51, 0x64,
        0x37, 0xc4, 0xa2, 0x8b, 0x73, 0xe3, 0x69, 0x5c, 0x6c, 0xe1, 0xf7, 0xf9, 0x54, 0x5d, 0xa8,
        0xee, 0x6b,
    ],
    intermediate_issuer_serial: 0,
};

fn reject(reason: &'static str) -> TransportError {
    TransportError::Handshake(reason.into())
}

/// Classic compiled-default configuration, not hybrid negotiation or IK resume.
pub fn comparison_noise_config() -> NoiseConfig {
    NoiseConfig {
        pattern: "Noise_XX_25519_AESGCM_SHA256".into(),
        prologue: COMPARISON_CONNECTION_HEADER.to_vec(),
    }
}

/// A verified server, before any client static key or login payload is sent.
/// Owns the socket: dropping or canceling this operation cannot reuse its state.
pub struct VerifiedHandshake<S> {
    connection: FramedConnection<S>,
    handshake: ClassicFinish,
    chain: VerifiedChain,
}

impl<S: AsyncRead + AsyncWrite + Unpin> VerifiedHandshake<S> {
    /// Send one classical ClientHello, decode ServerHello, verify the chain and
    /// bind its leaf to Noise's remote static key. No retries or downgrade.
    pub async fn start(
        mut connection: FramedConnection<S>,
        local: &StaticKeypair,
        epoch: ConnectionEpoch,
        now: Timestamp,
        policy: &ChainPolicy,
        verifier: &dyn SignatureVerifier,
    ) -> Result<Self, TransportError> {
        let handshake = ClassicHandshake::new(local, epoch)?;
        let ephemeral = Bytes::copy_from_slice(handshake.ephemeral());
        let hello = HandshakeMessage {
            client_hello: Some(ClientHello {
                ephemeral: Some(ephemeral),
                ..Default::default()
            }),
            ..Default::default()
        }
        .encode_to_vec();
        connection.write_frame(epoch, &hello).await?;
        let frame = connection
            .read_frame(epoch)
            .await?
            .ok_or_else(|| reject("EOF before server hello"))?;
        let wire =
            HandshakeMessage::decode(frame).map_err(|_| reject("invalid server hello protobuf"))?;
        if wire.client_hello.is_some() || wire.client_finish.is_some() {
            return Err(reject("unexpected handshake message direction"));
        }
        let server = wire
            .server_hello
            .ok_or_else(|| reject("missing server hello"))?;
        if server.extended_static.is_some() || server.extended_ciphertext.is_some() {
            return Err(reject("hybrid server hello is unsupported; no downgrade"));
        }
        let ephemeral = server
            .ephemeral
            .ok_or_else(|| reject("missing server ephemeral"))?;
        let static_key = server
            .r#static
            .ok_or_else(|| reject("missing encrypted server static"))?;
        let payload = server
            .payload
            .ok_or_else(|| reject("missing server certificate payload"))?;
        if ephemeral.len() != 32 || static_key.len() != 48 || payload.len() < 16 {
            return Err(reject("invalid classical server hello field lengths"));
        }
        let record_len = ephemeral.len() + static_key.len() + payload.len();
        if record_len > MAX_RECORD_LEN {
            return Err(TransportError::RecordTooLarge {
                len: record_len,
                max: MAX_RECORD_LEN,
            });
        }
        let (handshake, certificates, remote) =
            handshake.receive(&ephemeral, &static_key, &payload)?;
        let chain = verify_chain(&certificates, &remote, now, policy, verifier)?;
        Ok(Self {
            connection,
            handshake,
            chain,
        })
    }

    /// Public certificate facts only; this does not establish account login.
    pub fn chain(&self) -> &VerifiedChain {
        &self.chain
    }

    /// Encrypt the supplied primary login protobuf and send ClientFinish once.
    /// The returned channel still needs a server login-success response.
    pub async fn finish(mut self, login: &[u8]) -> Result<EncryptedConnection<S>, TransportError> {
        if login.len() > MAX_RECORD_LEN - 64 {
            return Err(reject("login payload exceeds classical handshake bound"));
        }
        let epoch = self.handshake.epoch();
        let (encrypted_static, payload, transport) = self.handshake.finish(login)?;
        let finish = HandshakeMessage {
            client_finish: Some(ClientFinish {
                r#static: Some(encrypted_static.into()),
                payload: Some(payload.into()),
                ..Default::default()
            }),
            ..Default::default()
        }
        .encode_to_vec();
        self.connection.write_frame(epoch, &finish).await?;
        Ok(EncryptedConnection {
            inner: Some((self.connection, transport)),
        })
    }
}

/// Certificate-verified encrypted records, not an authenticated account session.
/// Cancellation consumes both socket and cipher counters, never just one half.
pub struct EncryptedConnection<S> {
    inner: Option<(FramedConnection<S>, ClassicTransport)>,
}

impl<S: AsyncRead + AsyncWrite + Unpin> EncryptedConnection<S> {
    /// Send one encrypted binary record. An error permanently consumes the channel.
    pub async fn write(&mut self, plaintext: &[u8]) -> Result<(), TransportError> {
        let (mut socket, mut noise) = self.inner.take().ok_or(TransportError::Closed)?;
        let record = noise.encrypt(plaintext)?;
        socket.write_frame(noise.epoch(), &record).await?;
        self.inner = Some((socket, noise));
        Ok(())
    }

    /// Receive/decrypt one record, clearing plaintext on drop.
    pub async fn read(&mut self) -> Result<Option<Zeroizing<Vec<u8>>>, TransportError> {
        let (mut socket, mut noise) = self.inner.take().ok_or(TransportError::Closed)?;
        let Some(record) = socket.read_frame(noise.epoch()).await? else {
            return Ok(None);
        };
        let plaintext = noise.decrypt(&record)?;
        self.inner = Some((socket, noise));
        Ok(Some(plaintext))
    }
}

impl<S> std::fmt::Debug for VerifiedHandshake<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VerifiedHandshake")
            .field("connection", &self.connection)
            .finish_non_exhaustive()
    }
}
impl<S> std::fmt::Debug for EncryptedConnection<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncryptedConnection")
            .field("open", &self.inner.is_some())
            .finish_non_exhaustive()
    }
}

#[cfg(all(test, feature = "experimental-signal-crypto"))]
mod tests {
    use super::*;
    use crate::proto::wa_cert::{
        CertChain,
        cert_chain::{NoiseCertificate, noise_certificate::Details},
    };
    use crate::proto::wa_web_protobufs_wa6::handshake_message::ServerHello;
    use crate::signal::crypto::SignalSignatureVerifier;
    use crate::transport::{FrameCodec, FrameReader, socket::SocketLimits};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use wacore_libsignal::core::curve::PrivateKey;

    fn cert(signer: &PrivateKey, serial: u32, issuer: u32, public: &[u8]) -> NoiseCertificate {
        let details = Details {
            serial: Some(serial),
            issuer_serial: Some(issuer),
            key: Some(Bytes::copy_from_slice(public)),
            not_before: Some(100),
            not_after: Some(1000),
        }
        .encode_to_vec();
        let signature = signer
            .calculate_signature(&details, &mut rand::rng())
            .unwrap();
        NoiseCertificate {
            details: Some(details.into()),
            signature: Some(signature.to_vec().into()),
        }
    }

    async fn exchange(wrong_root: bool) {
        let cfg = comparison_noise_config();
        let client_key = StaticKeypair::generate(&cfg).unwrap();
        let server_key = StaticKeypair::generate(&cfg).unwrap();
        let root = PrivateKey::deserialize(&[19; 32]).unwrap();
        let intermediate = PrivateKey::deserialize(&[29; 32]).unwrap();
        let root_public = root.public_key().unwrap();
        let intermediate_public = intermediate.public_key().unwrap();
        let chain = CertChain {
            intermediate: Some(cert(&root, 7, 0, intermediate_public.public_key_bytes())),
            leaf: Some(cert(&intermediate, 9, 7, server_key.public())),
        }
        .encode_to_vec();
        let mut policy = ChainPolicy {
            trust_root: root_public.public_key_bytes().try_into().unwrap(),
            intermediate_issuer_serial: 0,
        };
        if wrong_root {
            policy.trust_root[0] ^= 1;
        }
        let (client, mut server) = tokio::io::duplex(8192);
        let peer = tokio::spawn(async move {
            let mut header = [0; 4];
            server.read_exact(&mut header).await.unwrap();
            assert_eq!(header, *b"WA\x06\x03");
            let codec = FrameCodec::new(3, MAX_RECORD_LEN).unwrap();
            let mut reader = FrameReader::new(codec);
            let frame = reader.read_frame(&mut server).await.unwrap().unwrap();
            // Handwritten protobuf keys: outer field 2, inner field 1, 32 bytes.
            assert_eq!(&frame[..4], &[0x12, 0x22, 0x0a, 0x20]);
            assert_eq!(frame.len(), 36);
            let hello = HandshakeMessage::decode(frame)
                .unwrap()
                .client_hello
                .unwrap();
            let (noise, buf) = super::super::classic::TestResponder::hello(
                &server_key,
                &hello.ephemeral.unwrap(),
                &chain,
            );
            let n = buf.len();
            let response = HandshakeMessage {
                server_hello: Some(ServerHello {
                    ephemeral: Some(Bytes::copy_from_slice(&buf[..32])),
                    r#static: Some(Bytes::copy_from_slice(&buf[32..80])),
                    payload: Some(Bytes::copy_from_slice(&buf[80..n])),
                    ..Default::default()
                }),
                ..Default::default()
            }
            .encode_to_vec();
            server
                .write_all(&codec.encode(&response, None).unwrap())
                .await
                .unwrap();
            let finish = reader.read_frame(&mut server).await.unwrap();
            if wrong_root {
                assert!(finish.is_none(), "untrusted peer must never receive login");
                return;
            }
            let finish = HandshakeMessage::decode(finish.unwrap())
                .unwrap()
                .client_finish
                .unwrap();
            let record = [
                finish.r#static.unwrap().as_ref(),
                finish.payload.unwrap().as_ref(),
            ]
            .concat();
            let (login, mut transport) = noise.finish(&record);
            assert_eq!(login.as_slice(), b"local login fixture");
            let incoming = reader.read_frame(&mut server).await.unwrap().unwrap();
            let plain = transport.decrypt(&incoming).unwrap();
            assert_eq!(plain.as_slice(), b"local record fixture");
            let buf = transport.encrypt(b"reply").unwrap();
            server
                .write_all(&codec.encode(&buf, None).unwrap())
                .await
                .unwrap();
        });
        let socket =
            FramedConnection::new(client, ConnectionEpoch(1), SocketLimits::default()).unwrap();
        let result = VerifiedHandshake::start(
            socket,
            &client_key,
            ConnectionEpoch(1),
            Timestamp::from_unix_seconds(500),
            &policy,
            &SignalSignatureVerifier,
        )
        .await;
        if wrong_root {
            assert!(result.is_err());
        } else {
            let verified = result.ok().unwrap();
            assert_eq!(verified.chain().leaf_serial, 9);
            let mut channel = verified.finish(b"local login fixture").await.unwrap();
            channel.write(b"local record fixture").await.unwrap();
            assert_eq!(channel.read().await.unwrap().unwrap().as_slice(), b"reply");
            assert!(channel.read().await.unwrap().is_none());
            assert!(matches!(
                channel.write(b"closed").await,
                Err(TransportError::Closed)
            ));
        }
        peer.await.unwrap();
    }

    #[tokio::test]
    async fn verified_handshake_and_encrypted_records() {
        exchange(false).await;
    }

    #[tokio::test]
    async fn untrusted_server_never_receives_login() {
        exchange(true).await;
    }

    #[test]
    fn root_matches_apk_dump() {
        let source = include_str!("../../fixtures/comparisons/whatsapp-2.26.37.70.json");
        let bytes = COMPARISON_CHAIN_POLICY
            .trust_root
            .iter()
            .map(|v| format!("{v:02x}"))
            .collect::<Vec<_>>()
            .join("");
        assert!(source.contains(&bytes));
    }
}
