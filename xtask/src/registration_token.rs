//! Extract the exact comparison-build inputs and derive its registration token.

use std::path::Path;

use whatsapp_api::model::phone::PhoneNumber;
use whatsapp_api::registration::token_material::TokenMaterial;

pub fn run(apk: &Path, country_code: u16, national_number: &str) -> Result<(), String> {
    let phone =
        PhoneNumber::new(country_code, national_number).map_err(|error| error.to_string())?;

    let material = TokenMaterial::open(apk).map_err(|error| error.to_string())?;
    let token = material.derive(&phone).map_err(|error| error.to_string())?;

    println!("{}", token.as_str());
    eprintln!(
        "derived from {}, {}, and classes.dex",
        material.logo_entry, material.signature_entry
    );
    Ok(())
}
