//! Bounded comparison transport experiment; never implicitly selected by send.

use std::{path::Path, sync::Arc, time::Duration};
use whatsapp_api::{
    Error,
    model::{ids::AccountId, time::Timestamp},
    session::AccountRecord,
    signal::crypto::SignalSignatureVerifier,
    store::{EnvKeySource, OpenOptions, ProtocolStore},
    transport::{
        ConnectionEpoch,
        comparison::{ComparisonEndpoint, ComparisonGateway},
        handshake::{COMPARISON_CHAIN_POLICY, VerifiedHandshake},
        noise::StaticKeypair,
        socket::{SocketLimits, connect_comparison},
    },
};

pub(super) async fn run(
    cli: &super::Cli,
    path: &Path,
    allow_live: bool,
    login: bool,
    message: Option<&Path>,
) -> Result<(), Error> {
    if !allow_live {
        return Err(Error::invalid("transport", "requires --allow-live-request"));
    }
    let config = super::signup::config(path)?;
    let message = message
        .map(whatsapp_api::transport::messaging::MessageRequest::read)
        .transpose()?;
    let store = ProtocolStore::open(OpenOptions {
        path: cli.store.clone(),
        account_id: AccountId::new(cli.account.clone())?,
        key_source: Arc::new(EnvKeySource::new(cli.key_env.clone())),
        create: false,
    })?;
    let account = store
        .read(AccountRecord::load)?
        .ok_or_else(|| Error::invalid("transport", "missing account"))?;
    if let Some(message) = &message {
        let previous: Option<serde_json::Value> = store.read(|tx| {
            tx.get_json(
                whatsapp_api::store::Table::Checkpoints,
                format!("message:{}:send", message.request_id).as_bytes(),
                1,
            )
        })?;
        if let Some(previous) = previous {
            if previous["to"] != message.to.as_str() || previous["text"] != message.text.as_str() {
                return Err(Error::invalid(
                    "message",
                    "request_id already belongs to a different message",
                ));
            }
            if previous["status"] == "server_acknowledged" {
                println!("transport: message already server_acknowledged; not sent again");
                return Ok(());
            }
            return Err(Error::invalid(
                "message",
                "saved send is refused or uncertain; no replay",
            ));
        }
    }
    if !account.is_registered() || account.phone.as_ref() != Some(&config.phone) {
        return Err(Error::invalid(
            "transport",
            "requires matching registered account",
        ));
    }
    let payload = login
        .then(|| whatsapp_api::transport::login::primary_login(&account, &config))
        .transpose()?;
    const LOGIN_KEY: &[u8] = b"comparison-primary-login-v1";
    if login {
        let previous: Option<serde_json::Value> =
            store.read(|tx| tx.get_json(whatsapp_api::store::Table::Checkpoints, LOGIN_KEY, 1))?;
        if previous.as_ref().is_some_and(|v| v["status"] != "success") {
            return Err(Error::invalid(
                "transport",
                "previous login experiment exists; inspect saved result before another attempt",
            ));
        }
    }
    let key = account
        .noise_static
        .as_ref()
        .ok_or_else(|| Error::invalid("transport", "missing saved Noise key"))?;
    let local = StaticKeypair::from_parts(key.0.private, key.0.public);
    let endpoint = ComparisonEndpoint::built_in(ComparisonGateway::Primary, 443)
        .map_err(|_| Error::invalid("transport", "invalid endpoint"))?;
    let limits = SocketLimits {
        connect_timeout: Duration::from_millis(config.transport.connect_timeout_ms),
        io_timeout: Duration::from_millis(config.transport.timeout_ms),
        ..Default::default()
    };
    let map = |e| Error::invalid("transport", format!("{e}"));
    eprintln!(
        "transport: one raw TCP connection, classic XX, certificate verification enabled; automatic retries disabled"
    );
    let socket = connect_comparison(&endpoint, ConnectionEpoch(1), limits)
        .await
        .map_err(map)?;
    eprintln!("transport: TCP connected; sending framed ClientHello");
    let verified = VerifiedHandshake::start(
        socket,
        &local,
        ConnectionEpoch(1),
        Timestamp::now(),
        &COMPARISON_CHAIN_POLICY,
        &SignalSignatureVerifier,
    )
    .await
    .map_err(map)?;
    println!(
        "transport: server certificate verified; intermediate_serial={}, leaf_serial={}",
        verified.chain().intermediate_serial,
        verified.chain().leaf_serial
    );
    if let Some(payload) = payload {
        use whatsapp_api::{
            binary::{Decoder, tokens::TokenDictionary, unpack_frame_payload},
            profile::Limits,
            store::Table,
        };
        store.write(|tx| {
            let previous: Option<serde_json::Value> = tx.get_json(Table::Checkpoints, LOGIN_KEY, 1)?;
            if previous.as_ref().is_some_and(|v| v["status"] != "success") { return Err(Error::invalid("transport", "login already reserved or refused")); }
            tx.put_json(Table::Checkpoints, LOGIN_KEY, 1, &serde_json::json!({"status":"outcome_unknown", "at":Timestamp::now(), "payload_bytes":payload.len()}))
        })?;
        eprintln!(
            "transport: sending one minimal primary login ({} bytes); unknown hardware fields omitted",
            payload.len()
        );
        let mut channel = verified.finish(&payload).await.map_err(map)?;
        let frame = channel.read().await.map_err(map)?.ok_or_else(|| {
            Error::invalid(
                "transport",
                "EOF awaiting login result; outcome preserved as unknown",
            )
        })?;
        let limits = Limits::default();
        let unpacked = unpack_frame_payload(&frame, &limits)
            .map_err(|_| Error::invalid("transport", "cannot unpack encrypted login response"))?;
        let node = Decoder::new(&unpacked, TokenDictionary::comparison_2_26_37_70(), &limits)
            .decode_node()
            .map_err(|_| {
                Error::invalid(
                    "transport",
                    "cannot decode login response with comparison dictionary",
                )
            })?;
        let status = match node.description.as_str() {
            "success" => "success",
            "failure" => "failure",
            "stream:error" => "stream_error",
            _ => "unrecognized",
        };
        let reason = node
            .get_attr("reason")
            .and_then(|v| v.as_text())
            .filter(|v| v.len() <= 5 && v.bytes().all(|b| b.is_ascii_digit()));
        let result = serde_json::json!({"status":status,"reason":reason,"at":Timestamp::now(),"message_sent":false});
        store.write(|tx| {
            tx.put_json(Table::Checkpoints, LOGIN_KEY, 1, &result)?;
            if status == "success" {
                let mut account = AccountRecord::load_in_write(tx)?
                    .ok_or_else(|| Error::invalid("transport", "account disappeared"))?;
                account.last_authenticated_at = Some(Timestamp::now());
                account.save(tx)?;
            }
            Ok(())
        })?;
        println!(
            "transport: login status={status}, reason={}; message not sent",
            reason.unwrap_or("none")
        );
        if status != "success" {
            return Err(Error::invalid(
                "transport",
                "login not accepted; saved result must be inspected before retrying",
            ));
        }
        if status == "success"
            && let Some(message) = &message
        {
            let recipient = message.recipient()?;
            let saved_keys: Option<Vec<u8>> = store.read(|tx| {
                tx.get_json(
                    Table::Checkpoints,
                    format!("message:{}:keys", message.request_id).as_bytes(),
                    1,
                )
            })?;
            let (reply, pending) = if let Some(bytes) = saved_keys {
                let decoded = unpack_frame_payload(&bytes, &limits)?;
                (
                    Decoder::new(&decoded, TokenDictionary::comparison_2_26_37_70(), &limits)
                        .decode_node()?,
                    Vec::new(),
                )
            } else {
                let query = whatsapp_api::transport::messaging::key_request(
                    &recipient,
                    &format!("{}-keys", message.request_id),
                );
                whatsapp_api::transport::messaging::write_node(&mut channel, &query)
                    .await
                    .map_err(map)?;
                let (reply, pending) = whatsapp_api::transport::messaging::wait_for(
                    &mut channel,
                    "iq",
                    &format!("{}-keys", message.request_id),
                )
                .await
                .map_err(map)?;
                (reply, pending)
            };
            let dict = TokenDictionary::comparison_2_26_37_70();
            store.write(|tx| {
                let bytes = whatsapp_api::binary::Encoder::new(dict)
                    .encode(&reply)
                    .map_err(|_| Error::invalid("transport", "cannot preserve key reply"))?;
                tx.put_json(
                    Table::Checkpoints,
                    format!("message:{}:keys", message.request_id).as_bytes(),
                    1,
                    &bytes,
                )?;
                for (i, node) in pending.iter().enumerate() {
                    let bytes = whatsapp_api::binary::Encoder::new(dict)
                        .encode(node)
                        .map_err(|_| {
                            Error::invalid("transport", "cannot preserve pending stanza")
                        })?;
                    tx.put_json(
                        Table::Checkpoints,
                        format!("message:{}:pending:{i}", message.request_id).as_bytes(),
                        1,
                        &bytes,
                    )?;
                }
                Ok(())
            })?;
            let kind = reply
                .get_attr("type")
                .and_then(|v| v.as_text())
                .filter(|v| matches!(*v, "result" | "error"))
                .unwrap_or("unknown");
            let user = reply.child("list").and_then(|v| v.child("user"));
            println!(
                "transport: recipient key query type={kind}; user_present={}, identity_present={}, signed_key_present={}, one_time_key_present={}; response saved encrypted",
                user.is_some(),
                user.is_some_and(|v| v.child("identity").is_some()),
                user.is_some_and(|v| v.child("skey").is_some()),
                user.is_some_and(|v| v.child("key").is_some())
            );
            super::transport_send::send(&store, &account, &mut channel, message, &reply).await?;
        }
    } else {
        println!(
            "transport: certificate probe complete; account login not attempted, message not sent"
        );
    }
    Ok(())
}
