# Aliyun OSS Rust SDK

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/jjeffcaii/aliyun-oss-rust-sdk/rust.yml)
[![Codecov](https://img.shields.io/codecov/c/github/jjeffcaii/aliyun-oss-rust-sdk)](https://app.codecov.io/gh/jjeffcaii/aliyun-oss-rust-sdk)
[![Crates.io Version](https://img.shields.io/crates/v/yunoss)](https://crates.io/crates/yunoss)
[![Crates.io Total Downloads](https://img.shields.io/crates/d/yunoss)](https://crates.io/crates/yunoss)
![GitHub Tag](https://img.shields.io/github/v/tag/jjeffcaii/aliyun-oss-rust-sdk)
![GitHub License](https://img.shields.io/github/license/jjeffcaii/aliyun-oss-rust-sdk)

An unofficial Alibaba Cloud OSS SDK for Rust.

**Still in work progress!**

## Example

```rust
use yunoss::Client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Client::builder()
        .endpoint("your.endpoint.com")
        .access_key("your_access_key")
        .access_secret("your_access_secret")
        .build()?;

    let b = cli
        .bucket("your_bucket")?
        .get_object("your_object.txt")
        .await?;

    println!("bingo: {}", String::from_utf8_lossy(b));

    Ok(())
}
```
