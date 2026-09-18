//! Opt-in byte transport for the comparison APK. Socket readiness is not login.
//! No account credentials, reconnect loops, or implicit endpoint fallbacks.

use super::comparison::{
    COMPARISON_CONNECTION_HEADER, ComparisonEndpoint, ComparisonEndpointMode, ComparisonSocketLayer,
};
use super::{ConnectionEpoch, FrameCodec, FrameReader, TransportError};
use bytes::Bytes;
use std::{fmt, time::Duration};
use tokio::{
    io::{AsyncRead, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
    time::timeout,
};

/// Local resource/deadline bounds, independent of remote configuration.
#[derive(Debug, Clone, Copy)]
pub struct SocketLimits {
    /// Includes DNS resolution and TCP establishment.
    pub connect_timeout: Duration,
    /// Per complete frame operation, including flush.
    pub io_timeout: Duration,
    /// At most the comparison protocol's 24-bit frame length.
    pub max_frame_len: usize,
}

impl Default for SocketLimits {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(15),
            io_timeout: Duration::from_secs(30),
            max_frame_len: 65_535,
        }
    }
}

impl SocketLimits {
    /// Validate before DNS, connection or allocation.
    pub fn validate(self) -> Result<(), TransportError> {
        for value in [self.connect_timeout, self.io_timeout] {
            if value.is_zero() || value > Duration::from_secs(300) {
                return Err(TransportError::Config(
                    "socket deadlines must be in (0, 300s]",
                ));
            }
        }
        if self.max_frame_len == 0 {
            return Err(TransportError::Config("frame bound must be positive"));
        }
        FrameCodec::new(3, self.max_frame_len)?;
        Ok(())
    }
}

/// Connect only to a canonical raw/direct comparison endpoint. TLS and HTTP
/// wrapper modes are deliberately rejected, never silently downgraded.
pub async fn connect_comparison(
    endpoint: &ComparisonEndpoint,
    epoch: ConnectionEpoch,
    limits: SocketLimits,
) -> Result<FramedConnection<TcpStream>, TransportError> {
    limits.validate()?;
    let canonical = ComparisonEndpoint::built_in(endpoint.gateway, endpoint.port)
        .map_err(|_| TransportError::Config("invalid comparison endpoint"))?;
    if canonical != *endpoint {
        return Err(TransportError::Config(
            "endpoint layer/mode does not match APK mapping",
        ));
    }
    if endpoint.layer != ComparisonSocketLayer::RawTcp
        || endpoint.mode != ComparisonEndpointMode::Direct
    {
        return Err(TransportError::Config(
            "TLS and pseudo-HTTP socket wrappers are not implemented",
        ));
    }
    let host = endpoint.hostname();
    let stream = timeout(
        limits.connect_timeout,
        TcpStream::connect((host.as_str(), endpoint.port)),
    )
    .await
    .map_err(|_| TransportError::Deadline("connect"))?
    .map_err(|e| TransportError::Io(e.kind()))?;
    stream
        .set_nodelay(true)
        .map_err(|e| TransportError::Io(e.kind()))?;
    FramedConnection::new(stream, epoch, limits)
}

/// One connection epoch over an owned byte stream. Send only properly encoded
/// handshake/encrypted records here, never plaintext chat or raw login data.
/// Cancellation during I/O drops the stream; no partial write can be replayed.
pub struct FramedConnection<S> {
    stream: Option<S>,
    codec: FrameCodec,
    reader: FrameReader,
    epoch: ConnectionEpoch,
    first_write: bool,
    io_timeout: Duration,
}

