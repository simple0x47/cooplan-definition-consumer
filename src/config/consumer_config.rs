use crate::error::{Error, ErrorKind};

const DEFINITION_PROVIDER_URI_ENV: &str = "DEFINITION_PROVIDER_URI";

pub struct ConsumerConfig {
    connection_uri: String,
}

impl ConsumerConfig {
    pub fn connection_uri(&self) -> &str {
        self.connection_uri.as_str()
    }
}

pub async fn try_generate() -> Result<ConsumerConfig, Error> {
    let connection_uri = match std::env::var(DEFINITION_PROVIDER_URI_ENV) {
        Ok(connection_uri) => connection_uri,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::AutoConfigFailure,
                format!("failed to find definition provider uri: {}", error),
            ))
        }
    };

    Ok(ConsumerConfig { connection_uri })
}
