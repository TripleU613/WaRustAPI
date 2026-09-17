//! Account runtime: one actor owns mutable protocol state (README section 8.1).
//!
//! The actor receives commands over a bounded channel, publishes connection
//! state through a watch channel, and tags every connection attempt with a
//! fresh [`ConnectionEpoch`]. Request correlation is bounded and epoch-tagged.
//! For the current profile no socket is opened: `connect` fails with a typed
//! error before any network activity because the required facts are absent.

pub mod correlation;

use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot, watch};

use crate::error::{Error, Result};
use crate::model::time::Timestamp;
use crate::profile::ProtocolProfile;
use crate::session::AccountRecord;
use crate::store::ProtocolStore;
use crate::sync::RevocationReason;
use crate::transport::ConnectionEpoch;

/// Live connection state (README section 14).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ConnectionState {
    /// Runtime running, no connection requested.
    Stopped,
    /// Connection attempt in progress.
    Connecting {
        /// Epoch of the attempt.
        epoch: ConnectionEpoch,
    },
    /// Authenticated and ready.
    Ready {
        /// Epoch of the live connection.
        epoch: ConnectionEpoch,
    },
    /// Waiting before a reconnect attempt.
    Backoff {
        /// Earliest retry time.
        until: Timestamp,
        /// Consecutive attempt count.
        attempt: u32,
    },
    /// Disconnected; not reconnecting.
    Disconnected {
        /// Sanitized reason.
        reason: String,
    },
    /// The account is not registered; connecting is impossible.
    NeedsRegistration,
    /// The session was revoked.
    Revoked(RevocationReason),
}

/// Connection status with its epoch and a sanitized reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionStatus {
    /// Current state.
    pub state: ConnectionState,
    /// Current epoch counter.
    pub epoch: ConnectionEpoch,
    /// When the state last changed.
    pub changed_at: Timestamp,
}

enum Command {
    Connect { reply: oneshot::Sender<Result<()>> },
    Disconnect { reply: oneshot::Sender<()> },
    Nudge,
    Shutdown { reply: oneshot::Sender<()> },
}

/// Handle to the account actor. Cloning clones the handle, not the actor.
#[derive(Debug, Clone)]
pub struct RuntimeHandle {
    cmd: mpsc::Sender<Command>,
    status: watch::Receiver<ConnectionStatus>,
}

impl RuntimeHandle {
    /// Watch the connection status.
    pub fn status(&self) -> watch::Receiver<ConnectionStatus> {
        self.status.clone()
    }

    /// Current status snapshot.
    pub fn current(&self) -> ConnectionStatus {
        self.status.borrow().clone()
    }

    /// Request a connection and wait for the outcome (bounded by `timeout`).
    pub async fn connect(&self, timeout: Duration) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.cmd
            .send(Command::Connect { reply: tx })
            .await
            .map_err(|_| Error::Cancelled)?;
        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(r)) => r,
            Ok(Err(_)) => Err(Error::Cancelled),
            Err(_) => Err(Error::Timeout(timeout)),
        }
    }

    /// Stop reconnecting and close any connection. Durable state is preserved.
    pub async fn disconnect(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.cmd
            .send(Command::Disconnect { reply: tx })
            .await
            .map_err(|_| Error::Cancelled)?;
        rx.await.map_err(|_| Error::Cancelled)
    }

    /// Tell the actor new outbox work exists. Never blocks; drops if the
    /// bounded channel is full because the actor re-scans the outbox anyway.
    pub fn nudge(&self) {
        let _ = self.cmd.try_send(Command::Nudge);
    }

    /// Shut the actor down and wait until it has released its resources
    /// (including its store handle). Idempotent.
    pub async fn shutdown(&self) {
        let (tx, rx) = oneshot::channel();
        if self.cmd.send(Command::Shutdown { reply: tx }).await.is_ok() {
            let _ = rx.await;
        }
    }
}

