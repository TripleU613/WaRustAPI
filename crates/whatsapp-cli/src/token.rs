//! Token derivation and verification commands.

use std::path::PathBuf;

use clap::Subcommand;
use whatsapp_api::Error;
use whatsapp_api::model::phone::PhoneNumber;
use whatsapp_api::registration::request::LabRegistrationConfig;
use whatsapp_api::registration::token_material::TokenMaterial;

#[derive(Subcommand)]
pub enum TokenCmd {
    /// Derive the registration token from APK material.
    Derive {
        /// Country calling code.
        #[arg(long)]
        country_code: u16,
        /// National phone number digits.
        #[arg(long)]
        national_number: String,
        /// Path to APK or material archive.
        #[arg(long, env = "WHATSAPP_API_TOKEN_MATERIAL")]
        material: PathBuf,
    },
    /// Verify a config token against APK material.
    Verify {
        /// Path to the private JSON configuration.
        #[arg(long)]
        config: PathBuf,
        /// Path to APK or material archive.
        #[arg(long, env = "WHATSAPP_API_TOKEN_MATERIAL")]
        material: PathBuf,
    },
}

pub fn run(cmd: &TokenCmd) -> Result<(), Error> {
    match cmd {
        TokenCmd::Derive {
            country_code,
            national_number,
            material,
        } => {
            let phone = PhoneNumber::new(*country_code, national_number)?;
            let material = TokenMaterial::open(material)?;
            let token = material.derive(&phone)?;
            println!("{}", token.as_str());
            eprintln!(
                "derived from {}, {}, and classes.dex",
                material.logo_entry, material.signature_entry
            );
            Ok(())
        }
        TokenCmd::Verify { config, material } => {
            let config_data = std::fs::read_to_string(config)
                .map_err(|e| Error::invalid("token", format!("cannot read config: {e}")))?;
            let config: LabRegistrationConfig = serde_json::from_str(&config_data)
                .map_err(|e| Error::invalid("token", format!("invalid config JSON: {e}")))?;
            config.validate()?;

            let material = TokenMaterial::open(material)?;
            let derived_token = material.derive(&config.phone)?;

            if derived_token.as_str() == config.token.as_str() {
                println!("token matches material");
                Ok(())
            } else {
                println!("token does not match material");
                Err(Error::invalid("token", "token mismatch"))
            }
        }
    }
}