impl<S> fmt::Debug for FramedConnection<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FramedConnection")
            .field("epoch", &self.epoch)
            .field("open", &self.stream.is_some())
            .finish_non_exhaustive()
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> FramedConnection<S> {
    pub(crate) fn into_split(mut self) -> Result<SplitSocket<S>, TransportError> {
        if self.first_write {
            return Err(TransportError::Config("cannot split before handshake"));
        }
        let stream = self.stream.take().ok_or(TransportError::Closed)?;
        let (read, write) = tokio::io::split(stream);
        Ok(SplitSocket {
            read,
            write,
            reader: self.reader,
            codec: self.codec,
            io_timeout: self.io_timeout,
        })
    }
    /// Wrap an established byte stream. Does not perform I/O or authentication.
    pub fn new(
        stream: S,
        epoch: ConnectionEpoch,
        limits: SocketLimits,
    ) -> Result<Self, TransportError> {
        limits.validate()?;
        let codec = FrameCodec::new(3, limits.max_frame_len)?;
        Ok(Self {
            stream: Some(stream),
            codec,
            reader: FrameReader::new(codec),
            epoch,
            first_write: true,
            io_timeout: limits.io_timeout,
        })
    }

    /// Whether the owned stream is still usable, not whether it is authenticated.
    pub fn is_open(&self) -> bool {
        self.stream.is_some()
    }

    /// Close without replaying pending writes or reconnecting.
    pub fn close(&mut self) {
        self.stream = None;
    }

    fn check_epoch(&self, epoch: ConnectionEpoch) -> Result<(), TransportError> {
        if epoch != self.epoch {
            return Err(TransportError::StaleEpoch {
                stale: epoch,
                current: self.epoch,
            });
        }
        if self.stream.is_none() {
            return Err(TransportError::Closed);
        }
        Ok(())
    }

    /// Write one frame, prepending WA/version bytes exactly once per socket.
    pub async fn write_frame(
        &mut self,
        epoch: ConnectionEpoch,
        payload: &[u8],
    ) -> Result<(), TransportError> {
        self.check_epoch(epoch)?;
        let header = self
            .first_write
            .then_some(COMPARISON_CONNECTION_HEADER.as_slice());
        let frame = self.codec.encode(payload, header)?;
        let mut stream = self.stream.take().ok_or(TransportError::Closed)?;
        timeout(self.io_timeout, async {
            stream.write_all(&frame).await?;
            stream.flush().await
        })
        .await
        .map_err(|_| TransportError::Deadline("write"))?
        .map_err(|e| TransportError::Io(e.kind()))?;
        self.first_write = false;
        self.stream = Some(stream);
        Ok(())
    }

    /// Receive one bounded frame. EOF, errors, deadlines and cancellation close
    /// this connection; coalesced frames remain buffered for the next call.
    pub async fn read_frame(
        &mut self,
        epoch: ConnectionEpoch,
    ) -> Result<Option<Bytes>, TransportError> {
        self.check_epoch(epoch)?;
        let mut stream = self.stream.take().ok_or(TransportError::Closed)?;
        let result = timeout(self.io_timeout, self.reader.read_frame(&mut stream))
            .await
            .map_err(|_| TransportError::Deadline("read"))??;
        if result.is_some() {
            self.stream = Some(stream);
        }
        Ok(result)
    }
}

pub(crate) struct SplitSocket<S> {
    pub read: tokio::io::ReadHalf<S>,
    pub write: tokio::io::WriteHalf<S>,
    pub reader: FrameReader,
    pub codec: FrameCodec,
    pub io_timeout: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt;