/// Spawn the account actor.
pub fn spawn(store: Arc<ProtocolStore>, profile: Arc<ProtocolProfile>) -> RuntimeHandle {
    let (cmd_tx, cmd_rx) = mpsc::channel(profile.limits.max_pending_requests.max(1) as usize);
    let (status_tx, status_rx) = watch::channel(ConnectionStatus {
        state: ConnectionState::Stopped,
        epoch: ConnectionEpoch(0),
        changed_at: Timestamp::now(),
    });
    tokio::spawn(actor(store, profile, cmd_rx, status_tx));
    RuntimeHandle {
        cmd: cmd_tx,
        status: status_rx,
    }
}

async fn actor(
    store: Arc<ProtocolStore>,
    profile: Arc<ProtocolProfile>,
    mut cmd_rx: mpsc::Receiver<Command>,
    status_tx: watch::Sender<ConnectionStatus>,
) {
    let mut epoch = ConnectionEpoch(0);
    let mut shutdown_ack: Option<oneshot::Sender<()>> = None;
    let set = |status_tx: &watch::Sender<ConnectionStatus>,
               state: ConnectionState,
               epoch: ConnectionEpoch| {
        let _ = status_tx.send(ConnectionStatus {
            state,
            epoch,
            changed_at: Timestamp::now(),
        });
    };
    while let Some(cmd) = cmd_rx.recv().await {
        match cmd {
            Command::Connect { reply } => {
                epoch = epoch.next();
                set(&status_tx, ConnectionState::Connecting { epoch }, epoch);
                let result = connect_once(&store, &profile).await;
                match &result {
                    Ok(()) => set(&status_tx, ConnectionState::Ready { epoch }, epoch),
                    Err(Error::NotRegistered | Error::RegistrationActionRequired(_)) => {
                        set(&status_tx, ConnectionState::NeedsRegistration, epoch)
                    }
                    Err(Error::Revoked(r)) => {
                        set(&status_tx, ConnectionState::Revoked(r.clone()), epoch)
                    }
                    Err(e) => set(
                        &status_tx,
                        ConnectionState::Disconnected {
                            reason: sanitized(e),
                        },
                        epoch,
                    ),
                }
                let _ = reply.send(result);
            }
            Command::Disconnect { reply } => {
                set(
                    &status_tx,
                    ConnectionState::Disconnected {
                        reason: "requested".into(),
                    },
                    epoch,
                );
                let _ = reply.send(());
            }
            Command::Nudge => {
                // No transmission path exists for this profile; queued
                // entries remain Queued and are visible through the outbox.
            }
            Command::Shutdown { reply } => {
                shutdown_ack = Some(reply);
                break;
            }
        }
    }
    set(&status_tx, ConnectionState::Stopped, epoch);
    // Release the store before acknowledging so a caller can reopen it.
    drop(store);
    drop(profile);
    if let Some(ack) = shutdown_ack {
        let _ = ack.send(());
    }
}

fn sanitized(e: &Error) -> String {
    match e {
        Error::ProfileIncomplete { missing, .. } => {
            format!("profile incomplete: {}", missing.join(", "))
        }
        Error::Unsupported { capability, .. } => format!("unsupported: {capability:?}"),
        Error::Transport(t) => format!("transport: {t}"),
        Error::Timeout(d) => format!("timeout after {d:?}"),
        other => other.to_string().chars().take(120).collect(),
    }
}

/// One connection attempt. Order: registered account check, then profile
/// facts, then (future) socket + Noise + login. Never re-registers.
async fn connect_once(store: &Arc<ProtocolStore>, profile: &ProtocolProfile) -> Result<()> {
    let account = store.run_blocking(|s| s.read(AccountRecord::load)).await?;
    match account {
        Some(a) if a.is_registered() => {}
        Some(a) if a.registration == crate::session::RegistrationStatus::Revoked => {
            return Err(Error::Revoked(RevocationReason::ServerLoggedOut {
                code: None,
            }));
        }
        _ => return Err(Error::NotRegistered),
    }
    profile.require_connect_facts()?;
    Err(Error::unsupported(
        crate::capabilities::Capability::Connect,
        "login payload and transport endpoint are not observed for this profile",
    ))
}
