//! music-exporter errors

use serde_json::Value;
use std::{
    fmt, io,
    sync::{Arc, MutexGuard, PoisonError},
};
use url::ParseError;

use crate::oauth::ReceivedCode;

/// Galion error wrapper
#[derive(Debug)]
pub struct MusicExporterError {
    /// error message
    pub message: String,
    /// source error
    source: Option<Arc<dyn std::error::Error + Send + Sync>>,
}

impl std::error::Error for MusicExporterError {}

impl Clone for MusicExporterError {
    fn clone(&self) -> Self {
        Self {
            message: self.message.clone(),
            source: self.source.clone(),
        }
    }
}
impl fmt::Display for MusicExporterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.source {
            Some(src) => write!(f, "{} - caused by: {}", self.message, src),
            None => write!(f, "{}", self.message),
        }
    }
}

impl MusicExporterError {
    /// Create new AppError
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        let ref_str = s.as_ref();
        let message = ref_str.to_string();
        Self {
            message,
            source: None,
        }
    }
}

impl From<&str> for MusicExporterError {
    fn from(message: &str) -> Self {
        Self::new(message)
    }
}

impl From<String> for MusicExporterError {
    fn from(message: String) -> Self {
        Self::new(message)
    }
}

impl From<io::Error> for MusicExporterError {
    fn from(error: io::Error) -> Self {
        Self {
            message: error.to_string(),
            source: Some(Arc::new(error)),
        }
    }
}

impl From<std::num::ParseIntError> for MusicExporterError {
    fn from(error: std::num::ParseIntError) -> Self {
        Self {
            message: error.to_string(),
            source: Some(Arc::new(error)),
        }
    }
}

impl From<ParseError> for MusicExporterError {
    fn from(error: ParseError) -> Self {
        Self {
            message: error.to_string(),
            source: Some(Arc::new(error)),
        }
    }
}

impl From<reqwest::Error> for MusicExporterError {
    fn from(error: reqwest::Error) -> Self {
        Self {
            message: error.to_string(),
            source: Some(Arc::new(error)),
        }
    }
}
impl From<hyper::http::Error> for MusicExporterError {
    fn from(error: hyper::http::Error) -> Self {
        Self {
            message: error.to_string(),
            source: Some(Arc::new(error)),
        }
    }
}

impl
    From<
        PoisonError<
            std::sync::MutexGuard<
                '_,
                std::option::Option<tokio::sync::oneshot::Sender<ReceivedCode>>,
            >,
        >,
    > for MusicExporterError
{
    fn from(
        error: PoisonError<MutexGuard<'_, Option<tokio::sync::oneshot::Sender<ReceivedCode>>>>,
    ) -> Self {
        Self {
            message: error.to_string(),
            source: None,
        }
    }
}

impl From<serde_json::Error> for MusicExporterError {
    fn from(error: serde_json::Error) -> Self {
        Self {
            message: error.to_string(),
            source: Some(Arc::new(error)),
        }
    }
}

impl From<Value> for MusicExporterError {
    fn from(value: Value) -> Self {
        match value.get("error") {
            Some(Value::String(error_message)) => Self::new(error_message.clone()),
            _ => Self::new(value.to_string()),
        }
    }
}
