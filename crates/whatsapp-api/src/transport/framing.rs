//! Byte-stream framing with a bounded big-endian length prefix.
//!
//! The reader owns its buffer, so cancelling an in-flight read keeps partial
//! bytes (README section 8.4). Multiple frames per read and fragmented
//! headers/bodies are handled.

use bytes::{Bytes, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt};

use super::TransportError;

/// Framing parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameCodec {
    /// Width of the big-endian length prefix in bytes (1..=4).
    pub prefix_bytes: u8,
    /// Maximum accepted frame body length.
    pub max_frame_len: usize,
}

impl FrameCodec {
    /// Validate and construct.
    pub fn new(prefix_bytes: u8, max_frame_len: usize) -> Result<Self, TransportError> {
        if !(1..=4).contains(&prefix_bytes) {
            return Err(TransportError::Config("length prefix must be 1..=4 bytes"));
        }
        let representable = if prefix_bytes == 4 {
            u32::MAX as usize
        } else {
            (1usize << (8 * prefix_bytes as usize)) - 1
        };
        if max_frame_len == 0 || max_frame_len > representable {
            return Err(TransportError::Config(
                "max frame length must be positive and representable in the prefix",
            ));
        }
        Ok(Self {
            prefix_bytes,
            max_frame_len,
        })
    }

    /// Encode one frame. `header` (if any) is emitted before the length
    /// prefix; used for the one-time connection header.
    pub fn encode(&self, payload: &[u8], header: Option<&[u8]>) -> Result<Vec<u8>, TransportError> {
        if payload.len() > self.max_frame_len {
            return Err(TransportError::FrameTooLarge {
                len: payload.len(),
                limit: self.max_frame_len,
            });
        }
        let hlen = header.map_or(0, <[u8]>::len);
        let mut out = Vec::with_capacity(hlen + self.prefix_bytes as usize + payload.len());
        if let Some(h) = header {
            out.extend_from_slice(h);
        }
        let len = payload.len() as u64;
        for i in (0..self.prefix_bytes).rev() {
            out.push(((len >> (8 * i as u64)) & 0xFF) as u8);
        }
        out.extend_from_slice(payload);
        Ok(out)
    }
}

/// Incremental frame reader over an owned buffer.
#[derive(Debug)]
pub struct FrameReader {
    codec: FrameCodec,
    buf: BytesMut,
}

impl FrameReader {
    /// Create a reader.
    pub fn new(codec: FrameCodec) -> Self {
        Self {
            codec,
            buf: BytesMut::with_capacity(4096),
        }
    }

    /// Bytes currently buffered.
    pub fn buffered(&self) -> usize {
        self.buf.len()
    }

    fn bound(&self) -> usize {
        self.codec.prefix_bytes as usize + self.codec.max_frame_len
    }

    /// Append received bytes. The declared length is checked as soon as the
    /// prefix is complete, and the buffer never grows beyond one maximum frame.
    pub fn feed(&mut self, data: &[u8]) -> Result<(), TransportError> {
        if let Some(len) = self.peek_len_with(data)
            && len > self.codec.max_frame_len
        {
            return Err(TransportError::FrameTooLarge {
                len,
                limit: self.codec.max_frame_len,
            });
        }
        if self.buf.len() + data.len() > self.bound() {
            return Err(TransportError::BufferOverflow {
                limit: self.bound(),
            });
        }
        self.buf.extend_from_slice(data);
        Ok(())
    }

    fn peek_len_with(&self, extra: &[u8]) -> Option<usize> {
        let n = self.codec.prefix_bytes as usize;
        if self.buf.len() + extra.len() < n {
            return None;
        }
        let mut len = 0usize;
        for i in 0..n {
            let b = if i < self.buf.len() {
                self.buf[i]
            } else {
                extra[i - self.buf.len()]
            };
            len = (len << 8) | b as usize;
        }
        Some(len)
    }

    /// Pop the next complete frame, if one is buffered.
    pub fn next_frame(&mut self) -> Result<Option<Bytes>, TransportError> {
        let n = self.codec.prefix_bytes as usize;
        let Some(len) = self.peek_len_with(&[]) else {
            return Ok(None);
        };
        if len > self.codec.max_frame_len {
            return Err(TransportError::FrameTooLarge {
                len,
                limit: self.codec.max_frame_len,
            });
        }
        if self.buf.len() < n + len {
            return Ok(None);
        }
        let _ = self.buf.split_to(n);
        Ok(Some(self.buf.split_to(len).freeze()))
    }

