//! CacheStorage 页面宿主。
//!
//! 本模块解析 `zero-engine` 同步 wire 请求，并在共享 [`StorageManager`] 上执行
//! per-origin Cache API 操作。

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json::json;
use zero_engine::CacheStorageHandler;
use zero_storage::{CacheRequest, CacheResponse, StorageManager};

const FIELD_SEP: char = '\x1f';
const HEADER_SEP: char = '\x1e';
const RESPONSE_PREFIX: &str = "__zwfr:";
const BYTES_PREFIX: &str = "__zw_bytes:";

#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum CacheStorageRequest {
    Open {
        name: String,
    },
    Has {
        name: String,
    },
    Delete {
        name: String,
        #[serde(default)]
        request: Option<CacheRequestWire>,
    },
    Keys,
    Match {
        request: CacheRequestWire,
        #[serde(default)]
        cache_name: Option<String>,
    },
    Put {
        cache_name: String,
        request: CacheRequestWire,
        response: CacheResponseWire,
    },
}

#[derive(Debug, Deserialize)]
struct CacheRequestWire {
    url: String,
    #[serde(default)]
    method: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CacheResponseWire {
    status: u16,
    #[serde(default, rename = "statusText")]
    status_text: String,
    #[serde(default)]
    headers: String,
    #[serde(default)]
    body: String,
    #[serde(default, rename = "bodyIsBytes")]
    body_is_bytes: bool,
}

#[derive(Debug, Serialize)]
struct CacheMatchResponse {
    response: Option<String>,
}

/// 构造由页面运行路径共享的 CacheStorage handler。
pub fn cache_storage_handler(storage: Arc<Mutex<StorageManager>>) -> CacheStorageHandler {
    Arc::new(move |origin, request| handle_request(&storage, origin, request))
}

fn handle_request(storage: &Mutex<StorageManager>, origin: &str, request: &str) -> Result<String, String> {
    if origin == "null" {
        return Err("SecurityError: CacheStorage is unavailable for opaque origins".to_string());
    }

    let request: CacheStorageRequest =
        serde_json::from_str(request).map_err(|error| format!("TypeError: invalid CacheStorage request: {error}"))?;
    let mut storage = storage
        .lock()
        .map_err(|_| "UnknownError: CacheStorage lock is poisoned".to_string())?;
    let response = dispatch_request(&mut storage, origin, request)?;
    serde_json::to_string(&response).map_err(|error| format!("UnknownError: failed to serialize response: {error}"))
}

fn dispatch_request(
    storage: &mut StorageManager,
    origin: &str,
    request: CacheStorageRequest,
) -> Result<serde_json::Value, String> {
    match request {
        CacheStorageRequest::Open { name } => {
            storage.cache_storage(origin).open(&name);
            Ok(json!({"name": name}))
        }
        CacheStorageRequest::Has { name } => {
            Ok(json!({"has": storage.cache_storage_ref(origin).is_some_and(|caches| caches.has(&name))}))
        }
        CacheStorageRequest::Delete { name, request } => {
            let deleted = if let Some(request) = request {
                let request = request.into_storage_request()?;
                storage
                    .cache_storage(origin)
                    .get_mut(&name)
                    .is_some_and(|cache| cache.delete(&request))
            } else {
                storage.cache_storage(origin).delete(&name)
            };
            Ok(json!({"deleted": deleted}))
        }
        CacheStorageRequest::Keys => {
            let mut keys: Vec<String> = storage
                .cache_storage_ref(origin)
                .map(|cache_storage| cache_storage.keys().into_iter().map(str::to_string).collect())
                .unwrap_or_default();
            keys.sort_unstable();
            Ok(json!({"keys": keys}))
        }
        CacheStorageRequest::Match { request, cache_name } => {
            let request = request.into_storage_request()?;
            let response = match cache_name {
                Some(name) => storage
                    .cache_storage_ref(origin)
                    .and_then(|cache_storage| cache_storage.get(&name))
                    .and_then(|cache| cache.match_request(&request)),
                None => storage
                    .cache_storage_ref(origin)
                    .and_then(|cache_storage| cache_storage.match_request(&request)),
            };
            let wire = response.map(cache_response_wire);
            serde_json::to_value(CacheMatchResponse { response: wire })
                .map_err(|error| format!("UnknownError: failed to serialize response: {error}"))
        }
        CacheStorageRequest::Put {
            cache_name,
            request,
            response,
        } => {
            let request = request.into_storage_request()?;
            let response = response.into_storage_response()?;
            storage
                .cache_storage(origin)
                .open(&cache_name)
                .put(request, response)
                .map_err(storage_error)?;
            Ok(json!({"ok": true}))
        }
    }
}

impl CacheRequestWire {
    fn into_storage_request(self) -> Result<CacheRequest, String> {
        if self.url.is_empty() {
            return Err("TypeError: Cache request URL is required".to_string());
        }
        let method = self.method.unwrap_or_else(|| "GET".to_string()).to_ascii_uppercase();
        Ok(CacheRequest::with_method(&self.url, &method))
    }
}

impl CacheResponseWire {
    fn into_storage_response(self) -> Result<CacheResponse, String> {
        let body = if self.body_is_bytes {
            decode_body_bytes_raw(&self.body)
                .ok_or_else(|| "TypeError: invalid Cache response byte body".to_string())?
        } else {
            self.body.into_bytes()
        };
        let headers: HashMap<String, String> = decode_headers(&self.headers).into_iter().collect();
        Ok(CacheResponse {
            status: self.status,
            status_text: self.status_text,
            headers,
            body,
        })
    }
}

fn cache_response_wire(response: &CacheResponse) -> String {
    let mut headers: Vec<(String, String)> = response
        .headers
        .iter()
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect();
    headers.sort_unstable_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    let body = match std::str::from_utf8(&response.body) {
        Ok(text) => text.to_string(),
        Err(_) => encode_body_bytes(&response.body),
    };
    format!(
        "{RESPONSE_PREFIX}{status}{FIELD_SEP}{status_text}{FIELD_SEP}{headers}{FIELD_SEP}{body}",
        status = response.status,
        status_text = response.status_text,
        headers = encode_headers(&headers),
        body = body,
    )
}

fn encode_headers(headers: &[(String, String)]) -> String {
    let mut out = String::new();
    for (i, (name, value)) in headers.iter().enumerate() {
        if i > 0 {
            out.push(HEADER_SEP);
        }
        out.push_str(name);
        out.push(HEADER_SEP);
        out.push_str(value);
    }
    out
}

fn decode_headers(wire: &str) -> Vec<(String, String)> {
    if wire.is_empty() {
        return Vec::new();
    }
    let parts: Vec<&str> = wire.split(HEADER_SEP).collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < parts.len() {
        out.push((parts[i].to_string(), parts[i + 1].to_string()));
        i += 2;
    }
    out
}

fn encode_body_bytes(bytes: &[u8]) -> String {
    let mut out = String::from(BYTES_PREFIX);
    for (i, byte) in bytes.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&byte.to_string());
    }
    out
}

