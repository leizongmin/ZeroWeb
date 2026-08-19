//! Shared Service Worker registration URL validation.

use url::Url;

/// Stable web-visible registration validation category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceWorkerRegistrationErrorKind {
    /// Web IDL or URL parsing/normalization rejected the input.
    Type,
    /// The registration violates secure-context or same-origin policy.
    Security,
}

impl ServiceWorkerRegistrationErrorKind {
    /// JavaScript exception name for this validation category.
    pub fn exception_name(self) -> &'static str {
        match self {
            Self::Type => "TypeError",
            Self::Security => "SecurityError",
        }
    }
}

/// Service Worker registration URL validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceWorkerRegistrationError {
    /// Web-visible error category.
    pub kind: ServiceWorkerRegistrationErrorKind,
    /// Stable diagnostic safe for renderer exposure.
    pub message: &'static str,
}

impl std::fmt::Display for ServiceWorkerRegistrationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.message)
    }
}

impl std::error::Error for ServiceWorkerRegistrationError {}

/// Validate and normalize registration URLs against a trusted document URL.
pub fn validate_service_worker_registration(
    script_url: &str,
    scope: Option<&str>,
    document: &Url,
) -> Result<(Url, Url, String), ServiceWorkerRegistrationError> {
    let secure = document.scheme() == "https"
        || (document.scheme() == "http"
            && document.host_str().is_some_and(|host| {
                host == "localhost" || host.parse::<std::net::IpAddr>().is_ok_and(|ip| ip.is_loopback())
            }));
    if !secure {
        return Err(security_error("Service Worker registration requires a secure context"));
    }

    let mut script = document
        .join(script_url)
        .map_err(|_| type_error("invalid Service Worker script URL"))?;
    if !matches!(script.scheme(), "http" | "https") {
        return Err(type_error("Service Worker script URL must use http or https"));
    }
    if script.origin() != document.origin() {
        return Err(security_error("Service Worker script URL must be same-origin"));
    }
    script.set_fragment(None);
    if has_encoded_path_separator(&script) {
        return Err(type_error(
            "Service Worker script URL must not contain an encoded path separator",
        ));
    }

    let mut scope = match scope {
        Some(value) => document
            .join(value)
            .map_err(|_| type_error("invalid Service Worker scope"))?,
        None => script
            .join("./")
            .map_err(|_| type_error("invalid default Service Worker scope"))?,
    };
    if !matches!(scope.scheme(), "http" | "https") {
        return Err(type_error("Service Worker scope must use http or https"));
    }
    if scope.origin() != document.origin() {
        return Err(security_error("Service Worker scope must be same-origin"));
    }
    scope.set_fragment(None);
    if has_encoded_path_separator(&scope) {
        return Err(type_error(
            "Service Worker scope must not contain an encoded path separator",
        ));
    }

    let script_directory = script
        .join("./")
        .map_err(|_| type_error("invalid Service Worker script directory"))?;
    if !scope.path().starts_with(script_directory.path()) {
        return Err(security_error("Service Worker scope exceeds the script directory"));
    }

    Ok((script, scope, document.origin().ascii_serialization()))
}

/// Parse a document URL, then validate and normalize registration URLs.
pub fn validate_service_worker_registration_for_document(
    script_url: &str,
    scope: Option<&str>,
    document_url: &str,
) -> Result<(Url, Url, String), ServiceWorkerRegistrationError> {
    let document = Url::parse(document_url).map_err(|_| type_error("invalid Service Worker document URL"))?;
    validate_service_worker_registration(script_url, scope, &document)
}

fn has_encoded_path_separator(url: &Url) -> bool {
    let path = url.path().as_bytes();
    path.windows(3).any(|bytes| {
        bytes[0] == b'%'
            && ((bytes[1] == b'2' && matches!(bytes[2], b'f' | b'F'))
                || (bytes[1] == b'5' && matches!(bytes[2], b'c' | b'C')))
    })
}

fn type_error(message: &'static str) -> ServiceWorkerRegistrationError {
    ServiceWorkerRegistrationError {
        kind: ServiceWorkerRegistrationErrorKind::Type,
        message,
    }
}

fn security_error(message: &'static str) -> ServiceWorkerRegistrationError {
    ServiceWorkerRegistrationError {
        kind: ServiceWorkerRegistrationErrorKind::Security,
        message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document() -> Url {
        Url::parse("https://example.test/service-worker/page.html").unwrap()
    }

    #[test]
    fn fragments_are_removed_from_script_and_scope() {
        let (script, scope, _) =
            validate_service_worker_registration("resources/sw.js#script", Some("resources/scope#scope"), &document())
                .unwrap();
        assert_eq!(script.as_str(), "https://example.test/service-worker/resources/sw.js");
        assert_eq!(scope.as_str(), "https://example.test/service-worker/resources/scope");
    }

    #[test]
    fn encoded_separators_are_rejected_case_insensitively() {
        for value in [
            "resources/scope%2fchild",
            "resources/scope%2Fchild",
            "resources/scope%5cchild",
        ] {
            assert_eq!(
                validate_service_worker_registration("resources/sw.js", Some(value), &document())
                    .unwrap_err()
                    .kind,
                ServiceWorkerRegistrationErrorKind::Type
            );
        }
    }

    #[test]
    fn scope_outside_script_directory_is_security_error() {
        let error = validate_service_worker_registration("resources/sw.js", Some("null"), &document()).unwrap_err();
        assert_eq!(error.kind, ServiceWorkerRegistrationErrorKind::Security);
    }
}
