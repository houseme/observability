use crate::{AppConfig, LogEntry, SerializableLevel, Sink};
use std::sync::Arc;
use tokio::sync::mpsc::{self, Receiver, Sender};

/// 服务端日志处理器
pub struct Logger {
    sender: Sender<LogEntry>, // 日志发送通道
}

impl Logger {
    /// 创建新的 Logger 实例
    /// 返回 Logger 和对应的 Receiver
    pub fn new(_config: &AppConfig) -> (Self, Receiver<LogEntry>) {
        let (sender, receiver) = mpsc::channel(10000); // 队列容量 10000
        (Logger { sender }, receiver)
    }

    /// 异步记录服务端日志
    /// 将日志附加到当前 Span，并生成独立的 Tracing Event
    #[tracing::instrument(skip(self), fields(log_source = "logger"))]
    pub async fn log(&self, entry: LogEntry) {
        // 将日志消息记录到当前 Span
        tracing::Span::current()
            .record("log_message", &entry.message)
            .record("source", &entry.source);

        // 生成独立的 Tracing Event，包含完整 LogEntry 信息
        // 根据 level 生成对应的事件
        match entry.level {
            SerializableLevel(tracing::Level::ERROR) => {
                tracing::error!(
                    target: "server_logs",
                    timestamp = %entry.timestamp,
                    message = %entry.message,
                    source = %entry.source,
                    request_id = ?entry.request_id,
                    user_id = ?entry.user_id,
                    fields = ?entry.fields
                );
            }
            SerializableLevel(tracing::Level::WARN) => {
                tracing::warn!(
                    target: "server_logs",
                    timestamp = %entry.timestamp,
                    message = %entry.message,
                    source = %entry.source,
                    request_id = ?entry.request_id,
                    user_id = ?entry.user_id,
                    fields = ?entry.fields
                );
            }
            SerializableLevel(tracing::Level::INFO) => {
                tracing::info!(
                    target: "server_logs",
                    timestamp = %entry.timestamp,
                    message = %entry.message,
                    source = %entry.source,
                    request_id = ?entry.request_id,
                    user_id = ?entry.user_id,
                    fields = ?entry.fields
                );
            }
            SerializableLevel(tracing::Level::DEBUG) => {
                tracing::debug!(
                    target: "server_logs",
                    timestamp = %entry.timestamp,
                    message = %entry.message,
                    source = %entry.source,
                    request_id = ?entry.request_id,
                    user_id = ?entry.user_id,
                    fields = ?entry.fields
                );
            }
            SerializableLevel(tracing::Level::TRACE) => {
                tracing::trace!(
                    target: "server_logs",
                    timestamp = %entry.timestamp,
                    message = %entry.message,
                    source = %entry.source,
                    request_id = ?entry.request_id,
                    user_id = ?entry.user_id,
                    fields = ?entry.fields
                );
            }
        }

        // 将日志发送到异步队列
        let _ = self.sender.send(entry).await;
    }
}

/// 启动日志模块
pub fn start_logger(config: &AppConfig, sinks: Vec<Arc<dyn Sink>>) -> Logger {
    let (logger, receiver) = Logger::new(config);
    tokio::spawn(crate::worker::start_worker(receiver, sinks));
    logger
}
