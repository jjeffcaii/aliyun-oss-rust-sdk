#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(clippy::type_complexity)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::upper_case_acronyms)]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]

#[macro_use]
extern crate log;
#[macro_use]
extern crate anyhow;

pub type Result<T> = anyhow::Result<T>;

mod bucket;
mod client;
mod config;
mod conn;
mod error;
mod prelude;
mod request;

#[cfg(test)]
mod test_oss {
    use crate::prelude::*;

    #[tokio::test]
    async fn test_api_style() {
        let cli = Client::builder()
            .endpoint("http://fake.endpoint.com")
            .access_key("fake_access_key")
            .access_secret("fake_access_secret")
            .build()
            .unwrap();
        let bucket = cli.bucket("fake_bucket").unwrap();
        let data = bucket.get_object("foobar.txt").await.unwrap();

        assert!(!data.is_empty());
    }
}
