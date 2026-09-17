//! Bounded, epoch-tagged request correlation (README section 8.1).

use std::collections::HashMap;

use tokio::sync::oneshot;

use crate::binary::Node;
use crate::error::{Error, Result};
use crate::model::time::Timestamp;
use crate::transport::{ConnectionEpoch, TransportError};

/// Outcome delivered to a waiting request.
pub type Response = Result<Node>;

struct Pending {
    epoch: ConnectionEpoch,
    deadline: Timestamp,
    reply: oneshot::Sender<Response>,
}

/// Table of outstanding IQ-style requests.
pub struct RequestTable {
    epoch: ConnectionEpoch,
    max: usize,
    next: u64,
    pending: HashMap<String, Pending>,
}

impl std::fmt::Debug for RequestTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequestTable")
            .field("epoch", &self.epoch)
            .field("pending", &self.pending.len())
            .finish()
    }
}

impl RequestTable {
    /// Create a table for an epoch with a maximum number of outstanding requests.
    pub fn new(epoch: ConnectionEpoch, max: usize) -> Self {
        Self {
            epoch,
            max,
            next: 1,
            pending: HashMap::new(),
        }
    }

    /// Register a request. Returns its id and the receiver for its response.
    pub fn register(
        &mut self,
        deadline: Timestamp,
    ) -> Result<(String, oneshot::Receiver<Response>)> {
        if self.pending.len() >= self.max {
            return Err(Error::ResourceExhausted {
                resource: "pending requests",
                limit: self.max as u64,
            });
        }
        let id = format!("{}.{}", self.epoch.0, self.next);
        self.next += 1;
        let (tx, rx) = oneshot::channel();
        self.pending.insert(
            id.clone(),
            Pending {
                epoch: self.epoch,
                deadline,
                reply: tx,
            },
        );
        Ok((id, rx))
    }

    /// Complete a request. A completion from another epoch is rejected and
    /// the pending request is left untouched.
    pub fn complete(
        &mut self,
        epoch: ConnectionEpoch,
        id: &str,
        response: Response,
    ) -> Result<bool> {
        if epoch != self.epoch {
            return Err(TransportError::StaleEpoch {
                stale: epoch,
                current: self.epoch,
            }
            .into());
        }
        match self.pending.remove(id) {
            Some(p) => {
                debug_assert_eq!(p.epoch, epoch);
                let _ = p.reply.send(response);
                Ok(true)
            }
            None => Ok(false),
        }
    }

    /// Fail every request whose deadline passed. Returns how many.
    pub fn expire(&mut self, now: Timestamp) -> usize {
        let expired: Vec<String> = self
            .pending
            .iter()
            .filter(|(_, p)| p.deadline <= now)
            .map(|(k, _)| k.clone())
            .collect();
        for k in &expired {
            if let Some(p) = self.pending.remove(k) {
                let _ = p
                    .reply
                    .send(Err(Error::Timeout(std::time::Duration::from_secs(0))));
            }
        }
        expired.len()
    }

    /// Fail every request because the connection closed.
    pub fn fail_all(&mut self, reason: TransportError) -> usize {
        let n = self.pending.len();
        for (_, p) in self.pending.drain() {
            let _ = p.reply.send(Err(Error::Transport(reason.clone())));
        }
        n
    }

    /// Number of outstanding requests.
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    /// Whether no requests are outstanding.
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn stale_epoch_cannot_complete_and_bounds_hold() {
        let mut t = RequestTable::new(ConnectionEpoch(2), 2);
        let (id, rx) = t.register(Timestamp(i64::MAX)).unwrap();
        let (_id2, _rx2) = t.register(Timestamp(i64::MAX)).unwrap();
        assert!(matches!(
            t.register(Timestamp(i64::MAX)),
            Err(Error::ResourceExhausted { .. })
        ));
        let err = t
            .complete(ConnectionEpoch(1), &id, Ok(Node::new("iq")))
            .unwrap_err();
        assert!(matches!(
            err,
            Error::Transport(TransportError::StaleEpoch { .. })
        ));
        assert_eq!(t.len(), 2);
        assert!(
            t.complete(ConnectionEpoch(2), &id, Ok(Node::new("iq")))
                .unwrap()
        );
        assert_eq!(rx.await.unwrap().unwrap().description, "iq");
        assert!(
            !t.complete(ConnectionEpoch(2), "nope", Ok(Node::new("iq")))
                .unwrap()
        );
    }

    #[tokio::test]
    async fn expiry_and_fail_all() {
        let mut t = RequestTable::new(ConnectionEpoch(1), 8);
        let (_a, ra) = t.register(Timestamp(10)).unwrap();
        let (_b, rb) = t.register(Timestamp(100)).unwrap();
        assert_eq!(t.expire(Timestamp(50)), 1);
        assert!(matches!(ra.await.unwrap(), Err(Error::Timeout(_))));
        assert_eq!(t.fail_all(TransportError::UnexpectedEof { buffered: 0 }), 1);
        assert!(matches!(rb.await.unwrap(), Err(Error::Transport(_))));
        assert!(t.is_empty());
    }
}
