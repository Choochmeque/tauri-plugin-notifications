// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{ser::Serializer, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[cfg(mobile)]
    #[error(transparent)]
    PluginInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }

    #[test]
    fn test_io_error_serialization() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let err = Error::Io(io_err);
        let json = serde_json::to_string(&err).expect("Failed to serialize IO error");
        assert!(json.contains("access denied"));
    }

    #[test]
    fn test_io_error_display() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "test error");
        let err = Error::Io(io_err);
        let display_str = format!("{}", err);
        assert!(display_str.contains("test error"));
    }

    #[cfg(mobile)]
    #[test]
    fn test_plugin_invoke_error_conversion() {
        use tauri::plugin::mobile::PluginInvokeError;

        let plugin_err = PluginInvokeError::MethodNotFound("test_method".to_string());
        let err: Error = plugin_err.into();
        assert!(matches!(err, Error::PluginInvoke(_)));
    }

    #[test]
    fn test_result_type_err() {
        let io_err = io::Error::other("test");
        let result: Result<i32> = Err(Error::Io(io_err));
        assert!(result.is_err());
    }
}
