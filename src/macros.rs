//! Custom macros for the project

/// Custom environment variable macro
macro_rules! custom_env {
    ($($arg:tt)*) => {
        concat!("MUSIC_EXPORTER_", $($arg)*)
    };
}

pub(crate) use custom_env;
