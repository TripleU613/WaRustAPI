//! Version-scoped connection metadata observed in WhatsApp 2.26.37.70.
//!
//! These values describe a separately pinned comparison APK. They are useful
//! for building and testing the connector boundary, but are not installed in
//! the active 2.26.36.75 Android profile.

use std::fmt;

/// Protocol version selected by the comparison client's connection engine.
pub const COMPARISON_PROTOCOL_VERSION: u8 = 6;
/// Token-dictionary version written into the comparison connection header.
pub const COMPARISON_DICTIONARY_VERSION: u8 = 3;
/// Header written before the first Noise handshake frame.
pub const COMPARISON_CONNECTION_HEADER: [u8; 4] = [
    b'W',
    b'A',
    COMPARISON_PROTOCOL_VERSION,
    COMPARISON_DICTIONARY_VERSION,
];
/// Built-in ports selected by the comparison connection sequence.
pub const COMPARISON_PRIMARY_PORTS: [u16; 2] = [443, 5222];
/// Last-resort port used by dedicated fallback states.
pub const COMPARISON_FALLBACK_PORT: u16 = 80;

/// Built-in gateway families in the comparison client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComparisonGateway {
    /// `g.whatsapp.net`.
    Primary,
    /// `g-fallback.whatsapp.net`.
    Fallback,
    /// One of the sixteen numbered edge hosts.
    Edge(u8),
}

impl ComparisonGateway {
    /// Return the DNS hostname, rejecting an edge index outside 1..=16.
    pub fn hostname(self) -> Result<String, ComparisonEndpointError> {
        match self {
            Self::Primary => Ok("g.whatsapp.net".to_owned()),
            Self::Fallback => Ok("g-fallback.whatsapp.net".to_owned()),
            Self::Edge(1..=16) => {
                let Self::Edge(index) = self else {
                    unreachable!()
                };
                Ok(format!("e{index}.whatsapp.net"))
            }
            Self::Edge(index) => Err(ComparisonEndpointError::InvalidEdge(index)),
        }
    }
}

/// Socket layer selected before the Noise stream is started.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComparisonSocketLayer {
    /// A direct Java TCP socket.
    RawTcp,
    /// A Java TLS socket created through `SSLSocketFactory`.
    Tls,
}

/// How the built-in connection-sequence branch treats the endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComparisonEndpointMode {
    /// Normal 443/5222 gateway or edge branch.
    Direct,
    /// Dedicated last-resort port-80 branch.
    Port80Fallback,
}

/// Deterministic socket metadata for a built-in comparison-client branch.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComparisonEndpoint {
    /// Gateway family.
    pub gateway: ComparisonGateway,
    /// Socket port.
    pub port: u16,
    /// Layer created before Noise.
    pub layer: ComparisonSocketLayer,
    /// Connection-sequence branch mode.
    pub mode: ComparisonEndpointMode,
}

impl ComparisonEndpoint {
    /// Resolve the built-in layer/mode mapping observed in the comparison APK.
    ///
    /// Primary and fallback gateway branches remain raw TCP on port 443. Edge
    /// branches wrap port 443 with TLS. All built-in port-5222 and port-80
    /// branches use raw TCP; port 80 is marked as a distinct fallback mode.
    pub fn built_in(
        gateway: ComparisonGateway,
        port: u16,
    ) -> Result<Self, ComparisonEndpointError> {
        // Validate the edge index even though the endpoint stores the enum.
        let _ = gateway.hostname()?;

        let (layer, mode) = match port {
            443 => (
                if matches!(gateway, ComparisonGateway::Edge(_)) {
                    ComparisonSocketLayer::Tls
                } else {
                    ComparisonSocketLayer::RawTcp
                },
                ComparisonEndpointMode::Direct,
            ),
            5222 => (
                ComparisonSocketLayer::RawTcp,
                ComparisonEndpointMode::Direct,
            ),
            80 => (
                ComparisonSocketLayer::RawTcp,
                ComparisonEndpointMode::Port80Fallback,
            ),
            unsupported => return Err(ComparisonEndpointError::UnsupportedPort(unsupported)),
        };

        Ok(Self {
            gateway,
            port,
            layer,
            mode,
        })
    }

    /// DNS hostname for this endpoint.
    pub fn hostname(&self) -> String {
        // Construction validates the gateway, so this cannot fail.
        self.gateway
            .hostname()
            .expect("validated comparison gateway")
    }
}

/// Invalid comparison endpoint metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonEndpointError {
    /// The comparison APK has numbered edge hosts only in the range 1..=16.
    InvalidEdge(u8),
    /// The port is not one of the statically observed built-in ports.
    UnsupportedPort(u16),
}

impl fmt::Display for ComparisonEndpointError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEdge(index) => {
                write!(f, "comparison edge index {index} is outside 1..=16")
            }
            Self::UnsupportedPort(port) => {
                write!(f, "comparison built-in port {port} is unsupported")
            }
        }
    }
}

impl std::error::Error for ComparisonEndpointError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comparison_header_is_exact() {
        assert_eq!(COMPARISON_CONNECTION_HEADER, *b"WA\x06\x03");
    }

    #[test]
    fn built_in_gateway_443_is_raw_but_edge_443_is_tls() {
        let primary = ComparisonEndpoint::built_in(ComparisonGateway::Primary, 443).unwrap();
        assert_eq!(primary.hostname(), "g.whatsapp.net");
        assert_eq!(primary.layer, ComparisonSocketLayer::RawTcp);

        let fallback = ComparisonEndpoint::built_in(ComparisonGateway::Fallback, 443).unwrap();
        assert_eq!(fallback.hostname(), "g-fallback.whatsapp.net");
        assert_eq!(fallback.layer, ComparisonSocketLayer::RawTcp);

        let edge = ComparisonEndpoint::built_in(ComparisonGateway::Edge(16), 443).unwrap();
        assert_eq!(edge.hostname(), "e16.whatsapp.net");
        assert_eq!(edge.layer, ComparisonSocketLayer::Tls);
    }

    #[test]
    fn ports_5222_and_80_are_raw_with_distinct_modes() {
        let direct = ComparisonEndpoint::built_in(ComparisonGateway::Edge(1), 5222).unwrap();
        assert_eq!(direct.layer, ComparisonSocketLayer::RawTcp);
        assert_eq!(direct.mode, ComparisonEndpointMode::Direct);

        let fallback = ComparisonEndpoint::built_in(ComparisonGateway::Edge(1), 80).unwrap();
        assert_eq!(fallback.layer, ComparisonSocketLayer::RawTcp);
        assert_eq!(fallback.mode, ComparisonEndpointMode::Port80Fallback);
    }

    #[test]
    fn invalid_edges_and_unobserved_ports_are_rejected() {
        assert_eq!(
            ComparisonEndpoint::built_in(ComparisonGateway::Edge(0), 443),
            Err(ComparisonEndpointError::InvalidEdge(0))
        );
        assert_eq!(
            ComparisonEndpoint::built_in(ComparisonGateway::Primary, 8443),
            Err(ComparisonEndpointError::UnsupportedPort(8443))
        );
    }
}
