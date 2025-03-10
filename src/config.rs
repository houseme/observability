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

/// Kafka Sink 配置 - 增加批处理参数
#[derive(Debug, Deserialize, Clone)]
pub struct KafkaSinkConfig {
    pub enabled: bool,
    pub bootstrap_servers: String,
    pub topic: String,
    pub batch_size: Option<usize>,     // 批处理大小，默认 100
    pub batch_timeout_ms: Option<u64>, // 批处理超时时间，默认 1000ms
}

/// Webhook Sink 配置 - 增加重试参数
#[derive(Debug, Deserialize, Clone)]
pub struct WebhookSinkConfig {
    pub enabled: bool,
    pub url: String,
    pub max_retries: Option<usize>,  // 最大重试次数，默认 3
    pub retry_delay_ms: Option<u64>, // 重试延迟基数，默认 100ms
}

/// 文件 Sink 配置 - 增加缓冲参数
#[derive(Debug, Deserialize, Clone)]
pub struct FileSinkConfig {
    pub enabled: bool,
    pub path: String,
    pub buffer_size: Option<usize>,     // 写缓冲区大小，默认 8192
    pub flush_interval_ms: Option<u64>, // 刷新间隔时间，默认 1000ms
    pub flush_threshold: Option<usize>, // 刷新阈值，默认 100 条日志
}

/// Sink 配置集合
#[derive(Debug, Deserialize, Clone)]
pub struct SinkConfig {
    pub kafka: KafkaSinkConfig,
    pub webhook: WebhookSinkConfig,
    pub file: FileSinkConfig,
}

// 在 config.rs 中增加
#[derive(Debug, Deserialize, Clone)]
pub struct LoggerConfig {
    pub queue_capacity: Option<usize>,
}

/// 应用总体配置
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub opentelemetry: OtelConfig,
    pub sinks: SinkConfig,
    pub logger: LoggerConfig,
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
