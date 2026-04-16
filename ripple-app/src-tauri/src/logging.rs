use std::path::PathBuf;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

pub fn init() {
    let log_dir = dirs();

    let file_appender = tracing_appender::rolling::daily(&log_dir, "ripple.log");
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);

    std::mem::forget(_guard);

    let file_layer = fmt::layer()
        .with_writer(file_writer)
        .with_ansi(false)
        .with_target(true)
        .with_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")));

    let console_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")));

    tracing_subscriber::registry()
        .with(file_layer)
        .with(console_layer)
        .init();

    tracing::info!("Ripple logging initialized, log dir: {}", log_dir.display());
}

pub fn dirs() -> PathBuf {
    let base = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join(".ripple").join("logs");
    let _ = std::fs::create_dir_all(&dir);
    dir
}
