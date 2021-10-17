use std::net::Ipv4Addr;
use std::sync::Arc;
use std::time::Duration;

use reqwest::Url;

use crate::config::ClientConfig;
use crate::conn::{Conn, UrlMaker};
use crate::request;
use crate::{bucket::Bucket, Result};

#[derive(Clone)]
pub struct Client {
    config: Arc<ClientConfig>,
    conn: Conn,
    client: reqwest::Client,
}

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder {
            config: Default::default(),
        }
    }

    fn new(config: ClientConfig) -> Result<Client> {
        let client = reqwest::Client::builder().build()?;
        /*
        reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(3))
                .timeout(Duration::from_secs(3))
                .build()
                .unwrap() */

        let um = UrlMaker::new(&config.endpoint, config.cname, config.http_proxy.is_some())?;
        let config = Arc::new(config);
        let conn = Conn::new(config.clone(), Arc::new(um), client.clone());

        Ok(Client {
            conn,
            config,
            client,
        })
    }

    pub fn bucket(&self, bucket: impl Into<String>) -> Result<Bucket> {
        // TODO: validate bucket name
        Ok(Bucket::new(self.clone(), bucket.into()))
    }
}

pub struct ClientBuilder {
    config: ClientConfig,
}

impl ClientBuilder {
    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.config.endpoint = endpoint.into();
        self
    }

    pub fn access_key(mut self, key: impl Into<String>) -> Self {
        self.config.access_key_id = key.into();
        self
    }

    pub fn access_secret(mut self, secret: impl Into<String>) -> Self {
        self.config.access_key_secret = secret.into();
        self
    }

    pub fn build(self) -> Result<Client> {
        Client::new(self.config)
    }
}
