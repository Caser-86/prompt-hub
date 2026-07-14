use std::fmt;

use thiserror::Error;
use url::Url;

#[derive(Default)]
pub struct Redactor {
    _private: (),
}

impl Redactor {
    pub fn url(&self, value: &str) -> Result<String, RedactionError> {
        let mut url = Url::parse(value)?;
        let _ = url.set_username("");
        let _ = url.set_password(None);
        if url.query().is_some() {
            let keys = url
                .query_pairs()
                .map(|(key, _)| key.into_owned())
                .collect::<Vec<_>>();
            url.query_pairs_mut()
                .clear()
                .extend_pairs(keys.into_iter().map(|key| (key, "[REDACTED]".to_owned())));
        }
        Ok(url.into())
    }

    #[must_use]
    pub fn header(&self, name: &str, _value: &str) -> String {
        format!("{name}: [REDACTED]")
    }
}

pub struct RedactedBody {
    _value: String,
}

impl RedactedBody {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            _value: value.into(),
        }
    }
}

impl fmt::Display for RedactedBody {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("[REDACTED PROMPT BODY]")
    }
}

impl fmt::Debug for RedactedBody {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RedactedBody([REDACTED])")
    }
}

#[derive(Debug, Error)]
pub enum RedactionError {
    #[error("URL could not be parsed for redaction: {0}")]
    Url(#[from] url::ParseError),
}
