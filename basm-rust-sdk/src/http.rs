use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

use crate::{io::HostResult, memory::{leak_to_shared_memory, FatPointer}};

#[link(wasm_import_module = "env")]
extern "C" {
    pub fn httpRequest(offset: u32, size: u32) -> FatPointer;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HttpRequestOutput {
    pub status_code: u16,
    pub headers: Option<BTreeMap<String, Vec<String>>>,
    pub body: Option<String>,
}

#[derive(Error, Debug)]
pub enum HttpRequestError {
    #[error("Bad serialization of input data: {0}")]
    BadSerialization(serde_json::Error),
    #[error("Bad deserialization of output data: {0}")]
    BadDeserialization(serde_json::Error),
    #[error("Request Failed: {0}")]
    RequestFailed(String)
}

/// Sends an HTTP request to the host environment and returns the result.
/// An HttpRequestError implies that there was some issue with sending or
/// receiving the request or response.
/// 
/// The url, method, headers, and body are serialized into JSON before being
/// sent to the host.
/// 
/// Example:
/// ```
/// let result = send_http_request(
///     "GET".into(),
///     "https://my-example-api.example.com/api/v2/test".into(),
///     &BTreeMap::from([
///         ("Content-Type".into(), vec!["application/json".into()]),
///         ("Authorization".into(), vec!["Bearer mytoken".into()])
///     ]),
///     &[]
/// );
/// 
/// log!("HTTP Request Result: {:?}", result);
/// ```
pub fn send_http_request(
    method: String,
    url: String,
    headers: &BTreeMap<String, Vec<String>>,
    body: &[u8],
) -> Result<HttpRequestOutput, HttpRequestError> {
    let input = json!({
        "url": url,
        "method": method,
        "headers": headers,
        "body": body
    });

    let serialized_input = serde_json::to_string(&input)
        .map_err(|e| HttpRequestError::BadSerialization(e))?;
    let input_ptr = leak_to_shared_memory(serialized_input.as_bytes());
    let output_ptr = unsafe { httpRequest(input_ptr.offset(), input_ptr.size()) };
    let data = unsafe { output_ptr.copy_data() };

    let host_result = serde_json::from_slice::<HostResult<HttpRequestOutput>>(&data)
        .map_err(|e| HttpRequestError::BadDeserialization(e))?;
    
    if host_result.ok {
        Ok(host_result.value)
    } else {
        Err(HttpRequestError::RequestFailed(host_result.error))
    }
}