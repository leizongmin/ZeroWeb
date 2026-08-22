//! CacheStorage 页面宿主。
//!
//! 本模块解析 `zero-engine` 同步 wire 请求，并在共享 [`StorageManager`] 上执行
//! per-origin Cache API 操作。

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json::json;
use zero_engine::CacheStorageHandler;
use zero_storage::{Cache, CacheQueryOptions, CacheRequest, CacheResponse, StorageManager};

const FIELD_SEP: char = '\x1f';
const HEADER_SEP: char = '\x1e';
const RESPONSE_PREFIX: &str = "__zwfr:";
const BYTES_PREFIX: &str = "__zw_bytes:";

#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum CacheStorageRequest {
    Open {
        #[serde(default)]
        name: String,
        #[serde(default)]
        name_units: Option<String>,
    },
    Has {
        #[serde(default)]
        name: String,
        #[serde(default)]
        name_units: Option<String>,
    },
    Delete {
        #[serde(default)]
        name: String,
        #[serde(default)]
        name_units: Option<String>,
        #[serde(default)]
        request: Option<CacheRequestWire>,
        #[serde(default)]
        options: CacheQueryOptionsWire,
        #[serde(default)]
        cache_id: Option<u64>,
    },
    Keys,
    Match {
        request: CacheRequestWire,
        #[serde(default)]
        cache_name: Option<String>,
        #[serde(default)]
        cache_name_units: Option<String>,
        #[serde(default)]
        cache_id: Option<u64>,
        #[serde(default)]
        options: CacheQueryOptionsWire,
    },
    MatchAll {
        #[serde(default)]
        cache_name: String,
        #[serde(default)]
        cache_name_units: Option<String>,
        #[serde(default)]
        cache_id: Option<u64>,
        #[serde(default)]
        request: Option<CacheRequestWire>,
        #[serde(default)]
        options: CacheQueryOptionsWire,
    },
    CacheKeys {
        #[serde(default)]
        cache_name: String,
        #[serde(default)]
        cache_name_units: Option<String>,
        #[serde(default)]
        cache_id: Option<u64>,
        #[serde(default)]
        request: Option<CacheRequestWire>,
        #[serde(default)]
        options: CacheQueryOptionsWire,
    },
    Put {
        #[serde(default)]
        cache_name: String,
        #[serde(default)]
        cache_name_units: Option<String>,
        #[serde(default)]
        cache_id: Option<u64>,
        request: CacheRequestWire,
        response: CacheResponseWire,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct CacheRequestWire {
    url: String,
    #[serde(default)]
    method: Option<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    headers: String,
}

#[derive(Debug, Default, Deserialize)]
struct CacheQueryOptionsWire {
    #[serde(default, rename = "ignoreSearch")]
    ignore_search: bool,
    #[serde(default, rename = "ignoreMethod")]
    ignore_method: bool,
    #[serde(default, rename = "ignoreVary")]
    ignore_vary: bool,
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

#[derive(Debug, Serialize)]
struct CacheMatchAllResponse {
    responses: Vec<String>,
}

#[derive(Debug, Serialize)]
struct CacheKeysResponse {
    requests: Vec<CacheRequestWire>,
}

#[derive(Debug, Serialize)]
struct CacheStorageOpenResponse {
    name: String,
    #[serde(rename = "name_units")]
    name_units: String,
    cache_id: u64,
}

#[derive(Debug, Serialize)]
struct CacheStorageKeysResponse {
    keys: Vec<String>,
    #[serde(rename = "keys_units")]
    keys_units: Vec<String>,
}

#[derive(Debug, Default)]
struct CacheStorageHostState {
    next_cache_id: u64,
    active_cache_ids: HashMap<(String, String), u64>,
    doomed_caches: HashMap<(String, u64), Cache>,
}

/// 构造由页面运行路径共享的 CacheStorage handler。
pub fn cache_storage_handler(storage: Arc<Mutex<StorageManager>>) -> CacheStorageHandler {
    let state = Arc::new(Mutex::new(CacheStorageHostState::default()));
    Arc::new(move |origin, request| handle_request(&storage, &state, origin, request))
}

fn handle_request(
    storage: &Mutex<StorageManager>,
    state: &Mutex<CacheStorageHostState>,
    origin: &str,
    request: &str,
) -> Result<String, String> {
    if origin == "null" {
        return Err("SecurityError: CacheStorage is unavailable for opaque origins".to_string());
    }

    let request: CacheStorageRequest =
        serde_json::from_str(request).map_err(|error| format!("TypeError: invalid CacheStorage request: {error}"))?;
    let mut state = state
        .lock()
        .map_err(|_| "UnknownError: CacheStorage state lock is poisoned".to_string())?;
    let mut storage = storage
        .lock()
        .map_err(|_| "UnknownError: CacheStorage lock is poisoned".to_string())?;
    let response = dispatch_request(&mut storage, &mut state, origin, request)?;
    serde_json::to_string(&response).map_err(|error| format!("UnknownError: failed to serialize response: {error}"))
}

fn dispatch_request(
    storage: &mut StorageManager,
    state: &mut CacheStorageHostState,
    origin: &str,
    request: CacheStorageRequest,
) -> Result<serde_json::Value, String> {
    match request {
        CacheStorageRequest::Open { name, name_units } => {
            let name = cache_name_from_wire(&name, name_units.as_deref())?;
            storage.cache_storage(origin).open(&name);
            let cache_id = state.ensure_active_cache_id(origin, &name);
            serde_json::to_value(CacheStorageOpenResponse {
                name: display_cache_name(&name),
                name_units: encode_cache_name_units(&name),
                cache_id,
            })
            .map_err(|error| format!("UnknownError: failed to serialize response: {error}"))
        }
        CacheStorageRequest::Has { name, name_units } => {
            let name = cache_name_from_wire(&name, name_units.as_deref())?;
            Ok(json!({"has": storage.cache_storage_ref(origin).is_some_and(|caches| caches.has(&name))}))
        }
        CacheStorageRequest::Delete {
            name,
            name_units,
            request,
            options,
            cache_id,
        } => {
            let name = cache_name_from_wire(&name, name_units.as_deref())?;
            let deleted = if let Some(request) = request {
                let request = request.into_storage_request()?;
                let options = options.into_storage_options();
                if let Some(cache_id) = cache_id {
                    if let Some(cache) = state.doomed_caches.get_mut(&cache_instance_key(origin, cache_id)) {
                        cache.delete_with_options(&request, options)
                    } else if state.active_cache_id_matches(origin, &name, cache_id) {
                        storage
                            .cache_storage(origin)
                            .get_mut(&name)
                            .is_some_and(|cache| cache.delete_with_options(&request, options))
                    } else {
                        false
                    }
                } else {
                    storage
                        .cache_storage(origin)
                        .get_mut(&name)
                        .is_some_and(|cache| cache.delete_with_options(&request, options))
                }
            } else {
                let doomed = storage
                    .cache_storage_ref(origin)
                    .and_then(|caches| caches.get(&name))
                    .cloned();
                let deleted = storage.cache_storage(origin).delete(&name);
                let removed = if deleted {
                    state.remove_active_cache_id(origin, &name).zip(doomed)
                } else {
                    None
                };
                if let Some((cache_id, cache)) = removed {
                    state.doomed_caches.insert(cache_instance_key(origin, cache_id), cache);
                }
                deleted
            };
            Ok(json!({"deleted": deleted}))
        }
        CacheStorageRequest::Keys => {
            let keys: Vec<String> = storage
                .cache_storage_ref(origin)
                .map(|cache_storage| cache_storage.keys().into_iter().map(str::to_string).collect())
                .unwrap_or_default();
            let keys_units = keys.iter().map(|name| encode_cache_name_units(name)).collect();
            let keys = keys.into_iter().map(|name| display_cache_name(&name)).collect();
            serde_json::to_value(CacheStorageKeysResponse { keys, keys_units })
                .map_err(|error| format!("UnknownError: failed to serialize response: {error}"))
        }
        CacheStorageRequest::Match {
            request,
            cache_name,
            cache_name_units,
            cache_id,
            options,
        } => {
            let request = request.into_storage_request()?;
            let options = options.into_storage_options();
            let cache_name = match (cache_name, cache_name_units) {
                (Some(name), units) => Some(cache_name_from_wire(&name, units.as_deref())?),
                (None, Some(units)) => Some(cache_name_from_wire("", Some(&units))?),
                (None, None) => None,
            };
            let response = match cache_name {
                Some(name) => match cache_id {
                    Some(cache_id) => state
                        .doomed_caches
                        .get(&cache_instance_key(origin, cache_id))
                        .and_then(|cache| cache.match_request_with_options(&request, options))
                        .or_else(|| {
                            if state.active_cache_id_matches(origin, &name, cache_id) {
                                storage
                                    .cache_storage_ref(origin)
                                    .and_then(|cache_storage| cache_storage.get(&name))
                                    .and_then(|cache| cache.match_request_with_options(&request, options))
                            } else {
                                None
                            }
                        }),
                    None => storage
                        .cache_storage_ref(origin)
                        .and_then(|cache_storage| cache_storage.get(&name))
                        .and_then(|cache| cache.match_request_with_options(&request, options)),
                },
                None => storage
                    .cache_storage_ref(origin)
                    .and_then(|cache_storage| cache_storage.match_request_with_options(&request, options)),
            };
            let wire = response.map(cache_response_wire);
            serde_json::to_value(CacheMatchResponse { response: wire })
                .map_err(|error| format!("UnknownError: failed to serialize response: {error}"))
        }
        CacheStorageRequest::MatchAll {
            cache_name,
            cache_name_units,
            cache_id,
            request,
            options,
        } => {
            let cache_name = cache_name_from_wire(&cache_name, cache_name_units.as_deref())?;
            let request = request.map(CacheRequestWire::into_storage_request).transpose()?;
            let options = options.into_storage_options();
            let responses = selected_cache_ref(state, storage, origin, &cache_name, cache_id)
                .map(|cache| cache_response_list(cache, request.as_ref(), options))
                .unwrap_or_default();
            serde_json::to_value(CacheMatchAllResponse { responses })
                .map_err(|error| format!("UnknownError: failed to serialize response: {error}"))
        }
        CacheStorageRequest::CacheKeys {
            cache_name,
            cache_name_units,
            cache_id,
            request,
            options,
        } => {
            let cache_name = cache_name_from_wire(&cache_name, cache_name_units.as_deref())?;
            let request = request.map(CacheRequestWire::into_storage_request).transpose()?;
            let options = options.into_storage_options();
            let requests = match selected_cache_ref(state, storage, origin, &cache_name, cache_id) {
                Some(cache) => match &request {
                    Some(request) => cache.request_keys_with_options(request, options),
                    None => cache.request_keys(),
                }
                .into_iter()
                .map(|request| CacheRequestWire {
                    url: request.url.clone(),
                    method: Some(request.method.clone()),
                    headers: encode_headers(&request.headers),
                })
                .collect(),
                None => Vec::new(),
            };
            serde_json::to_value(CacheKeysResponse { requests })
                .map_err(|error| format!("UnknownError: failed to serialize response: {error}"))
        }
        CacheStorageRequest::Put {
            cache_name,
            cache_name_units,
            cache_id,
            request,
            response,
        } => {
            let cache_name = cache_name_from_wire(&cache_name, cache_name_units.as_deref())?;
            let request = request.into_storage_request()?;
            let response = response.into_storage_response()?;
            if let Some(cache_id) = cache_id {
                if let Some(cache) = state.doomed_caches.get_mut(&cache_instance_key(origin, cache_id)) {
                    cache.put(request, response).map_err(storage_error)?;
                } else if state.active_cache_id_matches(origin, &cache_name, cache_id) {
                    storage
                        .cache_storage(origin)
                        .open(&cache_name)
                        .put(request, response)
                        .map_err(storage_error)?;
                } else {
                    return Err("InvalidStateError: Cache backing store is no longer available".to_string());
                }
            } else {
                storage
                    .cache_storage(origin)
                    .open(&cache_name)
                    .put(request, response)
                    .map_err(storage_error)?;
            }
            Ok(json!({"ok": true}))
        }
    }
}

impl CacheStorageHostState {
    fn ensure_active_cache_id(&mut self, origin: &str, name: &str) -> u64 {
        let key = active_cache_key(origin, name);
        if let Some(id) = self.active_cache_ids.get(&key) {
            return *id;
        }
        self.next_cache_id = self.next_cache_id.saturating_add(1).max(1);
        let id = self.next_cache_id;
        self.active_cache_ids.insert(key, id);
        id
    }

    fn remove_active_cache_id(&mut self, origin: &str, name: &str) -> Option<u64> {
        self.active_cache_ids.remove(&active_cache_key(origin, name))
    }

    fn active_cache_id_matches(&self, origin: &str, name: &str, cache_id: u64) -> bool {
        self.active_cache_ids
            .get(&active_cache_key(origin, name))
            .is_some_and(|active| *active == cache_id)
    }
}

fn active_cache_key(origin: &str, name: &str) -> (String, String) {
    (origin.to_string(), name.to_string())
}

fn cache_instance_key(origin: &str, cache_id: u64) -> (String, u64) {
    (origin.to_string(), cache_id)
}

fn selected_cache_ref<'a>(
    state: &'a CacheStorageHostState,
    storage: &'a StorageManager,
    origin: &str,
    name: &str,
    cache_id: Option<u64>,
) -> Option<&'a Cache> {
    match cache_id {
        Some(cache_id) => state
            .doomed_caches
            .get(&cache_instance_key(origin, cache_id))
            .or_else(|| {
                if state.active_cache_id_matches(origin, name, cache_id) {
                    storage
                        .cache_storage_ref(origin)
                        .and_then(|cache_storage| cache_storage.get(name))
                } else {
                    None
                }
            }),
        None => storage
            .cache_storage_ref(origin)
            .and_then(|cache_storage| cache_storage.get(name)),
    }
}

