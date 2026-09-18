//! Version-scoped Noise pattern selector observed in WhatsApp 2.26.37.70.
//!
//! The comparison APK reads remotely overridable configuration. Its compiled
//! defaults are [`ComparisonPqMode::None`] and [`ComparisonPqVariant::IkKem`],
//! but those defaults are not proof of the effective server-provided values.
//! This selector is therefore descriptive and is not installed into the
//! active Android profile.

use serde::{Deserialize, Serialize};

/// Post-quantum handshake policy understood by the comparison client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonPqMode {
    /// Select the classic 25519-only patterns.
    None,
    /// Enable the configured ML-KEM512 hybrid patterns.
    Enable,
    /// Require the configured ML-KEM512 hybrid patterns.
    Enforce,
}

/// Hybrid protocol variant selected when PQ mode is enabled or enforced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonPqVariant {
    /// XXkem for full handshakes and IKkem for resumption.
    IkKem,
    /// Forward-secret XXkem/IKkem variant.
    IkKemFs,
    /// Ephemeral XXkem with IKkem2 resumption.
    XxKemEphIkKem2,
    /// Ephemeral XXkem full handshake with IKkem resumption.
    XxKemEphOnly,
}

/// Connection state that determines the full, resumed, or fallback pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonHandshakeKind {
    /// New full handshake.
    Full,
    /// Resume using a known remote static key.
    Resume,
    /// Full fallback after a failed resumed handshake.
    ResumeFallback,
}

/// Compiled PQ-mode default in WhatsApp 2.26.37.70 before remote overrides.
pub const COMPARISON_DEFAULT_PQ_MODE: ComparisonPqMode = ComparisonPqMode::None;
/// Compiled PQ-variant default in WhatsApp 2.26.37.70 before remote overrides.
pub const COMPARISON_DEFAULT_PQ_VARIANT: ComparisonPqVariant = ComparisonPqVariant::IkKem;

/// Return the exact pattern string selected by the comparison client's map.
pub const fn comparison_noise_pattern(
    mode: ComparisonPqMode,
    variant: ComparisonPqVariant,
    kind: ComparisonHandshakeKind,
) -> &'static str {
    use ComparisonHandshakeKind as Kind;
    use ComparisonPqMode as Mode;
    use ComparisonPqVariant as Variant;

    match (mode, variant, kind) {
        (Mode::None, _, Kind::Full) => "Noise_XX_25519_AESGCM_SHA256",
        (Mode::None, _, Kind::Resume) => "Noise_IK_25519_AESGCM_SHA256",
        (Mode::None, _, Kind::ResumeFallback) => "Noise_XXfallback_25519_AESGCM_SHA256",
        (Mode::Enable | Mode::Enforce, Variant::IkKem, Kind::Full) => {
            "Noise_XXkem_25519_MLKEM512_AESGCM_SHA256"
        }
        (Mode::Enable | Mode::Enforce, Variant::IkKem, Kind::Resume) => {
            "Noise_IKkem+X25519+MLKEM512+AESGCM256+SHA256"
        }
        (Mode::Enable | Mode::Enforce, Variant::IkKem, Kind::ResumeFallback) => {
            "Noise_XXkemfallback_25519_MLKEM512_AESGCM_SHA256"
        }
        (Mode::Enable | Mode::Enforce, Variant::IkKemFs, Kind::Full) => {
            "Noise_XXkem-FS_25519_MLKEM512_AESGCM_SHA256"
        }
        (Mode::Enable | Mode::Enforce, Variant::IkKemFs, Kind::Resume) => {
            "Noise_IKkem-FS+X25519+MLKEM512+AESGCM256+SHA256"
        }
        (Mode::Enable | Mode::Enforce, Variant::IkKemFs, Kind::ResumeFallback) => {
            "Noise_XXkem-FSfallback_25519_MLKEM512_AESGCM_SHA256"
        }
        (Mode::Enable | Mode::Enforce, Variant::XxKemEphIkKem2, Kind::Full)
        | (Mode::Enable | Mode::Enforce, Variant::XxKemEphOnly, Kind::Full) => {
            "Noise_XXkemEph_25519_MLKEM512_AESGCM_SHA256"
        }
        (Mode::Enable | Mode::Enforce, Variant::XxKemEphIkKem2, Kind::Resume) => {
            "Noise_IKkem2+X25519+MLKEM512+AESGCM256+SHA256"
        }
        (Mode::Enable | Mode::Enforce, Variant::XxKemEphIkKem2, Kind::ResumeFallback) => {
            "Noise_XXkemEphfallback_25519_MLKEM512_AESGCM_SHA256"
        }
        (Mode::Enable | Mode::Enforce, Variant::XxKemEphOnly, Kind::Resume) => {
            "Noise_IKkem+X25519+MLKEM512+AESGCM256+SHA256"
        }
        (Mode::Enable | Mode::Enforce, Variant::XxKemEphOnly, Kind::ResumeFallback) => {
            "Noise_XXkemfallback_25519_MLKEM512_AESGCM_SHA256"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiled_defaults_select_classic_patterns() {
        assert_eq!(
            comparison_noise_pattern(
                COMPARISON_DEFAULT_PQ_MODE,
                COMPARISON_DEFAULT_PQ_VARIANT,
                ComparisonHandshakeKind::Full,
            ),
            "Noise_XX_25519_AESGCM_SHA256"
        );
        assert_eq!(
            comparison_noise_pattern(
                COMPARISON_DEFAULT_PQ_MODE,
                COMPARISON_DEFAULT_PQ_VARIANT,
                ComparisonHandshakeKind::Resume,
            ),
            "Noise_IK_25519_AESGCM_SHA256"
        );
    }

    #[test]
    fn enable_and_enforce_use_the_same_variant_map() {
        for variant in [
            ComparisonPqVariant::IkKem,
            ComparisonPqVariant::IkKemFs,
            ComparisonPqVariant::XxKemEphIkKem2,
            ComparisonPqVariant::XxKemEphOnly,
        ] {
            for kind in [
                ComparisonHandshakeKind::Full,
                ComparisonHandshakeKind::Resume,
                ComparisonHandshakeKind::ResumeFallback,
            ] {
                assert_eq!(
                    comparison_noise_pattern(ComparisonPqMode::Enable, variant, kind),
                    comparison_noise_pattern(ComparisonPqMode::Enforce, variant, kind),
                );
            }
        }
    }

    #[test]
    fn every_hybrid_selection_names_mlkem512() {
        for variant in [
            ComparisonPqVariant::IkKem,
            ComparisonPqVariant::IkKemFs,
            ComparisonPqVariant::XxKemEphIkKem2,
            ComparisonPqVariant::XxKemEphOnly,
        ] {
            for kind in [
                ComparisonHandshakeKind::Full,
                ComparisonHandshakeKind::Resume,
                ComparisonHandshakeKind::ResumeFallback,
            ] {
                assert!(
                    comparison_noise_pattern(ComparisonPqMode::Enable, variant, kind)
                        .contains("MLKEM512")
                );
            }
        }
    }
}
