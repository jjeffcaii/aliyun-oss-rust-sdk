use crate::client::Client;
use crate::Result;

#[derive(Clone)]
pub struct Bucket {
    client: Client,
    name: String,
}

impl Bucket {
    pub(crate) fn new(client: Client, name: String) -> Bucket {
        Bucket { client, name }
    }

    pub async fn get_object(&self, object_key: impl AsRef<str>) -> Result<Vec<u8>> {
        todo!("todo: get object")
    }

    async fn do_request(&self, method: reqwest::Method, object_name: &str) {
        // self.client
    }
}