fn cache_response_list(cache: &Cache, request: Option<&CacheRequest>, options: CacheQueryOptions) -> Vec<String> {
    match request {
        Some(request) => cache
            .match_all_with_options(request, options)
            .into_iter()
            .map(cache_response_wire)
            .collect(),
        None => cache
            .request_keys()
            .into_iter()
            .filter_map(|request| cache.match_request(request))
            .map(cache_response_wire)
            .collect(),
    }
}

fn cache_name_from_wire(name: &str, name_units: Option<&str>) -> Result<String, String> {
    if !name.is_empty() || name_units == Some("") {
        return Ok(name.to_string());
    }
    match name_units {
        Some(units) => decode_cache_name_units(units),
        None => Ok(String::new()),
    }
}

fn encode_cache_name_units(name: &str) -> String {
    if let Some(units) = name.strip_prefix("__zw_domstring16:") {
        return units.to_string();
    }
    let mut out = String::new();
    for unit in name.encode_utf16() {
        out.push_str(&format!("{unit:04x}"));
    }
    out
}

fn decode_cache_name_units(units: &str) -> Result<String, String> {
    if !units.as_bytes().chunks_exact(4).remainder().is_empty() {
        return Err("TypeError: invalid CacheStorage name code units".to_string());
    }
    for byte in units.bytes() {
        if !byte.is_ascii_hexdigit() {
            return Err("TypeError: invalid CacheStorage name code units".to_string());
        }
    }
    Ok(format!("__zw_domstring16:{units}"))
}

