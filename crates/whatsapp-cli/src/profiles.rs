//! Named account profiles: one private JSON file per account under the
//! profiles directory (default `lab/profiles`, ignored by Git), naming the
//! store, the local account id, the private request configuration and the
//! environment variable that holds that account's master key. `--profile NAME`
//! fills the global `--store`, `--account`, `--key-env` and the per-command
//! `--config` defaults so two accounts never share a store or a key variable.
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use whatsapp_api::Error;

/// One account's locations. Contains no secrets: the master key stays in the
/// named environment variable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    /// Protocol database path.
    pub store: PathBuf,
    /// Local account identifier inside that store.
    pub account: String,
    /// Private request/transport configuration (`--config` default).
    #[serde(default)]
    pub config: Option<PathBuf>,
    /// Environment variable holding this account's hex master key.
    pub key_env: String,
}

fn invalid(msg: impl Into<String>) -> Error {
    Error::invalid("profile", msg.into())
}

fn validate_name(name: &str) -> Result<(), Error> {
    if name.is_empty()
        || name.len() > 64
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(invalid(
            "profile names are 1-64 ASCII letters, digits, '-' or '_'",
        ));
    }
    Ok(())
}

fn path(dir: &Path, name: &str) -> Result<PathBuf, Error> {
    validate_name(name)?;
    Ok(dir.join(format!("{name}.json")))
}

/// Load `NAME.json` from `dir`.
pub fn load(dir: &Path, name: &str) -> Result<Profile, Error> {
    let path = path(dir, name)?;
    let text = std::fs::read_to_string(&path)
        .map_err(|e| invalid(format!("cannot read {}: {e}", path.display())))?;
    let profile: Profile = serde_json::from_str(&text)
        .map_err(|e| invalid(format!("{} is not a profile: {e}", path.display())))?;
    validate(&profile)?;
    Ok(profile)
}

fn validate(profile: &Profile) -> Result<(), Error> {
    if profile.account.is_empty() {
        return Err(invalid("account id must not be empty"));
    }
    if profile.key_env.is_empty()
        || !profile
            .key_env
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err(invalid("key_env must be an environment variable name"));
    }
    Ok(())
}

/// Write `NAME.json` into `dir` (created if missing) with private permissions.
/// Refuses to overwrite an existing profile.
pub fn save(dir: &Path, name: &str, profile: &Profile) -> Result<PathBuf, Error> {
    validate(profile)?;
    let path = path(dir, name)?;
    if path.exists() {
        return Err(invalid(format!("{} already exists", path.display())));
    }
    std::fs::create_dir_all(dir)
        .map_err(|e| invalid(format!("cannot create {}: {e}", dir.display())))?;
    let text = serde_json::to_string_pretty(profile).map_err(|e| invalid(e.to_string()))?;
    std::fs::write(&path, text)
        .map_err(|e| invalid(format!("cannot write {}: {e}", path.display())))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| invalid(format!("cannot chmod {}: {e}", path.display())))?;
    }
    Ok(path)
}

/// All profiles in `dir`, sorted by name. A missing directory is empty.
pub fn list(dir: &Path) -> Result<Vec<(String, Profile)>, Error> {
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(out),
        Err(e) => return Err(invalid(format!("cannot read {}: {e}", dir.display()))),
    };
    for entry in entries {
        let entry = entry.map_err(|e| invalid(e.to_string()))?;
        let file = entry.path();
        if file.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Some(name) = file.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if validate_name(name).is_err() {
            continue;
        }
        out.push((name.to_owned(), load(dir, name)?));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(store: &str, key_env: &str) -> Profile {
        Profile {
            store: PathBuf::from(store),
            account: "acct".into(),
            config: Some(PathBuf::from("lab/x/request.json")),
            key_env: key_env.into(),
        }
    }

    #[test]
    fn save_load_list_and_refuse_overwrite() {
        let dir = tempfile::tempdir().unwrap();
        let p = sample("lab/x/signup.redb", "KEY_A");
        save(dir.path(), "a", &p).unwrap();
        assert_eq!(load(dir.path(), "a").unwrap(), p);
        assert!(save(dir.path(), "a", &p).is_err(), "no overwrite");
        save(dir.path(), "b", &sample("lab/y/signup.redb", "KEY_B")).unwrap();
        let names: Vec<_> = list(dir.path())
            .unwrap()
            .into_iter()
            .map(|(n, _)| n)
            .collect();
        assert_eq!(names, vec!["a", "b"]);
        assert!(list(&dir.path().join("missing")).unwrap().is_empty());
    }

    #[test]
    fn rejects_bad_names_and_key_envs() {
        let dir = tempfile::tempdir().unwrap();
        assert!(save(dir.path(), "../x", &sample("s", "K")).is_err());
        assert!(save(dir.path(), "", &sample("s", "K")).is_err());
        assert!(save(dir.path(), "ok", &sample("s", "BAD-NAME")).is_err());
        assert!(save(dir.path(), "ok", &sample("s", "")).is_err());
    }
}
