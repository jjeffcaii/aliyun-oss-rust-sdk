use std::convert::TryFrom;
use std::net::Ipv4Addr;
use std::sync::Arc;

use reqwest::Url;

use crate::config::ClientConfig;
use crate::Result;

#[derive(Debug, Clone)]
pub(crate) struct Conn {
    config: Arc<ClientConfig>,
    url_maker: Arc<UrlMaker>,
    client: reqwest::Client,
}

impl Conn {
    pub(crate) fn new(
        config: Arc<ClientConfig>,
        url_maker: Arc<UrlMaker>,
        client: reqwest::Client,
    ) -> Conn {
        Conn {
            config,
            url_maker,
            client,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum UrlType {
    CNAME,
    IP,
    ALIYUN,
}

#[derive(Debug, Clone)]
pub(crate) struct UrlMaker {
    schema: String,
    net_loc: String,
    typ: UrlType,
    is_proxy: bool,
}

impl UrlMaker {
    pub(crate) fn new(endpoint: &str, is_cname: bool, is_proxy: bool) -> Result<UrlMaker> {
        let url = Url::parse(endpoint)?;
        let schema = url.scheme();

        match schema {
            "http" | "https" => match url.host_str() {
                Some(host) => {
                    let typ = match host.parse::<Ipv4Addr>() {
                        Ok(add) => UrlType::IP,
                        _ => {
                            if is_cname {
                                UrlType::CNAME
                            } else {
                                UrlType::ALIYUN
                            }
                        }
                    };

                    Ok(UrlMaker {
                        is_proxy,
                        schema: schema.into(),
                        net_loc: host.into(),
                        typ,
                    })
                }
                None => bail!("cannot extract host info from endpoint '{}'!", endpoint),
            },
            _ => bail!("invalid schema {}: should be http or https only!", schema),
        }
    }
}
