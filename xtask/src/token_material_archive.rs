//! Build token material archives from unpacked APK directories and logo sources.

use std::fs::{self, File};
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

pub fn run(
    from_dir: &Path,
    logo_from: &Path,
    out: &Path,
    verify_config: Option<&Path>,
) -> Result<(), String> {
    if out.exists() {
        return Err("refusing to overwrite existing output file".into());
    }

    let manifest_path = from_dir.join("AndroidManifest.xml");
    let dex_path = from_dir.join("classes.dex");

    if !manifest_path.exists() {
        return Err("AndroidManifest.xml not found in source directory".into());
    }
    if !dex_path.exists() {
        return Err("classes.dex not found in source directory".into());
    }

    let manifest = fs::read(&manifest_path).map_err(|e| e.to_string())?;
    let dex = fs::read(&dex_path).map_err(|e| e.to_string())?;

    let logo_entry = extract_logo(logo_from)?;
    let _ = extract_first_signature(from_dir)?;

    let resources_arsc =
        read_optional(&from_dir.join("resources.arsc")).map_err(|e| e.to_string())?;

    let output_file =
        File::create(out).map_err(|e| format!("failed to create output file: {e}"))?;
    #[cfg(unix)]
    fs::set_permissions(out, fs::Permissions::from_mode(0o600))
        .map_err(|e| format!("failed to set file permissions: {e}"))?;

    let mut zip = ZipWriter::new(output_file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    zip.start_file("AndroidManifest.xml", options)
        .map_err(|e| format!("failed to write manifest: {e}"))?;
    zip.write_all(&manifest).map_err(|e| e.to_string())?;
    eprintln!("AndroidManifest.xml ({})", manifest.len());

    zip.start_file("classes.dex", options)
        .map_err(|e| format!("failed to write dex: {e}"))?;
    zip.write_all(&dex).map_err(|e| e.to_string())?;
    eprintln!("classes.dex ({})", dex.len());

    if let Some(arsc) = resources_arsc {
        zip.start_file("resources.arsc", options)
            .map_err(|e| format!("failed to write resources.arsc: {e}"))?;
        zip.write_all(&arsc).map_err(|e| e.to_string())?;
        eprintln!("resources.arsc ({})", arsc.len());
    }

    for entry in fs::read_dir(from_dir.join("META-INF"))
        .map_err(|e| format!("failed to read META-INF: {e}"))?
    {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let file_name = path.file_name().ok_or("invalid file name")?;
        let file_name_str = file_name.to_string_lossy();
        if file_name_str.ends_with(".DSA")
            || file_name_str.ends_with(".RSA")
            || file_name_str.ends_with(".EC")
        {
            let sig_data = fs::read(&path).map_err(|e| e.to_string())?;
            let entry_name = format!("META-INF/{}", file_name_str);
            zip.start_file(entry_name.clone(), options)
                .map_err(|e| format!("failed to write {}: {e}", entry_name))?;
            zip.write_all(&sig_data).map_err(|e| e.to_string())?;
            eprintln!("{} ({})", entry_name, sig_data.len());
        }
    }

    let logo_data = extract_logo_data(logo_from)?;
    zip.start_file(&logo_entry, options)
        .map_err(|e| format!("failed to write logo: {e}"))?;
    zip.write_all(&logo_data).map_err(|e| e.to_string())?;
    eprintln!("{} ({})", logo_entry, logo_data.len());

    zip.finish()
        .map_err(|e| format!("failed to finalize zip: {e}"))?;

    println!("wrote {}", out.display());

    if let Some(config_path) = verify_config {
        verify_token(out, config_path)?;
    }

    Ok(())
}

fn extract_first_signature(dir: &Path) -> Result<(String, Vec<u8>), String> {
    let meta_inf = dir.join("META-INF");
    for entry in fs::read_dir(&meta_inf).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let file_name = path.file_name().ok_or("invalid file name")?;
        let file_name_str = file_name.to_string_lossy();
        if file_name_str.ends_with(".DSA")
            || file_name_str.ends_with(".RSA")
            || file_name_str.ends_with(".EC")
        {
            let data = fs::read(&path).map_err(|e| e.to_string())?;
            return Ok((format!("META-INF/{}", file_name_str), data));
        }
    }
    Err("no signature block found in META-INF".into())
}

