mod config;
mod entry;
mod logger;
mod sink;
mod telemetry;
mod worker;

use crate::logger::start_logger;
pub use config::load_config;
pub use config::AppConfig;
pub use entry::{LogEntry, SerializableLevel};
pub use logger::Logger;
pub use sink::Sink;
pub use telemetry::init_telemetry;
pub use worker::start_worker;

/// 日志模块初始化函数
/// 返回 Logger 和清理守卫
pub fn init_logging(config: AppConfig) -> (Logger, telemetry::OtelGuard) {
    let guard = init_telemetry(&config.opentelemetry);
    // let (logger, receiver) = Logger::new(&config);
    let sinks = sink::create_sinks(&config);
    // tokio::spawn(start_worker(receiver, sinks.clone()));
    let logger = start_logger(&config, sinks);
    (logger, guard)
}