fn display_cache_name(name: &str) -> String {
    name.strip_prefix("__zw_domstring16:")
        .map(decode_cache_name_units_lossy)
        .unwrap_or_else(|| name.to_string())
}

fn decode_cache_name_units_lossy(units: &str) -> String {
    let mut utf16 = Vec::new();
    for chunk in units.as_bytes().chunks_exact(4) {
        let Ok(hex) = std::str::from_utf8(chunk) else {
            return String::new();
        };
        let Ok(unit) = u16::from_str_radix(hex, 16) else {
            return String::new();
        };
        utf16.push(unit);
    }
    String::from_utf16_lossy(&utf16)
}

impl CacheQueryOptionsWire {
    fn into_storage_options(self) -> CacheQueryOptions {
        CacheQueryOptions {
            ignore_search: self.ignore_search,
            ignore_method: self.ignore_method,
            ignore_vary: self.ignore_vary,
        }
    }
}

impl CacheRequestWire {
    fn into_storage_request(self) -> Result<CacheRequest, String> {
        if self.url.is_empty() {
            return Err("TypeError: Cache request URL is required".to_string());
        }
        let method = self.method.unwrap_or_else(|| "GET".to_string()).to_ascii_uppercase();
        Ok(CacheRequest::with_method_and_headers(
            &self.url,
            &method,
            decode_headers(&self.headers),
        ))
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
    fn cache_storage_handler_preserves_domstring_code_unit_names() {
        let handler = cache_storage_handler(Arc::new(Mutex::new(StorageManager::new())));
        let unpaired_name_units = "0075006e007000610069007200650064d800";
        let converted_name_units = "0075006e007000610069007200650064fffd";

        let opened = call(
            &handler,
            "https://example.com",
            json!({"op": "open", "name_units": unpaired_name_units}),
        );
        assert_eq!(opened["name_units"], unpaired_name_units);

        let listed = call(&handler, "https://example.com", json!({"op": "keys"}));
        assert_eq!(listed["keys_units"], json!([unpaired_name_units]));

        let has_original = call(
            &handler,
            "https://example.com",
            json!({"op": "has", "name_units": unpaired_name_units}),
        );
        assert_eq!(has_original["has"], true);

        let has_converted = call(
            &handler,
            "https://example.com",
            json!({"op": "has", "name_units": converted_name_units}),
        );
        assert_eq!(has_converted["has"], false);
    }

    #[test]
    fn cache_storage_handler_dooms_deleted_cache_instances() {
        let handler = cache_storage_handler(Arc::new(Mutex::new(StorageManager::new())));
        let opened = call(&handler, "https://example.com", json!({"op": "open", "name": "v1"}));
        let first_cache_id = opened["cache_id"].as_u64().unwrap();

        assert_eq!(
            call(&handler, "https://example.com", json!({"op": "delete", "name": "v1"}))["deleted"],
            true
        );
        call(
            &handler,
            "https://example.com",
            json!({
                "op": "put",
                "cache_name": "v1",
                "cache_id": first_cache_id,
                "request": {"url": "https://example.com/old"},
                "response": {"status": 200, "statusText": "OK", "headers": "", "body": "old"}
            }),
        );

        let active_keys = call(&handler, "https://example.com", json!({"op": "keys"}));
        assert_eq!(active_keys["keys"], json!([]));

        let reopened = call(&handler, "https://example.com", json!({"op": "open", "name": "v1"}));
        let second_cache_id = reopened["cache_id"].as_u64().unwrap();
        assert_ne!(first_cache_id, second_cache_id);

        let second_keys = call(
            &handler,
            "https://example.com",
            json!({"op": "cache_keys", "cache_name": "v1", "cache_id": second_cache_id}),
        );
        assert_eq!(second_keys["requests"], json!([]));

        let first_keys = call(
            &handler,
            "https://example.com",
            json!({"op": "cache_keys", "cache_name": "v1", "cache_id": first_cache_id}),
        );
        assert_eq!(first_keys["requests"][0]["url"], "https://example.com/old");
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
        assert_eq!(listed["keys"], json!(["b", "a"]));
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
    fn cache_storage_handler_lists_cache_requests_and_matches_all() {
        let handler = cache_storage_handler(Arc::new(Mutex::new(StorageManager::new())));
        call(
            &handler,
            "https://example.com",
            json!({
                "op": "put",
                "cache_name": "runtime",
                "request": {"url": "https://example.com/data", "method": "GET"},
                "response": {"status": 200, "statusText": "OK", "headers": "", "body": "get"}
            }),
        );
        call(
            &handler,
            "https://example.com",
            json!({
                "op": "put",
                "cache_name": "runtime",
                "request": {"url": "https://example.com/data", "method": "POST"},
                "response": {"status": 201, "statusText": "Created", "headers": "", "body": "post"}
            }),
        );

        let listed = call(
            &handler,
            "https://example.com",
            json!({"op": "cache_keys", "cache_name": "runtime"}),
        );
        assert_eq!(
            listed["requests"],
            json!([
                {"url": "https://example.com/data", "method": "GET"},
                {"url": "https://example.com/data", "method": "POST"}
            ])
        );

        let filtered_keys = call(
            &handler,
            "https://example.com",
            json!({
                "op": "cache_keys",
                "cache_name": "runtime",
                "request": {"url": "https://example.com/data", "method": "POST"}
            }),
        );
        assert_eq!(
            filtered_keys["requests"],
            json!([{"url": "https://example.com/data", "method": "POST"}])
        );

        let matched = call(
            &handler,
            "https://example.com",
            json!({
                "op": "match_all",
                "cache_name": "runtime",
                "request": {"url": "https://example.com/data", "method": "POST"}
            }),
        );
        let responses = matched["responses"].as_array().unwrap();
        assert_eq!(responses.len(), 1);
        assert!(responses[0].as_str().unwrap().ends_with("post"));

        let all = call(
            &handler,
            "https://example.com",
            json!({"op": "match_all", "cache_name": "runtime"}),
        );
        assert_eq!(all["responses"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn cache_storage_handler_applies_query_options() {
        let handler = cache_storage_handler(Arc::new(Mutex::new(StorageManager::new())));
        call(
            &handler,
            "https://example.com",
            json!({
                "op": "put",
                "cache_name": "runtime",
                "request": {"url": "https://example.com/data?version=1", "method": "GET"},
                "response": {"status": 200, "statusText": "OK", "headers": "", "body": "get"}
            }),
        );
        call(
            &handler,
            "https://example.com",
            json!({
                "op": "put",
                "cache_name": "runtime",
                "request": {"url": "https://example.com/other?version=1", "method": "GET"},
                "response": {"status": 200, "statusText": "OK", "headers": "", "body": "other"}
            }),
        );

        let strict = call(
            &handler,
            "https://example.com",
            json!({
                "op": "match",
                "cache_name": "runtime",
                "request": {"url": "https://example.com/data?version=2", "method": "HEAD"}
            }),
        );
        assert!(strict["response"].is_null());

        let matched = call(
            &handler,
            "https://example.com",
            json!({
                "op": "match",
                "cache_name": "runtime",
                "request": {"url": "https://example.com/data?version=2", "method": "HEAD"},
                "options": {"ignoreSearch": true, "ignoreMethod": true}
            }),
        );
        assert!(matched["response"].as_str().unwrap().ends_with("get"));

        let filtered_keys = call(
            &handler,
            "https://example.com",
            json!({
                "op": "cache_keys",
                "cache_name": "runtime",
                "request": {"url": "https://example.com/data?version=3", "method": "POST"},
                "options": {"ignoreSearch": true, "ignoreMethod": true}
            }),
        );
        assert_eq!(
            filtered_keys["requests"],
            json!([{"url": "https://example.com/data?version=1", "method": "GET"}])
        );

        let deleted = call(
            &handler,
            "https://example.com",
            json!({
                "op": "delete",
                "name": "runtime",
                "request": {"url": "https://example.com/data?version=4", "method": "POST"},
                "options": {"ignoreSearch": true, "ignoreMethod": true}
            }),
        );
        assert_eq!(deleted["deleted"], true);
        let all = call(
            &handler,
            "https://example.com",
            json!({"op": "cache_keys", "cache_name": "runtime"}),
        );
        assert_eq!(
            all["requests"],
            json!([{"url": "https://example.com/other?version=1", "method": "GET"}])
        );
    }

    #[test]
    fn cache_storage_handler_applies_vary_query_options() {
        let handler = cache_storage_handler(Arc::new(Mutex::new(StorageManager::new())));
        call(
            &handler,
            "https://example.com",
            json!({
                "op": "put",
                "cache_name": "runtime",
                "request": {
                    "url": "https://example.com/c",
                    "method": "GET",
                    "headers": "Cookies\u{1e}is-for-cookie"
                },
                "response": {
                    "status": 200,
                    "statusText": "OK",
                    "headers": "Vary\u{1e}Cookies",
                    "body": "cookie"
                }
            }),
        );

        let mismatched = call(
            &handler,
            "https://example.com",
            json!({
                "op": "cache_keys",
                "cache_name": "runtime",
                "request": {"url": "https://example.com/c", "method": "GET"}
            }),
        );
        assert_eq!(mismatched["requests"], json!([]));

        let ignored = call(
            &handler,
            "https://example.com",
            json!({
                "op": "cache_keys",
                "cache_name": "runtime",
                "request": {"url": "https://example.com/c", "method": "GET"},
                "options": {"ignoreVary": true}
            }),
        );
        assert_eq!(
            ignored["requests"],
            json!([{
                "url": "https://example.com/c",
                "method": "GET",
                "headers": "Cookies\u{1e}is-for-cookie"
            }])
        );

        let not_deleted = call(
            &handler,
            "https://example.com",
            json!({
                "op": "delete",
                "name": "runtime",
                "request": {"url": "https://example.com/c", "method": "GET"}
            }),
        );
        assert_eq!(not_deleted["deleted"], false);

        let deleted = call(
            &handler,
            "https://example.com",
            json!({
                "op": "delete",
                "name": "runtime",
                "request": {"url": "https://example.com/c", "method": "GET"},
                "options": {"ignoreVary": true}
            }),
        );
        assert_eq!(deleted["deleted"], true);
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