    /// Read from `io` until a complete frame is available. Cancellation-safe:
    /// partial bytes stay in the reader. Returns `UnexpectedEof` if the stream
    /// closes mid-frame, and `Ok(None)` if it closes cleanly between frames.
    pub async fn read_frame<R: AsyncRead + Unpin>(
        &mut self,
        io: &mut R,
    ) -> Result<Option<Bytes>, TransportError> {
        loop {
            if let Some(f) = self.next_frame()? {
                return Ok(Some(f));
            }
            let mut chunk = [0u8; 8192];
            let n = io
                .read(&mut chunk)
                .await
                .map_err(|e| TransportError::Noise(format!("read failed: {e}")))?;
            if n == 0 {
                if self.buf.is_empty() {
                    return Ok(None);
                }
                return Err(TransportError::UnexpectedEof {
                    buffered: self.buf.len(),
                });
            }
            self.feed(&chunk[..n])?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::comparison::COMPARISON_CONNECTION_HEADER;

    fn codec() -> FrameCodec {
        FrameCodec::new(3, 1 << 20).unwrap()
    }

    #[test]
    fn config_is_validated() {
        assert!(FrameCodec::new(0, 10).is_err());
        assert!(FrameCodec::new(5, 10).is_err());
        assert!(FrameCodec::new(1, 256).is_err());
        assert!(FrameCodec::new(1, 255).is_ok());
        assert!(FrameCodec::new(3, 1 << 24).is_err());
        assert!(FrameCodec::new(3, (1 << 24) - 1).is_ok());
        assert!(FrameCodec::new(4, u32::MAX as usize).is_ok());
    }

    #[test]
    fn encode_with_header_and_prefix() {
        let c = codec();
        let f = c
            .encode(b"abc", Some(&COMPARISON_CONNECTION_HEADER))
            .unwrap();
        assert_eq!(f, b"WA\x06\x03\x00\x00\x03abc");
        let f = c.encode(&[1u8; 300], None).unwrap();
        assert_eq!(&f[..3], &[0x00, 0x01, 0x2C]);
        assert!(matches!(
            c.encode(&[0u8; (1 << 20) + 1], None),
            Err(TransportError::FrameTooLarge { .. })
        ));
    }

    #[test]
    fn fragmented_and_coalesced_frames() {
        let c = codec();
        let a = c.encode(b"hello", None).unwrap();
        let b = c.encode(b"", None).unwrap();
        let d = c.encode(&[9u8; 1000], None).unwrap();
        let mut stream = Vec::new();
        stream.extend_from_slice(&a);
        stream.extend_from_slice(&b);
        stream.extend_from_slice(&d);
        // Feed one byte at a time.
        let mut r = FrameReader::new(c);
        let mut frames = Vec::new();
        for byte in &stream {
            r.feed(&[*byte]).unwrap();
            while let Some(f) = r.next_frame().unwrap() {
                frames.push(f);
            }
        }
        assert_eq!(frames.len(), 3);
        assert_eq!(&frames[0][..], b"hello");
        assert_eq!(frames[1].len(), 0);
        assert_eq!(frames[2].len(), 1000);
        assert_eq!(r.buffered(), 0);
        // Feed everything at once.
        let mut r = FrameReader::new(c);
        r.feed(&stream).unwrap();
        let mut n = 0;
        while r.next_frame().unwrap().is_some() {
            n += 1;
        }
        assert_eq!(n, 3);
    }

    #[test]
    fn oversized_declared_length_is_rejected_before_allocation() {
        let c = FrameCodec::new(3, 1024).unwrap();
        let mut r = FrameReader::new(c);
        r.feed(&[0x00, 0x10]).unwrap();
        // Third byte completes a length of 0x1000 > 1024: rejected on feed,
        // before any body byte is buffered.
        assert!(matches!(
            r.feed(&[0x00]),
            Err(TransportError::FrameTooLarge {
                len: 4096,
                limit: 1024
            })
        ));
        assert_eq!(r.buffered(), 2);
        let mut r = FrameReader::new(c);
        assert!(matches!(
            r.feed(&[0u8; 2000]),
            Err(TransportError::BufferOverflow { .. })
        ));
        let mut r = FrameReader::new(c);
        assert!(matches!(
            r.feed(&[0xFF, 0xFF, 0xFF, 0, 0, 0, 0]),
            Err(TransportError::FrameTooLarge { .. })
        ));
    }

    #[tokio::test]
    async fn async_read_preserves_partial_bytes_across_cancellation() {
        let c = codec();
        let frame = c.encode(b"payload", None).unwrap();
        let (mut client, mut server) = tokio::io::duplex(64);
        let mut r = FrameReader::new(c);
        tokio::io::AsyncWriteExt::write_all(&mut client, &frame[..4])
            .await
            .unwrap();
        // First attempt times out with partial bytes buffered.
        let res = tokio::time::timeout(
            std::time::Duration::from_millis(20),
            r.read_frame(&mut server),
        )
        .await;
        assert!(res.is_err(), "should time out waiting for the rest");
        assert_eq!(r.buffered(), 4);
        tokio::io::AsyncWriteExt::write_all(&mut client, &frame[4..])
            .await
            .unwrap();
        let f = r.read_frame(&mut server).await.unwrap().unwrap();
        assert_eq!(&f[..], b"payload");
        // Clean close between frames -> None; mid-frame close -> error.
        drop(client);
        assert_eq!(r.read_frame(&mut server).await.unwrap(), None);
        let (mut client, mut server) = tokio::io::duplex(64);
        let mut r = FrameReader::new(c);
        tokio::io::AsyncWriteExt::write_all(&mut client, &frame[..2])
            .await
            .unwrap();
        drop(client);
        assert!(matches!(
            r.read_frame(&mut server).await,
            Err(TransportError::UnexpectedEof { buffered: 2 })
        ));
    }
}