fn decode_body_bytes_raw(wire: &str) -> Option<Vec<u8>> {
    if !wire.starts_with(BYTES_PREFIX) {
        return None;
    }
    let rest = &wire[BYTES_PREFIX.len()..];
    if rest.is_empty() {
        return Some(Vec::new());
    }
    let mut bytes = Vec::new();
    for part in rest.split(',') {
        match part.parse::<u8>() {
            Ok(byte) => bytes.push(byte),
            Err(_) => return None,
        }
    }
    Some(bytes)
}

fn storage_error(error: zero_storage::StorageError) -> String {
    match error {
        zero_storage::StorageError::QuotaExceeded(message) => format!("QuotaExceededError: {message}"),
        zero_storage::StorageError::InvalidKey(message) => format!("DataError: {message}"),
        zero_storage::StorageError::StoreNotFound(message) => format!("NotFoundError: {message}"),
        zero_storage::StorageError::KeyNotFound(message) => format!("NotFoundError: {message}"),
        zero_storage::StorageError::Serialization(message) => format!("DataError: {message}"),
        zero_storage::StorageError::Database(message) => format!("InvalidStateError: {message}"),
        zero_storage::StorageError::Io(message) => format!("UnknownError: {message}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(op: serde_json::Value) -> String {
        serde_json::to_string(&op).unwrap()
    }

    fn call(handler: &CacheStorageHandler, origin: &str, request: serde_json::Value) -> serde_json::Value {
        let response = handler(origin, &self::request(request)).unwrap();
        serde_json::from_str(&response).unwrap()
    }

    #[test]
    fn cache_storage_handler_put_and_match_round_trips_response() {
        let handler = cache_storage_handler(Arc::new(Mutex::new(StorageManager::new())));
        assert_eq!(
            call(&handler, "https://example.com", json!({"op": "open", "name": "v1"}))["name"],
            "v1"
        );
        call(
            &handler,
            "https://example.com",
            json!({
                "op": "put",
                "cache_name": "v1",
                "request": {"url": "https://example.com/data", "method": "GET"},
                "response": {
                    "status": 201,
                    "statusText": "Created",
                    "headers": "content-type\u{1e}text/plain",
                    "body": "cached body",
                    "bodyIsBytes": false
                }
            }),
        );

        let matched = call(
            &handler,
            "https://example.com",
            json!({
                "op": "match",
                "cache_name": "v1",
                "request": {"url": "https://example.com/data", "method": "GET"}
            }),
        );
        let wire = matched["response"].as_str().unwrap();
        assert!(wire.starts_with("__zwfr:201\u{1f}Created\u{1f}content-type\u{1e}text/plain\u{1f}"));
        assert!(wire.ends_with("cached body"));
    }

    #[test]
    fn cache_storage_handler_is_origin_scoped() {
        let handler = cache_storage_handler(Arc::new(Mutex::new(StorageManager::new())));
        call(
            &handler,
            "https://a.example",
            json!({
                "op": "put",
                "cache_name": "v1",
                "request": {"url": "https://a.example/data"},
                "response": {"status": 200, "statusText": "OK", "headers": "", "body": "a"}
            }),
        );

        let missing = call(
            &handler,
            "https://b.example",
            json!({
                "op": "match",
                "cache_name": "v1",
                "request": {"url": "https://a.example/data"}
            }),
        );
        assert!(missing["response"].is_null());
    }

    #[test]
    fn cache_storage_handler_lists_and_deletes_caches_and_entries() {
        let handler = cache_storage_handler(Arc::new(Mutex::new(StorageManager::new())));
        call(&handler, "https://example.com", json!({"op": "open", "name": "b"}));
        call(&handler, "https://example.com", json!({"op": "open", "name": "a"}));
        let listed = call(&handler, "https://example.com", json!({"op": "keys"}));
        assert_eq!(listed["keys"], json!(["a", "b"]));
        let has_a = call(&handler, "https://example.com", json!({"op": "has", "name": "a"}));
        assert_eq!(has_a["has"], true);

        call(
            &handler,
            "https://example.com",
            json!({
                "op": "put",
                "cache_name": "a",
                "request": {"url": "https://example.com/data"},
                "response": {"status": 200, "statusText": "OK", "headers": "", "body": "cached"}
            }),
        );
        let entry_deleted = call(
            &handler,
            "https://example.com",
            json!({
                "op": "delete",
                "name": "a",
                "request": {"url": "https://example.com/data"}
            }),
        );
        assert_eq!(entry_deleted["deleted"], true);
        let missing = call(
            &handler,
            "https://example.com",
            json!({"op": "match", "cache_name": "a", "request": {"url": "https://example.com/data"}}),
        );
        assert!(missing["response"].is_null());

        let cache_deleted = call(&handler, "https://example.com", json!({"op": "delete", "name": "a"}));
        assert_eq!(cache_deleted["deleted"], true);
        let has_a = call(&handler, "https://example.com", json!({"op": "has", "name": "a"}));
        assert_eq!(has_a["has"], false);
    }

    #[test]
    fn cache_storage_handler_rejects_opaque_origin() {
        let handler = cache_storage_handler(Arc::new(Mutex::new(StorageManager::new())));
        let error = handler(
            "null",
            &request(json!({
                "op": "open",
                "name": "v1"
            })),
        )
        .unwrap_err();
        assert!(error.starts_with("SecurityError:"));
    }

    #[test]
    fn cache_storage_handler_rejects_invalid_byte_body_wire() {
        let handler = cache_storage_handler(Arc::new(Mutex::new(StorageManager::new())));
        let error = handler(
            "https://example.com",
            &request(json!({
                "op": "put",
                "cache_name": "v1",
                "request": {"url": "https://example.com/data"},
                "response": {
                    "status": 200,
                    "headers": "",
                    "body": "not-byte-wire",
                    "bodyIsBytes": true
                }
            })),
        )
        .unwrap_err();
        assert_eq!(error, "TypeError: invalid Cache response byte body");
    }
}
