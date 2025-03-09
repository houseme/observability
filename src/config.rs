use config::{Config, File, FileFormat};
use serde::Deserialize;

/// OpenTelemetry 配置
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

/// Kafka Sink 配置
#[derive(Debug, Deserialize, Clone)]
pub struct KafkaSinkConfig {
    pub enabled: bool,
    pub bootstrap_servers: String,
    pub topic: String,
}

/// Webhook Sink 配置
#[derive(Debug, Deserialize, Clone)]
pub struct WebhookSinkConfig {
    pub enabled: bool,
    pub url: String,
}

/// 文件 Sink 配置
#[derive(Debug, Deserialize, Clone)]
pub struct FileSinkConfig {
    pub enabled: bool,
    pub path: String,
}

/// Sink 配置集合
#[derive(Debug, Deserialize, Clone)]
pub struct SinkConfig {
    pub kafka: KafkaSinkConfig,
    pub webhook: WebhookSinkConfig,
    pub file: FileSinkConfig,
}

/// 应用总体配置
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub opentelemetry: OtelConfig,
    pub sinks: SinkConfig,
}

/// 加载配置文件
/// 支持 TOML、YAML 和 .env 格式，按优先级依次读取
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