fn extract_logo(logo_from: &Path) -> Result<String, String> {
    let candidates = [
        "res/drawable-hdpi/about_logo.png",
        "res/drawable-hdpi-v4/about_logo.png",
        "res/drawable-xxhdpi-v4/about_logo.png",
    ];

    let file = File::open(logo_from).map_err(|e| format!("failed to open logo source: {e}"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("failed to open zip: {e}"))?;

    for candidate in &candidates {
        if archive.by_name(candidate).is_ok() {
            return Ok(candidate.to_string());
        }
    }

    Err("no logo candidate found in source archive".into())
}

fn extract_logo_data(logo_from: &Path) -> Result<Vec<u8>, String> {
    let candidates = [
        "res/drawable-hdpi/about_logo.png",
        "res/drawable-hdpi-v4/about_logo.png",
        "res/drawable-xxhdpi-v4/about_logo.png",
    ];

    let file = File::open(logo_from).map_err(|e| format!("failed to open logo source: {e}"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("failed to open zip: {e}"))?;

    for candidate in &candidates {
        if let Ok(mut entry) = archive.by_name(candidate) {
            let mut data = Vec::new();
            entry.read_to_end(&mut data).map_err(|e| e.to_string())?;
            return Ok(data);
        }
    }

    Err("no logo candidate found in source archive".into())
}

fn read_optional(path: &Path) -> std::io::Result<Option<Vec<u8>>> {
    if path.exists() {
        fs::read(path).map(Some)
    } else {
        Ok(None)
    }
}

fn verify_token(archive_path: &Path, config_path: &Path) -> Result<(), String> {
    use whatsapp_api::registration::{
        request::{LabRegistrationConfig, MAX_CONFIG_BYTES},
        token_material::TokenMaterial,
    };

    let config_data =
        fs::read(config_path).map_err(|e| format!("failed to read config file: {e}"))?;

    if config_data.len() > MAX_CONFIG_BYTES as usize {
        return Err("config file exceeds maximum size".into());
    }

    let config: LabRegistrationConfig =
        serde_json::from_slice(&config_data).map_err(|e| format!("failed to parse config: {e}"))?;

    config
        .validate()
        .map_err(|e| format!("config validation failed: {e}"))?;

    let material = TokenMaterial::open(archive_path).map_err(|e| e.to_string())?;

    let derived = material.derive(&config.phone).map_err(|e| e.to_string())?;

    if derived.as_str() == config.token.as_str() {
        eprintln!("token matches material");
        Ok(())
    } else {
        eprintln!("token does not match material");
        Err("token does not match material".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as StdWrite;
    use tempfile::TempDir;

    #[test]
    fn build_archive_with_required_entries() {
        let temp = TempDir::new().unwrap();
        let source_dir = temp.path().join("source");
        fs::create_dir_all(&source_dir).unwrap();
        fs::create_dir_all(source_dir.join("META-INF")).unwrap();

        let mut manifest = File::create(source_dir.join("AndroidManifest.xml")).unwrap();
        manifest.write_all(b"manifest").unwrap();

        let mut dex = File::create(source_dir.join("classes.dex")).unwrap();
        dex.write_all(b"dex_data").unwrap();

        let mut sig = File::create(source_dir.join("META-INF/X.DSA")).unwrap();
        sig.write_all(b"signature").unwrap();

        let logo_zip = temp.path().join("logo.zip");
        let logo_file = File::create(&logo_zip).unwrap();
        let mut zip = ZipWriter::new(logo_file);
        let options = SimpleFileOptions::default();
        zip.start_file("res/drawable-hdpi-v4/about_logo.png", options)
            .unwrap();
        zip.write_all(b"logo_data").unwrap();
        zip.finish().unwrap();

        let out = temp.path().join("output.zip");
        run(&source_dir, &logo_zip, &out, None).unwrap();

        assert!(out.exists());
        #[cfg(unix)]
        {
            let perms = fs::metadata(&out).unwrap().permissions();
            assert_eq!(perms.mode() & 0o777, 0o600);
        }

        let file = File::open(&out).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();

        let names: Vec<String> = (0..archive.len())
            .filter_map(|i| {
                archive
                    .by_index(i)
                    .ok()
                    .and_then(|e| e.name().ok().map(|n| n.into_owned()))
            })
            .collect();

        assert!(names.contains(&"AndroidManifest.xml".to_string()));
        assert!(names.contains(&"classes.dex".to_string()));
        assert!(names.contains(&"META-INF/X.DSA".to_string()));
        assert!(names.contains(&"res/drawable-hdpi-v4/about_logo.png".to_string()));
    }

    #[test]
    fn refuse_to_overwrite_existing_file() {
        let temp = TempDir::new().unwrap();
        let source_dir = temp.path().join("source");
        fs::create_dir_all(&source_dir).unwrap();
        fs::create_dir_all(source_dir.join("META-INF")).unwrap();

        let mut manifest = File::create(source_dir.join("AndroidManifest.xml")).unwrap();
        manifest.write_all(b"manifest").unwrap();

        let mut dex = File::create(source_dir.join("classes.dex")).unwrap();
        dex.write_all(b"dex_data").unwrap();

        let mut sig = File::create(source_dir.join("META-INF/X.DSA")).unwrap();
        sig.write_all(b"signature").unwrap();

        let logo_zip = temp.path().join("logo.zip");
        let logo_file = File::create(&logo_zip).unwrap();
        let mut zip = ZipWriter::new(logo_file);
        let options = SimpleFileOptions::default();
        zip.start_file("res/drawable-hdpi-v4/about_logo.png", options)
            .unwrap();
        zip.write_all(b"logo_data").unwrap();
        zip.finish().unwrap();

        let out = temp.path().join("output.zip");
        File::create(&out).unwrap();

        let result = run(&source_dir, &logo_zip, &out, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("overwrite"));
    }
}
