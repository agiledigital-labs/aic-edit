//! IDM script compile pre-flight. The endpoint is parse-only and IDM-only; AM
//! scripts have no equivalent compile action. See `docs/api/11-idm-endpoints.md`.

use crate::{Error, Result};
use serde_json::json;

/// Result of IDM compile validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Validation {
    Ok,
    Invalid(String),
}

/// Validate plaintext IDM source with `/openidm/script?_action=compile`.
pub async fn validate_idm_source(tenant: &str, source: &[u8]) -> Result<Validation> {
    let src = std::str::from_utf8(source)
        .map_err(|e| Error::Config(format!("IDM script source is not UTF-8: {e}")))?;
    let body = json!({
        "type": "text/javascript",
        "source": src,
    });

    match crate::aic::api::post(tenant, "/openidm/script?_action=compile", body, true).await {
        Ok(_) => Ok(Validation::Ok),
        Err(Error::Api { status: 400, body }) => Ok(Validation::Invalid(message_from_body(&body))),
        Err(e) => Err(e),
    }
}

fn message_from_body(body: &str) -> String {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| {
            v.get("message")
                .and_then(|m| m.as_str())
                .map(ToOwned::to_owned)
        })
        .unwrap_or_else(|| body.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_message_from_compile_error_body() {
        let body =
            r#"{"code":400,"reason":"Bad Request","message":"missing ) after formal parameters"}"#;

        assert_eq!(message_from_body(body), "missing ) after formal parameters");
    }

    #[test]
    fn falls_back_to_raw_body_when_shape_is_unexpected() {
        assert_eq!(message_from_body("not json"), "not json");
    }
}
