//! WebTransport client utilities for Unity-Network FFI bridge
//!
//! Provides certificate configuration helpers and client setup utilities.
//! Focuses on solving certificate validation issues in development.

use wtransport::{ClientConfig, Endpoint};

/// WebTransport client configuration options
#[derive(Debug, Clone)]
pub struct ClientConfigOptions {
    /// Skip certificate validation (for development with self-signed certificates)
    pub no_cert_validation: bool,
}

impl Default for ClientConfigOptions {
    fn default() -> Self {
        Self {
            no_cert_validation: true, // Default to true for development
        }
    }
}

/// Build a WebTransport client configuration
///
/// # Arguments
/// * `options` - Configuration options
///
/// # Returns
/// ClientConfig ready for use with Endpoint::client()
pub fn build_client_config(options: ClientConfigOptions) -> ClientConfig {
    if options.no_cert_validation {
        // For development/POC, bypass certificate validation
        // This solves "UnknownIssuer" errors with self-signed certificates
        ClientConfig::builder()
            .with_bind_default()
            .with_no_cert_validation()
            .build()
    } else {
        // Default configuration with certificate validation (production)
        ClientConfig::default()
    }
}

/// Create a client endpoint with custom configuration
///
/// # Arguments
/// * `no_cert_validation` - Whether to skip certificate validation
///
/// # Returns
/// Result containing the endpoint or an error
pub fn create_client_endpoint(
    no_cert_validation: bool,
) -> Result<Endpoint<wtransport::endpoint::endpoint_side::Client>, String> {
    let options = ClientConfigOptions { no_cert_validation };

    let config = build_client_config(options);

    Endpoint::client(config).map_err(|e| format!("Failed to create client endpoint: {:?}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = ClientConfigOptions::default();
        assert!(config.no_cert_validation);
    }

    #[test]
    fn test_build_client_config_no_validation() {
        let options = ClientConfigOptions {
            no_cert_validation: true,
        };
        let config = build_client_config(options);
        // Should not panic - just ensures it builds
        drop(config);
    }

    #[test]
    fn test_build_client_config_with_validation() {
        let options = ClientConfigOptions {
            no_cert_validation: false,
        };
        let config = build_client_config(options);
        // Should not panic - just ensures it builds
        drop(config);
    }

    #[tokio::test]
    async fn test_create_endpoint_no_validation() {
        let result = create_client_endpoint(true);
        assert!(result.is_ok());
        let endpoint = result.unwrap();
        drop(endpoint);
    }
}
