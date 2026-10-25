//! Ordinary desktop media playback. Never routes view-once content here.
use std::{path::Path, process::Stdio, sync::Arc, time::Duration};
use whatsapp_api::{
    Error,
    model::ids::AccountId,
    session::AccountRecord,
    store::{EnvKeySource, OpenOptions, ProtocolStore},
    transport::media::{self, MediaKind},
};
fn store(cli: &super::Cli) -> Result<ProtocolStore, Error> {
    ProtocolStore::open(OpenOptions {
        path: cli.store.clone(),
        account_id: AccountId::new(cli.account.clone())?,
        key_source: Arc::new(EnvKeySource::new(cli.key_env.clone())),
        create: false,
    })
}
pub(super) fn list(cli: &super::Cli) -> Result<(), Error> {
    let entries = media::list(&store(cli)?)?;
    if cli.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&entries)
                .map_err(|e| Error::invalid("media", e.to_string()))?
        );
        return Ok(());
    }
    for entry in entries {
        println!(
            "media: id={}, kind={:?}, bytes={}",
            entry.id, entry.kind, entry.bytes
        );
    }
    Ok(())
}
pub(super) async fn open(
    cli: &super::Cli,
    path: &Path,
    id: &str,
    allow_live: bool,
) -> Result<(), Error> {
    use tokio::io::AsyncWriteExt;
    if !allow_live {
        return Err(Error::invalid("media", "requires --allow-live-request"));
    }
    let config = super::signup::config(path)?;
    let store = store(cli)?;
    let account = store
        .read(AccountRecord::load)?
        .ok_or(Error::NotRegistered)?;
    if !account.is_registered() || account.phone.as_ref() != Some(&config.phone) {
        return Err(Error::NotRegistered);
    }
    if media::list(&store)?
        .iter()
        .any(|entry| entry.id == id && entry.kind == MediaKind::Document)
    {
        return Err(Error::invalid(
            "media",
            "document retrieval requires media-save --output",
        ));
    }
    let (kind, plain) = media::open(&store, &config, id).await?;
    let mut command = tokio::process::Command::new("mpv");
    command.args([
        "--no-config",
        "--load-scripts=no",
        "--input-default-bindings=no",
        "--input-conf=/dev/null",
        "--osc=no",
        "--terminal=no",
        "--save-position-on-quit=no",
        "--resume-playback=no",
        "--keep-open=no",
        "--image-display-duration=30",
        "--title=WaRustAPI attachment",
    ]);
    match kind {
        MediaKind::Image | MediaKind::Sticker => {
            command.arg("--no-audio");
        }
        MediaKind::VoiceNote => {
            command.arg("--no-video");
        }
        MediaKind::Video | MediaKind::VideoNote => {}
        MediaKind::Document => {
            return Err(Error::invalid(
                "media",
                "document retrieval requires media-save --output",
            ));
        }
    }
    let mut child = command
        .arg("-")
        .env_remove(&cli.key_env)
        .env_remove("WHATSAPP_API_MASTER_KEY")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|_| Error::invalid("media", "cannot start mpv"))?;
    let mut input = child
        .stdin
        .take()
        .ok_or_else(|| Error::invalid("media", "viewer input unavailable"))?;
    tokio::time::timeout(Duration::from_secs(30), input.write_all(&plain))
        .await
        .map_err(|_| Error::invalid("media", "viewer input timeout"))?
        .map_err(|_| Error::invalid("media", "viewer input failed"))?;
    drop(input);
    drop(plain);
    println!(
        "media: authenticated {:?} piped to local player; no plaintext file",
        kind
    );
    let status = tokio::time::timeout(Duration::from_secs(610), child.wait())
        .await
        .map_err(|_| Error::invalid("media", "playback deadline exceeded"))?
        .map_err(|_| Error::invalid("media", "player wait failed"))?;
    if !status.success() {
        return Err(Error::invalid("media", "player reported failure"));
    }
    println!("media: player finished successfully");
    Ok(())
}

pub(super) async fn save(
    cli: &super::Cli,
    path: &Path,
    id: &str,
    output: &Path,
    allow_live: bool,
) -> Result<(), Error> {
    if !allow_live {
        return Err(Error::invalid("media", "requires --allow-live-request"));
    }
    let config = super::signup::config(path)?;
    let store = store(cli)?;
    let bytes = media::save(&store, &config, id, output).await?;
    println!("media: authenticated attachment exported to new private file; bytes={bytes}");
    Ok(())
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn save_requires_explicit_network_opt_in_before_files_or_store() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "whatsapp-cli",
            "media-save",
            "--config",
            "missing-request.json",
            "--id",
            "opaque",
            "--output",
            "unused-output.txt",
        ])
        .unwrap();
        let error = super::save(
            &cli,
            std::path::Path::new("missing-request.json"),
            "opaque",
            std::path::Path::new("unused-output.txt"),
            false,
        )
        .await
        .unwrap_err();
        assert!(error.to_string().contains("--allow-live-request"));
    }
}
