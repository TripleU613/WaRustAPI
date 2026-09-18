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
async fn actor(
    store: Arc<ProtocolStore>,
    config: LabRegistrationConfig,
    mut commands: mpsc::Receiver<Command>,
    status: watch::Sender<ConnectionStatus>,
    send_outbox: bool,
    ticket: Option<crate::model::ids::LocalMessageId>,
    mut start: impl FnMut(WorkerInput) -> tokio::task::JoinHandle<Result<()>>,
) {
    let mut epoch = ConnectionEpoch(0);
    let mut running: Option<Running> = None;
    let mut shutdown_ack = None;
    loop {
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
                        epoch=epoch.next();
                        set(&status,ConnectionState::Connecting{epoch},epoch);
                        let (stop_tx,stop_rx)=watch::channel(false);
                        let (ready_tx,ready_rx)=oneshot::channel();
                        let task=start(WorkerInput{store:store.clone(),config:config.clone(),epoch,stop:stop_rx,ready:ready_tx,send_outbox,ticket});
                        running=Some(Running{task,ready:Some(ready_rx),waiters:vec![reply],_stop:stop_tx});
                    },
                    Some(Command::Disconnect{reply})=> {
                        if let Some(mut run)=running.take() {
                            run.task.abort(); let _=(&mut run.task).await;
                            for waiter in run.waiters.drain(..) {let _=waiter.send(Err(Error::Cancelled));}
                        }
                        set(&status,ConnectionState::Disconnected{reason:"requested".into()},epoch);
                        let _=reply.send(());
                    },
                    Some(Command::Nudge)=> {},
                    Some(Command::Shutdown{reply})=>{shutdown_ack=Some(reply);break;},
                    None=>break,
                }
            }
            event=worker_event(&mut running)=> { match event {
            WorkerEvent::Finished(result)=> {
                let error=match result {Ok(Err(error))=>error,Ok(Ok(()))|Err(_)=>Error::Cancelled};
                let state=match &error {Error::NotRegistered=>ConnectionState::NeedsRegistration,
                    Error::Revoked(r)=>ConnectionState::Revoked(r.clone()),
                    _=>ConnectionState::Disconnected{reason:sanitized(&error)}};
                set(&status,state,epoch);
                if let Some(mut run)=running.take() {for waiter in run.waiters.drain(..) {
                    let error = match &error {
                        Error::NotRegistered => Error::NotRegistered,
                        Error::Revoked(reason) => Error::Revoked(reason.clone()),
                        _ => Error::invalid("connect",sanitized(&error)),
                    };
                    let _=waiter.send(Err(error));
                }}
            }
            WorkerEvent::Ready(ready)=> {
                if ready.is_ok() {
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
    set(&status, ConnectionState::Stopped, epoch);
    drop(store);
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
}
