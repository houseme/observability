use config::{Config, File, FileFormat};
use serde::Deserialize;

/// OpenTelemetry Configuration
#[derive(Debug, Deserialize, Clone)]
pub struct OtelConfig {
    pub endpoint: String,
    pub use_stdout: bool,
    pub sample_ratio: f64,
    pub meter_interval: u64,
    pub service_name: String,
    pub service_version: String,
    pub deployment_environment: String,
}

/// Kafka Sink Configuration - Add batch parameters
#[derive(Debug, Deserialize, Clone)]
pub struct KafkaSinkConfig {
    pub enabled: bool,
    pub bootstrap_servers: String,
    pub topic: String,
    pub batch_size: Option<usize>,     // Batch size, default 100
    pub batch_timeout_ms: Option<u64>, // Batch timeout time, default 1000ms
}

/// Webhook Sink Configuration - Add Retry Parameters
#[derive(Debug, Deserialize, Clone)]
pub struct WebhookSinkConfig {
    pub enabled: bool,
    pub url: String,
    pub max_retries: Option<usize>, // Maximum number of retry times, default 3
    pub retry_delay_ms: Option<u64>, // Retry the delay cardinality, default 100ms
}

/// File Sink Configuration - Add buffering parameters
#[derive(Debug, Deserialize, Clone)]
pub struct FileSinkConfig {
    pub enabled: bool,
    pub path: String,
    pub buffer_size: Option<usize>, // Write buffer size, default 8192
    pub flush_interval_ms: Option<u64>, // Refresh interval time, default 1000ms
    pub flush_threshold: Option<usize>, // Refresh threshold, default 100 logs
}

/// Sink configuration collection
#[derive(Debug, Deserialize, Clone)]
pub struct SinkConfig {
    pub kafka: KafkaSinkConfig,
    pub webhook: WebhookSinkConfig,
    pub file: FileSinkConfig,
}

///Logger Configuration
#[derive(Debug, Deserialize, Clone)]
pub struct LoggerConfig {
    pub queue_capacity: Option<usize>,
}

/// Overall application configuration
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub observability: OtelConfig,
    pub sinks: SinkConfig,
    pub logger: LoggerConfig,
}

/// Loading the configuration file
/// Supports TOML, YAML and .env formats, read in order by priority
pub fn load_config() -> AppConfig {
    let config = Config::builder()
        .add_source(File::with_name("config").format(FileFormat::Toml))
        .add_source(
            File::with_name("config")
                .format(FileFormat::Yaml)
                .required(false),
        )
        .add_source(config::Environment::with_prefix(""))
        .build()
        .unwrap();

    config.try_deserialize().unwrap()
}
