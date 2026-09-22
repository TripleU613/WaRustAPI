//! Explicit comparison driver for the normal account actor API.
use super::*;
use crate::registration::request::LabRegistrationConfig;

/// Spawn an opt-in live-capable actor. No network occurs until `connect`.
pub fn spawn_comparison(
    store: Arc<ProtocolStore>,
    profile: Arc<ProtocolProfile>,
    config: LabRegistrationConfig,
    send_outbox: bool,
    ticket: Option<crate::model::ids::LocalMessageId>,
) -> RuntimeHandle {
    let (cmd_tx, cmd_rx) = mpsc::channel(profile.limits.max_pending_requests.max(1) as usize);
    let (status_tx, status_rx) = watch::channel(ConnectionStatus {
        state: ConnectionState::Stopped,
        epoch: ConnectionEpoch(0),
        changed_at: Timestamp::now(),
    });
    tokio::spawn(actor(
        store,
        config,
        cmd_rx,
        status_tx,
        send_outbox,
        ticket,
        start_connection,
    ));
    RuntimeHandle {
        cmd: cmd_tx,
        status: status_rx,
    }
}
struct WorkerInput {
    store: Arc<ProtocolStore>,
    config: LabRegistrationConfig,
    epoch: ConnectionEpoch,
    stop: watch::Receiver<bool>,
    ready: oneshot::Sender<()>,
    send_outbox: bool,
    ticket: Option<crate::model::ids::LocalMessageId>,
}
fn start_connection(input: WorkerInput) -> tokio::task::JoinHandle<Result<()>> {
    tokio::spawn(crate::transport::connection::run(
        input.store,
        input.config,
        input.epoch,
        input.stop,
        input.ready,
        input.send_outbox,
        input.ticket,
    ))
}
struct Running {
    task: tokio::task::JoinHandle<Result<()>>,
    ready: Option<oneshot::Receiver<()>>,
    waiters: Vec<oneshot::Sender<Result<()>>>,
    _stop: watch::Sender<bool>,
}
enum WorkerEvent {
    Finished(std::result::Result<Result<()>, tokio::task::JoinError>),
    Ready(std::result::Result<(), oneshot::error::RecvError>),
}
async fn worker_event(running: &mut Option<Running>) -> WorkerEvent {
    let Some(run) = running.as_mut() else {
        return std::future::pending().await;
    };
    if let Some(ready) = run.ready.as_mut() {
        tokio::select! {
            result = &mut run.task => WorkerEvent::Finished(result),
            result = ready => WorkerEvent::Ready(result),
        }
    } else {
        WorkerEvent::Finished((&mut run.task).await)
    }
}
impl Drop for Running {
    fn drop(&mut self) {
        self.task.abort();
    }
}
fn set(status: &watch::Sender<ConnectionStatus>, state: ConnectionState, epoch: ConnectionEpoch) {
    let _ = status.send(ConnectionStatus {
        state,
        epoch,
        changed_at: Timestamp::now(),
    });
}
/// A scheduled automatic reconnect. Callers still waiting on `connect` stay
/// attached until the retry succeeds, fails terminally or is cancelled.
struct Backoff {
    retry_at: tokio::time::Instant,
    waiters: Vec<oneshot::Sender<Result<()>>>,
}
/// Everything a fresh connection attempt needs besides the epoch and waiters.
struct Launcher {
    store: Arc<ProtocolStore>,
    config: LabRegistrationConfig,
    send_outbox: bool,
    ticket: Option<crate::model::ids::LocalMessageId>,
}
impl Launcher {
    fn launch(
        &self,
        start: &mut impl FnMut(WorkerInput) -> tokio::task::JoinHandle<Result<()>>,
        epoch: &mut ConnectionEpoch,
        status: &watch::Sender<ConnectionStatus>,
        waiters: Vec<oneshot::Sender<Result<()>>>,
    ) -> Running {
        *epoch = epoch.next();
        set(
            status,
            ConnectionState::Connecting { epoch: *epoch },
            *epoch,
        );
        let (stop_tx, stop_rx) = watch::channel(false);
        let (ready_tx, ready_rx) = oneshot::channel();
        let task = start(WorkerInput {
            store: self.store.clone(),
            config: self.config.clone(),
            epoch: *epoch,
            stop: stop_rx,
            ready: ready_tx,
            send_outbox: self.send_outbox,
            ticket: self.ticket,
        });
        Running {
            task,
            ready: Some(ready_rx),
            waiters,
            _stop: stop_tx,
        }
    }
}
fn jitter_byte() -> u8 {
    let mut b = [0u8; 1];
    // A failed random read only removes jitter; it never changes the bounds.
    let _ = getrandom::fill(&mut b);
    b[0]
}
fn waiter_error(error: &Error) -> Error {
    match error {
        Error::NotRegistered => Error::NotRegistered,
        Error::Revoked(reason) => Error::Revoked(reason.clone()),
        _ => Error::invalid("connect", sanitized(error)),
    }
}
/// Actor loop. Reconnect policy (docs/design/messaging.md, section 14): only
/// `Error::is_recoverable_by_reconnect` failures schedule a retry, each with a
/// fresh epoch and full handshake, bounded by `transport.reconnect`. Refusals,
/// revocation, missing registration, worker panics, uncertain login outcomes
/// and explicit disconnects never reconnect. Consecutive attempts reset only
/// after the server activates a session; an explicit `connect` also resets them.
async fn actor(
    store: Arc<ProtocolStore>,
    config: LabRegistrationConfig,
    mut commands: mpsc::Receiver<Command>,
    status: watch::Sender<ConnectionStatus>,
    send_outbox: bool,
    ticket: Option<crate::model::ids::LocalMessageId>,
    mut start: impl FnMut(WorkerInput) -> tokio::task::JoinHandle<Result<()>>,
) {
    let policy = config.transport.reconnect;
    let launcher = Launcher {
        store,
        config,
        send_outbox,
        ticket,
    };
    let mut epoch = ConnectionEpoch(0);
    let mut running: Option<Running> = None;
    let mut backoff: Option<Backoff> = None;
    let mut attempts: u32 = 0;
    let mut shutdown_ack = None;
    loop {
        let retry_at = backoff.as_ref().map(|b| b.retry_at);
        tokio::select! {
            biased;
            command=commands.recv()=> {
                match command {
                    Some(Command::Connect{reply})=> {
                        if let Some(run)=&mut running {
                            if run.ready.is_none() && matches!(status.borrow().state,ConnectionState::Ready{..}) {let _=reply.send(Ok(()));}
                            else if run.waiters.len()<64 {run.waiters.push(reply);}
                            else {let _=reply.send(Err(Error::invalid("connect","too many waiting callers")));}
                            continue;
                        }
                        // An explicit request ends any scheduled wait and restarts the attempt budget.
                        let mut waiters=backoff.take().map(|b|b.waiters).unwrap_or_default();
                        if waiters.len()<64 {waiters.push(reply);} else {let _=reply.send(Err(Error::invalid("connect","too many waiting callers")));}
                        attempts=0;
                        running=Some(launcher.launch(&mut start,&mut epoch,&status,waiters));
                    },
                    Some(Command::Disconnect{reply})=> {
                        if let Some(mut run)=running.take() {
                            run.task.abort(); let _=(&mut run.task).await;
                            for waiter in run.waiters.drain(..) {let _=waiter.send(Err(Error::Cancelled));}
                        }
                        if let Some(b)=backoff.take() {for waiter in b.waiters {let _=waiter.send(Err(Error::Cancelled));}}
                        attempts=0;
                        set(&status,ConnectionState::Disconnected{reason:"requested".into()},epoch);
                        let _=reply.send(());
                    },
                    Some(Command::Nudge)=> {
                        // New work: retry now instead of waiting; the attempt budget still applies.
                        if let Some(b)=backoff.as_mut() {b.retry_at=tokio::time::Instant::now();}
                    },
                    Some(Command::Shutdown{reply})=>{shutdown_ack=Some(reply);break;},
                    None=>break,
                }
            }
            _ = tokio::time::sleep_until(retry_at.unwrap_or_else(tokio::time::Instant::now)), if retry_at.is_some() => {
                let waiters=backoff.take().map(|b|b.waiters).unwrap_or_default();
                running=Some(launcher.launch(&mut start,&mut epoch,&status,waiters));
            }
            event=worker_event(&mut running)=> { match event {
            WorkerEvent::Finished(result)=> {
                let error=match result {Ok(Err(error))=>error,Ok(Ok(()))|Err(_)=>Error::Cancelled};
                let Some(mut run)=running.take() else {continue};
                if error.is_recoverable_by_reconnect() && attempts<policy.max_attempts {
                    attempts+=1;
                    let delay=policy.delay(attempts,jitter_byte());
                    set(&status,ConnectionState::Backoff{until:Timestamp::now().saturating_add(delay),attempt:attempts},epoch);
                    backoff=Some(Backoff{retry_at:tokio::time::Instant::now()+delay,waiters:std::mem::take(&mut run.waiters)});
                    continue;
                }
                attempts=0;
                let state=match &error {Error::NotRegistered=>ConnectionState::NeedsRegistration,
                    Error::Revoked(r)=>ConnectionState::Revoked(r.clone()),
                    _=>ConnectionState::Disconnected{reason:sanitized(&error)}};
                set(&status,state,epoch);
                for waiter in run.waiters.drain(..) {let _=waiter.send(Err(waiter_error(&error)));}
            }
            WorkerEvent::Ready(ready)=> {
                if ready.is_ok() {
                    attempts=0;
                    set(&status,ConnectionState::Ready{epoch},epoch);
                    if let Some(run)=&mut running {run.ready=None;for waiter in run.waiters.drain(..){let _=waiter.send(Ok(()));}}
                } else if let Some(run)=&mut running {run.ready=None;}
            }
            }}
        }
    }
    if let Some(mut run) = running.take() {
        run.task.abort();
        let _ = (&mut run.task).await;
    }
    if let Some(b) = backoff.take() {
        for waiter in b.waiters {
            let _ = waiter.send(Err(Error::Cancelled));
        }
    }
    set(&status, ConnectionState::Stopped, epoch);
    drop(launcher);
    if let Some(reply) = shutdown_ack {
        let _ = reply.send(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn late_readiness_after_disconnect_cannot_complete_a_new_epoch() {
        use crate::store::{MasterKey, OpenOptions, StaticKeySource};
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(
            ProtocolStore::open(OpenOptions {
                path: dir.path().join("cancel.redb"),
                account_id: crate::model::ids::AccountId::new("test").unwrap(),
                key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([6; 32]))),
                create: true,
            })
            .unwrap(),
        );
        let weak = Arc::downgrade(&store);
        let (cmd, commands) = mpsc::channel(8);
        let (status, receiver) = watch::channel(ConnectionStatus {
            state: ConnectionState::Stopped,
            epoch: ConnectionEpoch(0),
            changed_at: Timestamp::now(),
        });
        let runtime = RuntimeHandle {
            cmd,
            status: receiver,
        };
        let config = serde_json::from_str(crate::registration::request::TEMPLATE_JSON).unwrap();
        let (ready_tx, mut ready_rx) = mpsc::unbounded_channel();
        let actor = tokio::spawn(actor(
            store,
            config,
            commands,
            status,
            false,
            None,
            move |input| {
                ready_tx.send((input.epoch, input.ready)).unwrap();
                tokio::spawn(async move {
                    // Keep the actual account handle alive until worker cancellation.
                    let _store = input.store;
                    std::future::pending::<Result<()>>().await
                })
            },
        ));
        let first = tokio::spawn({
            let runtime = runtime.clone();
            async move { runtime.connect(Duration::from_secs(2)).await }
        });
        let (epoch, late_ready) = ready_rx.recv().await.unwrap();
        assert_eq!(epoch, ConnectionEpoch(1));
        runtime.disconnect().await.unwrap();
        assert!(matches!(first.await.unwrap(), Err(Error::Cancelled)));
        let second = tokio::spawn({
            let runtime = runtime.clone();
            async move { runtime.connect(Duration::from_secs(2)).await }
        });
        let (epoch, current_ready) = ready_rx.recv().await.unwrap();
        assert_eq!(epoch, ConnectionEpoch(2));
        assert!(late_ready.send(()).is_err());
        assert_eq!(
            runtime.current().state,
            ConnectionState::Connecting { epoch }
        );
        assert!(!second.is_finished());
        current_ready.send(()).unwrap();
        second.await.unwrap().unwrap();
        assert_eq!(runtime.current().state, ConnectionState::Ready { epoch });
        runtime.shutdown().await;
        actor.await.unwrap();
        assert!(weak.upgrade().is_none());
    }

    #[tokio::test]
    async fn disconnect_and_worker_failure_preserve_reservations_across_epochs_and_restart() {
        use crate::{
            model::{
                ids::{AccountId, ChatId, ClientRequestId},
                jid::Jid,
                message::MessageContent,
                send::SendRequest,
            },
            outbox::Outbox,
            profile::Limits,
            store::{MasterKey, OpenOptions, StaticKeySource, Table},
        };
        for fail_worker in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let options = || OpenOptions {
                path: dir.path().join("actor.redb"),
                account_id: AccountId::new("test").unwrap(),
                key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([5; 32]))),
                create: true,
            };
            let store = Arc::new(ProtocolStore::open(options()).unwrap());
            let outbox = Outbox::new("default", Limits::default());
            let request = SendRequest {
                client_request_id: ClientRequestId::new("durable-send").unwrap(),
                chat: ChatId::from_jid(&Jid::parse("15550000000@s.whatsapp.net").unwrap()).unwrap(),
                content: MessageContent::Text {
                    body: "fixture".into(),
                    quoted: None,
                    style: Default::default(),
                },
            };
            let ticket = store.write(|tx| outbox.enqueue(tx, &request)).unwrap().0;
            let (cmd, commands) = mpsc::channel(8);
            let (status, status_rx) = watch::channel(ConnectionStatus {
                state: ConnectionState::Stopped,
                epoch: ConnectionEpoch(0),
                changed_at: Timestamp::now(),
            });
            let runtime = RuntimeHandle {
                cmd,
                status: status_rx,
            };
            let mut config: LabRegistrationConfig =
                serde_json::from_str(crate::registration::request::TEMPLATE_JSON).unwrap();
            config.token = zeroize::Zeroizing::new("offline-fixture".into());
            let failure = Arc::new(tokio::sync::Notify::new());
            let signal = failure.clone();
            let (observed_tx, mut observed) = mpsc::unbounded_channel();
            // Exercise the real actor and durable outbox reservation, replacing
            // only the network worker. No socket or server is involved.
            let handle = tokio::spawn(actor(
                store.clone(),
                config,
                commands,
                status,
                true,
                Some(ticket.local_id),
                move |input| {
                    let signal = signal.clone();
                    let observed = observed_tx.clone();
                    tokio::spawn(async move {
                        let candidate =
                            crate::transport::outgoing::begin_selected(&input.store, input.ticket)?;
                        observed.send((input.epoch, candidate.is_some())).unwrap();
                        let _ = input.ready.send(());
                        signal.notified().await;
                        panic!("injected connection worker failure");
                    })
                },
            ));
            runtime.connect(Duration::from_secs(2)).await.unwrap();
            assert_eq!(observed.recv().await.unwrap(), (ConnectionEpoch(1), true));
            let checkpoint = format!("outbox-key-query:{}", ticket.local_id);
            let saved = store
                .read(|tx| tx.get(Table::Checkpoints, checkpoint.as_bytes(), 1))
                .unwrap()
                .unwrap();
            if fail_worker {
                failure.notify_one();
                let mut state = runtime.status();
                tokio::time::timeout(Duration::from_secs(2), async {
                    loop {
                        if matches!(state.borrow().state, ConnectionState::Disconnected { .. }) {
                            break;
                        }
                        state.changed().await.unwrap();
                    }
                })
                .await
                .unwrap();
                assert_eq!(runtime.current().epoch, ConnectionEpoch(1));
            } else {
                runtime.disconnect().await.unwrap();
            }
            assert!(observed.try_recv().is_err(), "no implicit reconnect");
            runtime.connect(Duration::from_secs(2)).await.unwrap();
            assert_eq!(observed.recv().await.unwrap(), (ConnectionEpoch(2), false));
            assert_eq!(
                store
                    .read(|tx| tx.get(Table::Checkpoints, checkpoint.as_bytes(), 1))
                    .unwrap()
                    .unwrap(),
                saved
            );
            runtime.shutdown().await;
            handle.await.unwrap();
            drop(store);
            let reopened = ProtocolStore::open(options()).unwrap();
            assert_eq!(
                reopened
                    .read(|tx| tx.get(Table::Checkpoints, checkpoint.as_bytes(), 1))
                    .unwrap()
                    .unwrap(),
                saved
            );
            assert_eq!(
                reopened
                    .read(|tx| outbox.get(tx, ticket.local_id))
                    .unwrap()
                    .unwrap()
                    .intent,
                request
            );
            assert!(
                crate::transport::outgoing::begin_selected(&reopened, Some(ticket.local_id))
                    .unwrap()
                    .is_none()
            );
        }
    }

    #[tokio::test]
    async fn offline_build_typed_registration_error_and_clean_shutdown() {
        use crate::store::{MasterKey, OpenOptions, StaticKeySource};
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("actor.redb");
        let options = || OpenOptions {
            path: path.clone(),
            account_id: crate::model::ids::AccountId::new("test").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([5; 32]))),
            create: true,
        };
        let store = Arc::new(ProtocolStore::open(options()).unwrap());
        let weak = Arc::downgrade(&store);
        let mut config: LabRegistrationConfig =
            serde_json::from_str(crate::registration::request::TEMPLATE_JSON).unwrap();
        config.token = zeroize::Zeroizing::new("offline-fixture".into());
        let runtime = spawn_comparison(
            store,
            Arc::new(ProtocolProfile::android_reference().unwrap()),
            config,
            false,
            None,
        );
        assert_eq!(runtime.current().state, ConnectionState::Stopped);
        for epoch in 1..=2 {
            assert!(matches!(
                runtime.connect(Duration::from_secs(2)).await,
                Err(Error::NotRegistered)
            ));
            assert_eq!(runtime.current().state, ConnectionState::NeedsRegistration);
            assert_eq!(runtime.current().epoch, ConnectionEpoch(epoch));
            runtime.disconnect().await.unwrap();
        }
        runtime.shutdown().await;
        assert_eq!(runtime.current().state, ConnectionState::Stopped);
        assert!(weak.upgrade().is_none());
        drop(ProtocolStore::open(options()).unwrap());
    }

    /// One injected worker launch. The test decides when the worker reports
    /// readiness and how it ends, replacing only the network; the real actor,
    /// store and outbox reservation logic run unchanged.
    struct Launch {
        epoch: ConnectionEpoch,
        reserved_now: bool,
        ready: oneshot::Sender<()>,
        end: oneshot::Sender<Result<()>>,
    }
    struct Harness {
        runtime: RuntimeHandle,
        actor: tokio::task::JoinHandle<()>,
        launches: mpsc::UnboundedReceiver<Launch>,
    }
    fn fixture_config(
        reconnect: crate::registration::request::ReconnectSettings,
    ) -> LabRegistrationConfig {
        let mut config: LabRegistrationConfig =
            serde_json::from_str(crate::registration::request::TEMPLATE_JSON).unwrap();
        config.token = zeroize::Zeroizing::new("offline-fixture".into());
        config.transport.reconnect = reconnect;
        config
    }
    fn harness(
        store: Arc<ProtocolStore>,
        config: LabRegistrationConfig,
        ticket: Option<crate::model::ids::LocalMessageId>,
    ) -> Harness {
        let (cmd, commands) = mpsc::channel(8);
        let (status, status_rx) = watch::channel(ConnectionStatus {
            state: ConnectionState::Stopped,
            epoch: ConnectionEpoch(0),
            changed_at: Timestamp::now(),
        });
        let (launch_tx, launches) = mpsc::unbounded_channel();
        let actor = tokio::spawn(actor(
            store,
            config,
            commands,
            status,
            ticket.is_some(),
            ticket,
            move |input| {
                let launch_tx = launch_tx.clone();
                tokio::spawn(async move {
                    let reserved_now =
                        crate::transport::outgoing::begin_selected(&input.store, input.ticket)?
                            .is_some();
                    let (end, ended) = oneshot::channel();
                    launch_tx
                        .send(Launch {
                            epoch: input.epoch,
                            reserved_now,
                            ready: input.ready,
                            end,
                        })
                        .unwrap();
                    ended.await.unwrap_or(Ok(()))
                })
            },
        ));
        Harness {
            runtime: RuntimeHandle {
                cmd,
                status: status_rx,
            },
            actor,
            launches,
        }
    }
    async fn next_launch(h: &mut Harness) -> Launch {
        tokio::time::timeout(Duration::from_secs(5), h.launches.recv())
            .await
            .expect("launch within deadline")
            .expect("launch channel open")
    }
    async fn no_launch(h: &mut Harness, window: Duration) {
        assert!(
            tokio::time::timeout(window, h.launches.recv())
                .await
                .is_err(),
            "unexpected reconnect launch"
        );
    }
    async fn wait_state(
        status: &mut watch::Receiver<ConnectionStatus>,
        pred: impl Fn(&ConnectionState) -> bool,
    ) -> ConnectionStatus {
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                {
                    let current = status.borrow_and_update();
                    if pred(&current.state) {
                        return current.clone();
                    }
                }
                status.changed().await.unwrap();
            }
        })
        .await
        .expect("state within deadline")
    }
    fn peer_closed() -> Result<()> {
        Err(Error::Transport(
            crate::transport::TransportError::PeerClosed,
        ))
    }

    #[tokio::test]
    async fn recoverable_loss_reconnects_with_fresh_epochs_bounded_attempts_and_kept_reservation() {
        use crate::{
            model::{
                ids::{AccountId, ChatId, ClientRequestId},
                jid::Jid,
                message::MessageContent,
                send::SendRequest,
            },
            outbox::Outbox,
            profile::Limits,
            registration::request::ReconnectSettings,
            store::{MasterKey, OpenOptions, StaticKeySource, Table},
        };
        let dir = tempfile::tempdir().unwrap();
        let options = || OpenOptions {
            path: dir.path().join("reconnect.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([7; 32]))),
            create: true,
        };
        let store = Arc::new(ProtocolStore::open(options()).unwrap());
        let outbox = Outbox::new("default", Limits::default());
        let request = SendRequest {
            client_request_id: ClientRequestId::new("durable-send").unwrap(),
            chat: ChatId::from_jid(&Jid::parse("15550000000@s.whatsapp.net").unwrap()).unwrap(),
            content: MessageContent::Text {
                body: "fixture".into(),
                quoted: None,
                style: Default::default(),
            },
        };
        let ticket = store.write(|tx| outbox.enqueue(tx, &request)).unwrap().0;
        let policy = ReconnectSettings {
            max_attempts: 3,
            base_delay_ms: 100,
            max_delay_ms: 200,
        };
        let mut h = harness(store.clone(), fixture_config(policy), Some(ticket.local_id));
        let mut status = h.runtime.status();

        // Epoch 1 reaches Ready and reserves the key query for the selected send.
        let connect = tokio::spawn({
            let runtime = h.runtime.clone();
            async move { runtime.connect(Duration::from_secs(5)).await }
        });
        let first = next_launch(&mut h).await;
        assert_eq!(
            (first.epoch, first.reserved_now),
            (ConnectionEpoch(1), true)
        );
        first.ready.send(()).unwrap();
        connect.await.unwrap().unwrap();
        let checkpoint = format!("outbox-key-query:{}", ticket.local_id);
        let saved = store
            .read(|tx| tx.get(Table::Checkpoints, checkpoint.as_bytes(), 1))
            .unwrap()
            .unwrap();

        // Peer close after Ready: one bounded wait, then a fresh epoch that
        // must not select the reserved send again.
        let before = Timestamp::now();
        first.end.send(peer_closed()).unwrap();
        let waiting = wait_state(&mut status, |s| {
            matches!(s, ConnectionState::Backoff { .. })
        })
        .await;
        let ConnectionState::Backoff { until, attempt } = waiting.state else {
            unreachable!()
        };
        assert_eq!(attempt, 1);
        assert!(until >= before.saturating_add(Duration::from_millis(75)));
        assert!(until <= Timestamp::now().saturating_add(Duration::from_millis(100)));
        assert_eq!(
            waiting.epoch,
            ConnectionEpoch(1),
            "epoch advances only when a socket is attempted"
        );
        let second = next_launch(&mut h).await;
        assert_eq!(
            (second.epoch, second.reserved_now),
            (ConnectionEpoch(2), false)
        );
        assert_eq!(
            h.runtime.current().state,
            ConnectionState::Connecting {
                epoch: ConnectionEpoch(2)
            }
        );
        second.ready.send(()).unwrap();
        wait_state(&mut status, |s| {
            matches!(
                s,
                ConnectionState::Ready {
                    epoch: ConnectionEpoch(2)
                }
            )
        })
        .await;

        // A second loss after Ready restarts the count at attempt 1. Then three
        // consecutive failures without readiness exhaust the budget.
        second
            .end
            .send(Err(Error::Timeout(Duration::from_secs(1))))
            .unwrap();
        let waiting = wait_state(&mut status, |s| {
            matches!(s, ConnectionState::Backoff { .. })
        })
        .await;
        assert!(matches!(
            waiting.state,
            ConnectionState::Backoff { attempt: 1, .. }
        ));
        let mut expected_epoch = 3;
        for attempt in 2..=3 {
            let launch = next_launch(&mut h).await;
            assert_eq!(launch.epoch, ConnectionEpoch(expected_epoch));
            assert!(!launch.reserved_now);
            launch.end.send(peer_closed()).unwrap();
            let waiting = wait_state(&mut status, |s| {
                matches!(s, ConnectionState::Backoff { .. })
            })
            .await;
            assert!(
                matches!(waiting.state, ConnectionState::Backoff { attempt: a, .. } if a == attempt),
                "{:?}",
                waiting.state
            );
            expected_epoch += 1;
        }
        let last = next_launch(&mut h).await;
        assert_eq!(last.epoch, ConnectionEpoch(expected_epoch));
        last.end.send(peer_closed()).unwrap();
        let stopped = wait_state(&mut status, |s| {
            matches!(s, ConnectionState::Disconnected { .. })
        })
        .await;
        let ConnectionState::Disconnected { reason } = stopped.state else {
            unreachable!()
        };
        assert_eq!(reason, "transport: peer closed the connection");
        no_launch(&mut h, Duration::from_millis(400)).await;

        // The reservation survived every epoch byte-for-byte and the reopen.
        assert_eq!(
            store
                .read(|tx| tx.get(Table::Checkpoints, checkpoint.as_bytes(), 1))
                .unwrap()
                .unwrap(),
            saved
        );
        h.runtime.shutdown().await;
        h.actor.await.unwrap();
        drop(store);
        let reopened = ProtocolStore::open(options()).unwrap();
        assert_eq!(
            reopened
                .read(|tx| tx.get(Table::Checkpoints, checkpoint.as_bytes(), 1))
                .unwrap()
                .unwrap(),
            saved
        );
        assert!(
            crate::transport::outgoing::begin_selected(&reopened, Some(ticket.local_id))
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn refusals_and_disabled_policy_never_reconnect() {
        use crate::registration::request::ReconnectSettings;
        use crate::store::{MasterKey, OpenOptions, StaticKeySource};
        for (max_attempts, error, expect_needs_registration) in [
            (3u32, Err(Error::Authentication { code: None }), false),
            (3, Err(Error::NotRegistered), true),
            (
                3,
                Err(Error::invalid("connection", "server ended session")),
                false,
            ),
            (0, peer_closed(), false),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let store = Arc::new(
                ProtocolStore::open(OpenOptions {
                    path: dir.path().join("refusal.redb"),
                    account_id: crate::model::ids::AccountId::new("test").unwrap(),
                    key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([8; 32]))),
                    create: true,
                })
                .unwrap(),
            );
            let policy = ReconnectSettings {
                max_attempts,
                base_delay_ms: 10,
                max_delay_ms: 20,
            };
            let mut h = harness(store, fixture_config(policy), None);
            let mut status = h.runtime.status();
            let connect = tokio::spawn({
                let runtime = h.runtime.clone();
                async move { runtime.connect(Duration::from_secs(5)).await }
            });
            let launch = next_launch(&mut h).await;
            assert_eq!(launch.epoch, ConnectionEpoch(1));
            launch.ready.send(()).unwrap();
            connect.await.unwrap().unwrap();
            launch.end.send(error).unwrap();
            let ended = wait_state(&mut status, |s| {
                !matches!(
                    s,
                    ConnectionState::Ready { .. } | ConnectionState::Connecting { .. }
                )
            })
            .await;
            if expect_needs_registration {
                assert_eq!(ended.state, ConnectionState::NeedsRegistration);
            } else {
                assert!(
                    matches!(ended.state, ConnectionState::Disconnected { .. }),
                    "{:?}",
                    ended.state
                );
            }
            assert_eq!(ended.epoch, ConnectionEpoch(1));
            no_launch(&mut h, Duration::from_millis(200)).await;
            // An explicit connect afterwards is still allowed and gets a new epoch.
            let connect = tokio::spawn({
                let runtime = h.runtime.clone();
                async move { runtime.connect(Duration::from_secs(5)).await }
            });
            let launch = next_launch(&mut h).await;
            assert_eq!(launch.epoch, ConnectionEpoch(2));
            launch.end.send(Err(Error::NotRegistered)).unwrap();
            assert!(matches!(connect.await.unwrap(), Err(Error::NotRegistered)));
            no_launch(&mut h, Duration::from_millis(100)).await;
            h.runtime.shutdown().await;
            h.actor.await.unwrap();
        }
    }

    #[tokio::test]
    async fn waiters_span_backoff_while_nudge_connect_and_disconnect_control_the_wait() {
        use crate::registration::request::ReconnectSettings;
        use crate::store::{MasterKey, OpenOptions, StaticKeySource};
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(
            ProtocolStore::open(OpenOptions {
                path: dir.path().join("wait.redb"),
                account_id: crate::model::ids::AccountId::new("test").unwrap(),
                key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
                create: true,
            })
            .unwrap(),
        );
        let weak = Arc::downgrade(&store);
        // Delays far longer than the test: any launch inside a wait is explicit.
        let policy = ReconnectSettings {
            max_attempts: 5,
            base_delay_ms: 30_000,
            max_delay_ms: 60_000,
        };
        let mut h = harness(store, fixture_config(policy), None);
        let mut status = h.runtime.status();

        // Loss before readiness keeps the caller waiting through the backoff;
        // nudge retries at once and the caller completes on the new epoch.
        let connect = tokio::spawn({
            let runtime = h.runtime.clone();
            async move { runtime.connect(Duration::from_secs(5)).await }
        });
        let first = next_launch(&mut h).await;
        first.end.send(peer_closed()).unwrap();
        wait_state(&mut status, |s| {
            matches!(s, ConnectionState::Backoff { attempt: 1, .. })
        })
        .await;
        no_launch(&mut h, Duration::from_millis(150)).await;
        assert!(
            !connect.is_finished(),
            "caller keeps waiting during a bounded backoff"
        );
        h.runtime.nudge();
        let second = next_launch(&mut h).await;
        assert_eq!(second.epoch, ConnectionEpoch(2));
        second.ready.send(()).unwrap();
        connect.await.unwrap().unwrap();
        assert_eq!(
            h.runtime.current().state,
            ConnectionState::Ready {
                epoch: ConnectionEpoch(2)
            }
        );

        // Loss after readiness, then an explicit connect: it launches now and
        // restarts the attempt budget.
        second.end.send(peer_closed()).unwrap();
        wait_state(&mut status, |s| {
            matches!(s, ConnectionState::Backoff { attempt: 1, .. })
        })
        .await;
        let connect = tokio::spawn({
            let runtime = h.runtime.clone();
            async move { runtime.connect(Duration::from_secs(5)).await }
        });
        let third = next_launch(&mut h).await;
        assert_eq!(third.epoch, ConnectionEpoch(3));
        third.end.send(peer_closed()).unwrap();
        let waiting = wait_state(&mut status, |s| {
            matches!(s, ConnectionState::Backoff { .. })
        })
        .await;
        assert!(
            matches!(waiting.state, ConnectionState::Backoff { attempt: 1, .. }),
            "explicit connect reset the count"
        );
        assert!(!connect.is_finished());

        // Disconnect during the wait cancels the retry and the waiting caller.
        h.runtime.disconnect().await.unwrap();
        assert!(matches!(connect.await.unwrap(), Err(Error::Cancelled)));
        assert_eq!(
            h.runtime.current().state,
            ConnectionState::Disconnected {
                reason: "requested".into()
            }
        );
        h.runtime.nudge();
        no_launch(&mut h, Duration::from_millis(150)).await;

        // Shutdown during a wait releases the store and cancels waiters.
        let connect = tokio::spawn({
            let runtime = h.runtime.clone();
            async move { runtime.connect(Duration::from_secs(5)).await }
        });
        let fourth = next_launch(&mut h).await;
        assert_eq!(fourth.epoch, ConnectionEpoch(4));
        fourth.end.send(peer_closed()).unwrap();
        wait_state(&mut status, |s| {
            matches!(s, ConnectionState::Backoff { .. })
        })
        .await;
        h.runtime.shutdown().await;
        assert!(matches!(connect.await.unwrap(), Err(Error::Cancelled)));
        h.actor.await.unwrap();
        assert_eq!(h.runtime.current().state, ConnectionState::Stopped);
        assert!(weak.upgrade().is_none());
    }
}