    fn limits() -> SocketLimits {
        SocketLimits {
            max_frame_len: 8,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn writes_header_once_and_rejects_stale_epochs_without_io() {
        let (client, mut server) = tokio::io::duplex(64);
        let mut connection = FramedConnection::new(client, ConnectionEpoch(7), limits()).unwrap();
        assert!(matches!(
            connection.write_frame(ConnectionEpoch(6), b"bad").await,
            Err(TransportError::StaleEpoch { .. })
        ));
        connection
            .write_frame(ConnectionEpoch(7), b"abc")
            .await
            .unwrap();
        connection
            .write_frame(ConnectionEpoch(7), b"d")
            .await
            .unwrap();
        let mut received = [0; 14];
        server.read_exact(&mut received).await.unwrap();
        assert_eq!(&received, b"WA\x06\x03\x00\x00\x03abc\x00\x00\x01d");
    }

    #[tokio::test]
    async fn coalesced_maximum_frames_and_clean_eof() {
        let (client, mut server) = tokio::io::duplex(64);
        let mut connection = FramedConnection::new(client, ConnectionEpoch(1), limits()).unwrap();
        server
            .write_all(b"\x00\x00\x0812345678\x00\x00\x08abcdefgh")
            .await
            .unwrap();
        drop(server);
        assert_eq!(
            &connection
                .read_frame(ConnectionEpoch(1))
                .await
                .unwrap()
                .unwrap()[..],
            b"12345678"
        );
        assert_eq!(
            &connection
                .read_frame(ConnectionEpoch(1))
                .await
                .unwrap()
                .unwrap()[..],
            b"abcdefgh"
        );
        assert!(
            connection
                .read_frame(ConnectionEpoch(1))
                .await
                .unwrap()
                .is_none()
        );
        assert!(!connection.is_open());
    }

    #[tokio::test(start_paused = true)]
    async fn interrupted_writes_and_reads_close_permanently() {
        let (client, _server) = tokio::io::duplex(1);
        let mut connection = FramedConnection::new(client, ConnectionEpoch(1), limits()).unwrap();
        assert!(
            timeout(
                Duration::from_millis(1),
                connection.write_frame(ConnectionEpoch(1), b"abc")
            )
            .await
            .is_err()
        );
        assert!(!connection.is_open());
        assert!(matches!(
            connection.write_frame(ConnectionEpoch(1), b"abc").await,
            Err(TransportError::Closed)
        ));
        let (client, _server) = tokio::io::duplex(8);
        let mut connection = FramedConnection::new(client, ConnectionEpoch(1), limits()).unwrap();
        assert!(matches!(
            connection.read_frame(ConnectionEpoch(1)).await,
            Err(TransportError::Deadline("read"))
        ));
        assert!(!connection.is_open());
    }

    #[tokio::test]
    async fn malformed_peer_frame_closes_before_large_allocation() {
        let (client, mut server) = tokio::io::duplex(64);
        let mut connection = FramedConnection::new(client, ConnectionEpoch(1), limits()).unwrap();
        server.write_all(b"\xff\xff\xff").await.unwrap();
        assert!(matches!(
            connection.read_frame(ConnectionEpoch(1)).await,
            Err(TransportError::FrameTooLarge { .. })
        ));
        assert!(!connection.is_open());
    }

    #[tokio::test]
    async fn local_oversize_preserves_connection_and_first_header() {
        let (client, mut server) = tokio::io::duplex(64);
        let mut connection = FramedConnection::new(client, ConnectionEpoch(1), limits()).unwrap();
        assert!(matches!(
            connection
                .write_frame(ConnectionEpoch(1), b"123456789")
                .await,
            Err(TransportError::FrameTooLarge { .. })
        ));
        assert!(connection.is_open());
        connection
            .write_frame(ConnectionEpoch(1), b"a")
            .await
            .unwrap();
        let mut received = [0; 8];
        server.read_exact(&mut received).await.unwrap();
        assert_eq!(&received, b"WA\x06\x03\x00\x00\x01a");
    }

    #[tokio::test(start_paused = true)]
    async fn canceled_partial_read_and_write_deadline_close() {
        let (client, mut server) = tokio::io::duplex(64);
        let mut connection = FramedConnection::new(client, ConnectionEpoch(1), limits()).unwrap();
        server.write_all(b"\x00\x00\x04a").await.unwrap();
        assert!(
            timeout(
                Duration::from_millis(1),
                connection.read_frame(ConnectionEpoch(1))
            )
            .await
            .is_err()
        );
        assert!(!connection.is_open());
        let (client, _server) = tokio::io::duplex(1);
        let mut connection = FramedConnection::new(client, ConnectionEpoch(1), limits()).unwrap();
        assert!(matches!(
            connection.write_frame(ConnectionEpoch(1), b"abc").await,
            Err(TransportError::Deadline("write"))
        ));
        assert!(!connection.is_open());
    }

    #[tokio::test]
    async fn truncated_frame_closes_without_replay() {
        let (client, mut server) = tokio::io::duplex(64);
        let mut connection = FramedConnection::new(client, ConnectionEpoch(1), limits()).unwrap();
        server.write_all(b"\x00\x00\x04a").await.unwrap();
        drop(server);
        assert!(matches!(
            connection.read_frame(ConnectionEpoch(1)).await,
            Err(TransportError::UnexpectedEof { buffered: 4 })
        ));
        assert!(matches!(
            connection.read_frame(ConnectionEpoch(1)).await,
            Err(TransportError::Closed)
        ));
    }

    #[tokio::test]
    async fn unsupported_layers_and_modified_endpoints_fail_before_connect() {
        use super::super::comparison::ComparisonGateway;
        let mut endpoint = ComparisonEndpoint::built_in(ComparisonGateway::Edge(1), 443).unwrap();
        assert!(matches!(
            connect_comparison(&endpoint, ConnectionEpoch(1), limits()).await,
            Err(TransportError::Config(_))
        ));
        endpoint.layer = ComparisonSocketLayer::RawTcp;
        assert!(matches!(
            connect_comparison(&endpoint, ConnectionEpoch(1), limits()).await,
            Err(TransportError::Config(_))
        ));
    }

    #[tokio::test]
    async fn real_loopback_tcp_roundtrip() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let peer = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut bytes = [0; 11];
            stream.read_exact(&mut bytes).await.unwrap();
            assert_eq!(&bytes, b"WA\x06\x03\x00\x00\x04ping");
            stream.write_all(b"\x00\x00\x04pong").await.unwrap();
        });
        let stream = TcpStream::connect(address).await.unwrap();
        let mut connection = FramedConnection::new(stream, ConnectionEpoch(1), limits()).unwrap();
        connection
            .write_frame(ConnectionEpoch(1), b"ping")
            .await
            .unwrap();
        assert_eq!(
            &connection
                .read_frame(ConnectionEpoch(1))
                .await
                .unwrap()
                .unwrap()[..],
            b"pong"
        );
        peer.await.unwrap();
    }
}
