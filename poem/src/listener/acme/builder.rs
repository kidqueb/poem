use std::{
    collections::HashSet,
    io::{Error as IoError, ErrorKind, Result as IoResult},
    path::PathBuf,
    sync::Arc,
};

use crate::listener::acme::{keypair::KeyPair, AutoCert, LETS_ENCRYPT_PRODUCTION};

/// ACME configuration builder
pub struct AutoCertBuilder {
    directory_url: String,
    domains: HashSet<String>,
    cache_path: Option<PathBuf>,
}

impl AutoCertBuilder {
    pub(crate) fn new() -> Self {
        Self {
            directory_url: LETS_ENCRYPT_PRODUCTION.to_string(),
            domains: HashSet::new(),
            cache_path: None,
        }
    }

    /// Sets the directory url.
    ///
    /// Defaults to [`LETS_ENCRYPT_PRODUCTION`]
    #[must_use]
    pub fn directory_url(self, directory_url: impl Into<String>) -> Self {
        Self {
            directory_url: directory_url.into(),
            ..self
        }
    }

    /// Adds a domain.
    #[must_use]
    pub fn domain(mut self, domain: impl Into<String>) -> Self {
        self.domains.insert(domain.into());
        self
    }

    /// Sets the cache path for caching certificates.
    ///
    /// This is not a necessary option. If you do not configure the cache path,
    /// the obtained certificate will be stored in memory and will need to be
    /// obtained again when the server is restarted next time.
    #[must_use]
    pub fn cache_path(self, path: impl Into<PathBuf>) -> Self {
        Self {
            cache_path: Some(path.into()),
            ..self
        }
    }

    /// Consumes this builder and returns a [`AutoCert`] object.
    pub fn build(self) -> IoResult<AutoCert> {
        let directory_url = self.directory_url.parse().map_err(|err| {
            IoError::new(ErrorKind::Other, format!("invalid directory url: {}", err))
        })?;
        if self.domains.is_empty() {
            return Err(IoError::new(
                ErrorKind::Other,
                "at least one domain name is expected",
            ));
        }

        let mut cache_key = None;
        let mut cache_cert = None;

        if let Some(cache_path) = &self.cache_path {
            let pkey_path = cache_path.join("key.pem");
            if pkey_path.exists() {
                tracing::debug!(path = %pkey_path.display(), "load private key from cache path");
                cache_key = Some(std::fs::read(cache_path)?);
            }

            let cert_path = cache_path.join("cert.pem");
            if cert_path.exists() {
                tracing::debug!(path = %cert_path.display(), "load certificate from cache path");
                cache_cert = Some(std::fs::read(cert_path)?);
            }
        }

        Ok(AutoCert {
            directory_url,
            domains: self.domains.into_iter().collect(),
            key_pair: Arc::new(KeyPair::generate()?),
            cache_path: self.cache_path,
            cache_key,
            cache_cert,
        })
    }
}