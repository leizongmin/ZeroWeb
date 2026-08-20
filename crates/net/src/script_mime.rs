//! JavaScript MIME type classification shared by script fetch consumers.

/// Return whether a MIME essence is a JavaScript MIME type.
pub fn is_javascript_mime(mime: &str) -> bool {
    matches!(
        mime.to_ascii_lowercase().as_str(),
        "application/ecmascript"
            | "application/javascript"
            | "application/x-ecmascript"
            | "application/x-javascript"
            | "text/ecmascript"
            | "text/javascript"
            | "text/javascript1.0"
            | "text/javascript1.1"
            | "text/javascript1.2"
            | "text/javascript1.3"
            | "text/javascript1.4"
            | "text/javascript1.5"
            | "text/jscript"
            | "text/livescript"
            | "text/x-ecmascript"
            | "text/x-javascript"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_standard_and_legacy_javascript_mime_types() {
        assert!(is_javascript_mime("text/javascript"));
        assert!(is_javascript_mime("Application/JavaScript"));
        assert!(is_javascript_mime("text/javascript1.5"));
        assert!(is_javascript_mime("text/livescript"));
        assert!(!is_javascript_mime("text/plain"));
        assert!(!is_javascript_mime(""));
    }
}
