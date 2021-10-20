use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt::Write;
use std::net::Ipv4Addr;
use std::sync::Arc;

use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::sha1::Sha1;
use crypto::sha2::Sha256;
use once_cell::sync::Lazy;
use reqwest::Url;

use crate::config::{AuthVersion, ClientConfig};
use crate::error::{OSSError, ServiceError};
use crate::types::{Credentials, Headers, Params, Request};
use crate::util;
use crate::Result;

static SIGN_KEYS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let v = vec![
        "acl",
        "uploads",
        "location",
        "cors",
        "logging",
        "website",
        "referer",
        "lifecycle",
        "delete",
        "append",
        "tagging",
        "objectMeta",
        "uploadId",
        "partNumber",
        "security-token",
        "position",
        "img",
        "style",
        "styleName",
        "replication",
        "replicationProgress",
        "replicationLocation",
        "cname",
        "bucketInfo",
        "comp",
        "qos",
        "live",
        "status",
        "vod",
        "startTime",
        "endTime",
        "symlink",
        "x-oss-process",
        "response-content-type",
        "x-oss-traffic-limit",
        "response-content-language",
        "response-expires",
        "response-cache-control",
        "response-content-disposition",
        "response-content-encoding",
        "udf",
        "udfName",
        "udfImage",
        "udfId",
        "udfImageDesc",
        "udfApplication",
        "comp",
        "udfApplicationLog",
        "restore",
        "callback",
        "callback-var",
        "qosInfo",
        "policy",
        "stat",
        "encryption",
        "versions",
        "versioning",
        "versionId",
        "requestPayment",
        "x-oss-request-payer",
        "sequential",
        "inventory",
        "inventoryId",
        "continuation-token",
        "asyncFetch",
        "worm",
        "wormId",
        "wormExtend",
        "withHashContext",
        "x-oss-enable-md5",
        "x-oss-enable-sha1",
        "x-oss-enable-sha256",
        "x-oss-hash-ctx",
        "x-oss-md5-ctx",
        "transferAcceleration",
        "regionList",
    ];
    v.into_iter().collect()
});

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

    pub(crate) async fn execute(
        &self,
        method: reqwest::Method,
        bucket: &str,
        object: &str,
        params: Option<Params>,
        headers: Option<Headers>,
        data: Vec<u8>,
        init_crc: u64,
    ) -> Result<Vec<u8>> {
        let url_params = match params {
            Some(ref it) => Some(Self::get_url_params(it)?),
            None => None,
        };

        let sub_resource = match params {
            Some(ref it) => Some(self.get_sub_resource(it)?),
            None => None,
        };

        let url = self
            .url_maker
            .get_url(bucket, object, &url_params.unwrap_or_default());

        let resource = self.get_resource(bucket, object, &sub_resource.unwrap_or_default())?;

        let req = Request {
            url,
            method,
            headers: headers.unwrap_or_default(),
            params: params.unwrap_or_default(),
            body: data,
        };

        self.do_request(req, resource, init_crc).await
    }

    async fn do_request(
        &self,
        mut req: Request,
        resource: String,
        init_crc: u64,
    ) -> Result<Vec<u8>> {
        self.handle_body(&mut req, init_crc);

        // TODO: http proxy

        // http time
        let date = util::httptime();
        req.headers.insert("date".into(), date);

        // user-agent
        req.headers
            .insert("user-agent".into(), self.config.ua.clone());

        // host
        if let Ok(it) = Url::parse(&req.url) {
            if let Some(host) = it.host_str() {
                req.headers.insert("host".into(), host.into());
            }
        }

        let token = self.config.security_token();
        if !token.is_empty() {
            req.headers
                .insert("x-oss-security-token".into(), token.into());
        }

        self.sign_header(&mut req, resource);

        let resp = req.send(&self.client).await?;

        if resp.status().is_success() {
            let b = resp.bytes().await?;
            let b = b.to_vec();
            Ok(b)
        } else {
            let status_code = resp.status().as_u16();
            let b = resp.bytes().await?;
            let b = b.to_vec();
            if let Ok(e) = ServiceError::try_from_xml(&b) {
                Err(OSSError::ServiceError(status_code, e.code, e.message, e.request_id).into())
            } else {
                bail!("{}", String::from_utf8_lossy(&b))
            }
        }
    }

    fn sign_header(&self, req: &mut Request, resource: String) {
        let additional_list = self.get_additional_header_keys(&req.headers);

        let signstr = self.get_signed_str(req, &resource, self.config.access_key_secret());

        let authorization_str = match self.config.auth_version {
            AuthVersion::V1 => {
                format!("OSS {}:{}", self.config.access_key_id(), signstr)
            }
            AuthVersion::V2 => {
                let additional_list = self.get_additional_header_keys(&req.headers);
                if additional_list.is_empty() {
                    format!(
                        "OSS2 AccessKeyId:{},Signature:{}",
                        self.config.access_key_id(),
                        signstr
                    )
                } else {
                    let mut additionnal_headers_str = String::new();
                    for (i, v) in additional_list.iter().enumerate() {
                        if i != 0 {
                            additionnal_headers_str.push(';')
                        }
                        additionnal_headers_str.push_str(v);
                    }
                    format!(
                        "OSS2 AccessKeyId:{},AdditionalHeaders:{},Signature:{}",
                        self.config.access_key_id(),
                        additionnal_headers_str,
                        signstr
                    )
                }
            }
        };
        req.headers
            .insert("authorization".into(), authorization_str);
    }

    fn get_additional_header_keys(&self, header: &Headers) -> BTreeSet<String> {
        let mut keys = BTreeSet::new();

        for it in &self.config.additional_headers {
            if header.contains_key(it) {
                keys.insert(it.to_lowercase());
            }
        }
        keys
    }

    fn get_signed_str(&self, req: &Request, resource: &str, key_secret: &str) -> String {
        let mut hs = BTreeMap::new();
        let additional_keys = self.get_additional_header_keys(&req.headers);

        for (k, v) in &req.headers {
            let k = k.to_lowercase();
            if k.starts_with("x-oss-") {
                hs.insert(k, v);
            } else if self.config.auth_version == AuthVersion::V2 {
                if additional_keys.contains(&k) {
                    hs.insert(k, v);
                }
            }
        }

        let mut canonicalized_oss_headers = String::new();

        for (k, v) in &hs {
            canonicalized_oss_headers.write_str(k).ok();
            canonicalized_oss_headers.write_char(':').ok();
            canonicalized_oss_headers.write_str(v).ok();
            canonicalized_oss_headers.write_char('\n').ok();
        }

        let date = req.headers.get("date");
        let content_type = req.headers.get("content-type");
        let content_md5 = req.headers.get("content-md5");

        let mut sign_str = String::new();

        sign_str.push_str(req.method.as_str());
        sign_str.push('\n');
        if let Some(it) = content_md5 {
            sign_str.push_str(it);
        }
        sign_str.push('\n');
        if let Some(it) = content_type {
            sign_str.push_str(it);
        }
        sign_str.push('\n');
        if let Some(it) = date {
            sign_str.push_str(it);
        }
        sign_str.push('\n');
        sign_str.push_str(&canonicalized_oss_headers);

        let sign = match self.config.auth_version {
            AuthVersion::V1 => {
                sign_str.push_str(resource);
                let mut mac = Hmac::new(Sha1::new(), key_secret.as_bytes());
                mac.input(sign_str.as_bytes());
                let res = mac.result();
                let code = res.code();
                base64::encode(code)
            }
            AuthVersion::V2 => {
                for (i, v) in additional_keys.iter().enumerate() {
                    if i != 0 {
                        sign_str.push(';');
                    }
                    sign_str.push_str(v);
                }
                sign_str.push('\n');
                sign_str.push_str(resource);

                let mut mac = Hmac::new(Sha256::new(), key_secret.as_bytes());
                mac.input(sign_str.as_bytes());
                let res = mac.result();
                let code = res.code();
                base64::encode(code)
            }
        };

        info!("sign: before={}, after={}", sign_str, sign);

        sign
    }

    fn handle_body(&self, req: &mut Request, init_crc: u64) {
        let length = req.body.len();
        req.headers
            .insert("content-length".into(), length.to_string());

        if !req.body.is_empty() && self.config.enable_md5 {
            // TODO: md5 threshold
            let md5sum = format!("{:x}", md5::compute(&req.body));
            req.headers.insert("content-md5".into(), md5sum);
        }

        if !req.body.is_empty() && self.config.enable_crc {
            // TODO: crc
        }
    }

    fn get_resource(&self, bucket: &str, object: &str, sub_resource: &str) -> Result<String> {
        let sub_resource = if sub_resource.is_empty() {
            Cow::from(sub_resource)
        } else {
            Cow::from(format!("?{}", sub_resource))
        };

        if bucket.is_empty() {
            match self.config.auth_version {
                AuthVersion::V1 => Ok(format!("/{}{}", bucket, sub_resource)),
                AuthVersion::V2 => {
                    // %2F ==> '/'
                    Ok(format!("%2F{}", sub_resource))
                }
            }
        } else {
            match self.config.auth_version {
                AuthVersion::V1 => Ok(format!("/{}/{}{}", bucket, object, sub_resource)),
                AuthVersion::V2 => {
                    let mut sb = String::new();
                    sb.write_str(&serde_urlencoded::to_string(format!("/{}/", bucket))?)
                        .ok();
                    sb.write_str(&util::query_escape(object)).ok();
                    sb.write_str(&sub_resource).ok();
                    Ok(sb)
                }
            }
        }
    }

    fn get_url_params(params: &Params) -> Result<String> {
        let s = serde_urlencoded::to_string(params)?;
        Ok(s.replace("+", "%20"))
    }

    fn get_sub_resource(&self, params: &Params) -> Result<String> {
        let mut sign_params = BTreeMap::<String, Option<String>>::new();
        for (k, v) in params {
            if self.config.auth_version == AuthVersion::V2 {
                let k = serde_urlencoded::to_string(k)?;
                let v = v
                    .as_ref()
                    .filter(|it| !it.is_empty())
                    .map(|it| util::query_escape(it));
                sign_params.insert(k, v);
            } else if Self::is_param_sign(k) {
                sign_params.insert(k.into(), v.clone());
            }
        }

        let mut buf = String::new();

        for (i, (k, v)) in sign_params.iter().enumerate() {
            if i != 0 {
                buf.write_char('&').ok();
            }
            buf.write_str(k).ok();

            if let Some(it) = v {
                buf.write_char('=').ok();
                buf.write_str(it).ok();
            }
        }

        Ok(buf)
    }

    #[inline]
    fn is_param_sign(param_key: &str) -> bool {
        SIGN_KEYS.contains(param_key)
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
        let url = match Url::parse(endpoint) {
            Ok(u) => u,
            Err(_) => Url::parse(&format!("http://{}", endpoint))?,
        };
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

    pub(crate) fn get_url(&self, bucket: &str, object: &str, params: &str) -> String {
        let (host, path) = self.build_url(bucket, object);
        if params.is_empty() {
            format!("{}://{}{}", self.schema, host, path)
        } else {
            format!("{}://{}{}?{}", self.schema, host, path, params)
        }
    }

    // build to (host,path)
    fn build_url(&self, bucket: &str, object: &str) -> (Cow<str>, Cow<str>) {
        let object = util::query_escape(object);
        match self.typ {
            UrlType::CNAME => {
                let host = Cow::from(&self.net_loc[..]);
                let path = Cow::from(format!("/{}", object));
                (host, path)
            }
            UrlType::IP => {
                let host = Cow::from(&self.net_loc[..]);
                let path = if bucket.is_empty() {
                    Cow::from("/")
                } else {
                    Cow::from(format!("/{}/{}", bucket, object))
                };
                (host, path)
            }
            UrlType::ALIYUN => {
                if bucket.is_empty() {
                    let host = Cow::from(&self.net_loc[..]);
                    let path = Cow::from("/");
                    (host, path)
                } else {
                    let host = Cow::from(format!("{}.{}", bucket, self.net_loc));
                    let path = Cow::from(format!("/{}", object));
                    (host, path)
                }
            }
        }
    }
}
