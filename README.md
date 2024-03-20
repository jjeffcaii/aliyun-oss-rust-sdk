# Aliyun OSS Rust SDK

An unofficial Alibaba Cloud OSS SDK for Rust.

**Still in working progress!**

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

    println!("bingo: {}", String::from_utf8(b).unwrap());

    Ok(())
}
```
