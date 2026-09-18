//! Extract the exact comparison-build inputs and derive its registration token.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use whatsapp_api::model::phone::PhoneNumber;
use whatsapp_api::registration::token::{
    ComparisonTokenMaterial, derive_comparison_registration_token,
};

const LOGO_CANDIDATES: [&str; 3] = [
    "res/drawable-hdpi/about_logo.png",
    "res/drawable-hdpi-v4/about_logo.png",
    "res/drawable-xxhdpi-v4/about_logo.png",
];

fn read_entry(
    archive: &mut zip::ZipArchive<File>,
    name: &str,
    maximum: usize,
) -> Result<Vec<u8>, String> {
    let entry = archive
        .by_name(name)
        .map_err(|error| format!("{name}: {error}"))?;
    let declared = usize::try_from(entry.size()).map_err(|_| format!("{name}: size overflow"))?;
    if declared == 0 || declared > maximum {
        return Err(format!(
            "{name}: expected 1..={maximum} bytes, got {declared}"
        ));
    }
    let mut bytes = Vec::with_capacity(declared);
    entry
        .take(maximum as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("{name}: {error}"))?;
    if bytes.len() > maximum {
        return Err(format!("{name}: exceeds {maximum} bytes"));
    }
    Ok(bytes)
}

pub fn run(apk: &Path, country_code: u16, national_number: &str) -> Result<(), String> {
    let phone =
        PhoneNumber::new(country_code, national_number).map_err(|error| error.to_string())?;
    let file = File::open(apk).map_err(|error| format!("{}: {error}", apk.display()))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|error| format!("zip: {error}"))?;

    let manifest = read_entry(&mut archive, "AndroidManifest.xml", 8 << 20)?;
    let package_name = crate::axml::parse_manifest(&manifest)?
        .package
        .ok_or("AndroidManifest.xml has no package name")?;
    let logo_name = LOGO_CANDIDATES
        .iter()
        .find(|name| archive.index_for_name(name).is_some())
        .ok_or("APK has none of the token helper's about_logo.png candidates")?;
    let logo = read_entry(
        &mut archive,
        logo_name,
        whatsapp_api::registration::token::MAX_TOKEN_LOGO_BYTES,
    )?;
    let classes_dex = read_entry(
        &mut archive,
        "classes.dex",
        whatsapp_api::registration::token::MAX_TOKEN_DEX_BYTES,
    )?;

    let signature_name = archive
        .file_names()
        .find(|name| {
            name.starts_with("META-INF/")
                && (name.ends_with(".DSA") || name.ends_with(".RSA") || name.ends_with(".EC"))
        })
        .map(str::to_owned)
        .ok_or("APK has no v1 signature block from which to extract a certificate")?;
    let signature = read_entry(&mut archive, &signature_name, 8 << 20)?;
    let certificate = crate::manifest::find_certificate(&signature, 0, signature.len(), 0)
        .ok_or("v1 signature block has no recognizable X.509 certificate")?;

    let token = derive_comparison_registration_token(
        &phone,
        ComparisonTokenMaterial {
            package_name: &package_name,
            about_logo: &logo,
            signing_certificate_der: certificate,
            classes_dex: &classes_dex,
        },
    )
    .map_err(|error| error.to_string())?;
    println!("{}", token.as_str());
    eprintln!("derived from {logo_name}, {signature_name}, and classes.dex");
    Ok(())
}
