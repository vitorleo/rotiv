use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::RotivError;

/// Request payload sent to the route-worker.
#[derive(Debug, Serialize)]
pub struct InvokeRequest {
    pub route_file: String,
    pub method: String,
    pub params: HashMap<String, String>,
    pub search_params: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

/// Successful response from the route-worker.
#[derive(Debug, Deserialize)]
pub struct InvokeResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

/// Error envelope returned by the route-worker on HTTP 500.
#[derive(Debug, Deserialize)]
struct ErrorEnvelope {
    error: RotivError,
}

/// Invoke a route via the route-worker process.
///
/// Uses a shared `reqwest::Client` for connection pooling.
/// HTTP 500 from the worker is deserialized as a `RotivError` and returned as `Err`.
pub async fn invoke_route(
    client: &reqwest::Client,
    worker_port: u16,
    req: InvokeRequest,
) -> Result<InvokeResponse, RotivError> {
    let url = format!("http://127.0.0.1:{}/_rotiv/invoke", worker_port);

    let response = client
        .post(&url)
        .json(&req)
        .send()
        .await
        .map_err(|e| {
            RotivError::new("E_WORKER_UNREACHABLE", e.to_string())
                .with_suggestion("Make sure the route worker is running")
        })?;

    let status = response.status();

    if status.as_u16() == 500 {
        // Worker signals an error via HTTP 500 with a JSON error envelope
        let body = response.text().await.unwrap_or_default();
        if let Ok(envelope) = serde_json::from_str::<ErrorEnvelope>(&body) {
            return Err(envelope.error);
        }
        return Err(RotivError::new(
            "E_ROUTE_ERROR",
            format!("route worker returned 500: {}", body),
        ));
    }

    // The worker forwards the route's HTTP response directly:
    // status code, headers, and raw body.
    let response_status = status.as_u16();
    let response_headers: HashMap<String, String> = response
        .headers()
        .iter()
        .filter_map(|(k, v)| v.to_str().ok().map(|v| (k.to_string(), v.to_string())))
        .collect();
    let body = response.text().await.unwrap_or_default();

    Ok(InvokeResponse {
        status: response_status,
        headers: response_headers,
        body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invoke_request_serializes_correctly() {
        let req = InvokeRequest {
            route_file: "/app/routes/index.tsx".to_string(),
            method: "GET".to_string(),
            params: HashMap::new(),
            search_params: "".to_string(),
            headers: HashMap::new(),
            body: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("route_file"));
        assert!(json.contains("/app/routes/index.tsx"));
        assert!(json.contains("\"method\":\"GET\""));
    }

    #[test]
    fn invoke_request_with_params_serializes() {
        let mut params = HashMap::new();
        params.insert("id".to_string(), "42".to_string());
        let req = InvokeRequest {
            route_file: "/app/routes/users/[id].tsx".to_string(),
            method: "GET".to_string(),
            params,
            search_params: "?foo=bar".to_string(),
            headers: HashMap::new(),
            body: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"id\":\"42\""));
    }
}
