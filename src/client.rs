use std::sync::OnceLock;

use serde::de::DeserializeOwned;

use crate::EndpointType;

static SERVER_URL: OnceLock<String> = OnceLock::new();

/// Sets the URL of the server to send requests to.
///
/// # Panics
///
/// Panics if the URL has already been set.
pub fn init(url: impl Into<String>) {
    SERVER_URL.set(url.into()).expect("Server URL already set");
}

pub async fn request<T: DeserializeOwned>(
    endpoint: &'static str,
    body: Vec<u8>,
    endpoint_type: EndpointType,
) -> Result<T, ()> {
    let server = match SERVER_URL.get() {
        Some(url) => url.as_str(),
        None => "/",
    };
    let url = format!("{server}/{endpoint}");
    let client = reqwest::Client::new();
    let builder = match endpoint_type {
        EndpointType::Query => client.get(url),
        EndpointType::Mutation => client.post(url),
    };
    let resp = builder
        .header("Content-Type", "application/bincode")
        .body(body)
        .send()
        .await
        .unwrap();
    let bytes = resp.bytes().await.unwrap();
    let bincode = bincode::deserialize::<T>(&bytes).unwrap();
    Ok(bincode)
}
