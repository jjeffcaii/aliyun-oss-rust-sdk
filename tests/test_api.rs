#[macro_use]
extern crate log;

use yunoss::Client;

fn init() {
    pretty_env_logger::try_init_timed().ok();
}

#[tokio::test]
async fn test_get_object() {
    init();

    let get_env = |k: &str| -> String { std::env::var(k).unwrap_or_default() };

    // NOTICE: ensure you have environments below!!!
    let endpoint = get_env("OSS_ENDPOINT");
    let access_key_id = get_env("OSS_ACCESS_KEY_ID");
    let access_key_secret = get_env("OSS_ACCESS_KEY_SECRET");
    let bucket = get_env("OSS_BUCKET");
    let object = get_env("OSS_OBJECT");

    let cli = Client::builder()
        .endpoint(endpoint)
        .access_key(access_key_id)
        .access_secret(access_key_secret)
        .build()
        .unwrap();
    let bucket = cli.bucket(bucket).unwrap();
    let result = bucket.get_object(object).await;

    match result {
        Ok(b) => {
            let s = String::from_utf8(b).unwrap();
            info!("{}", s)
        }
        Err(e) => error!("{}", e),
    }
}
