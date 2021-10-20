use anyhow::Result;
use yunoss::Client;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Client::builder()
        .access_key("your_access_key")
        .access_secret("your_access_secret")
        .endpoint("your.endpoint.com")
        .build()?;

    let b = cli
        .bucket("your_bucket")?
        .get_object("your_object.txt")
        .await?;

    println!("bingo: {}", String::from_utf8(b).unwrap());

    Ok(())
}
