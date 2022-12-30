use std::sync::Arc;

use lapin::{
    tcp::{Identity, OwnedIdentity, OwnedTLSConfig},
    ConnectionProperties,
};

pub struct AmqpConnectConfig {
    uri: String,
    options: ConnectionProperties,
    owned_tls_config: OwnedTLSConfig,
}

impl AmqpConnectConfig {
    pub fn new(
        uri: String,
        options: ConnectionProperties,
        owned_tls_config: OwnedTLSConfig,
    ) -> AmqpConnectConfig {
        AmqpConnectConfig {
            uri,
            options,
            owned_tls_config,
        }
    }

    pub fn uri(&self) -> &str {
        self.uri.as_str()
    }

    pub fn cloned_options(&self) -> ConnectionProperties {
        self.options.clone()
    }

    pub fn cloned_owned_tls_config(&self) -> OwnedTLSConfig {
        let identity = match &self.owned_tls_config.identity {
            Some(identity) => {
                let der = identity.der.clone();
                let password = identity.password.clone();

                Some(OwnedIdentity { der, password })
            }
            None => None,
        };

        let cert_chain = self.owned_tls_config.cert_chain.clone();

        OwnedTLSConfig {
            identity,
            cert_chain,
        }
    }
}
