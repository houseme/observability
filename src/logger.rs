use crate::{AppConfig, LogEntry, SerializableLevel, Sink};
use std::sync::Arc;
use tokio::sync::mpsc::{self, Receiver, Sender};

/// 服务端日志处理器
pub struct Logger {
    sender: Sender<LogEntry>, // 日志发送通道
    queue_capacity: usize,
}

impl Logger {
    /// 创建新的 Logger 实例
    /// 返回 Logger 和对应的 Receiver
    pub fn new(config: &AppConfig) -> (Self, Receiver<LogEntry>) {
        // 从配置中获取队列容量，或使用默认值
        let queue_capacity = config.logger.queue_capacity.unwrap_or(10000);
        let (sender, receiver) = mpsc::channel(queue_capacity); // 队列容量 10000
        (
            Logger {
                sender,
                queue_capacity,
            },
            receiver,
        )
    }

    // 添加获取队列容量的方法
    pub fn queue_capacity(&self) -> usize {
        self.queue_capacity
    }

    /// 异步记录服务端日志
    /// 将日志附加到当前 Span，并生成独立的 Tracing Event
    #[tracing::instrument(skip(self), fields(log_source = "logger"))]
    pub async fn log(&self, entry: LogEntry) -> Result<(), LogError> {
        // 将日志消息记录到当前 Span
        tracing::Span::current()
            .record("log_message", &entry.message)
            .record("source", &entry.source);

        // 记录队列利用率（如果超过某个阈值）
        let queue_len = self.sender.capacity();
        let utilization = queue_len as f64 / self.queue_capacity as f64;
        if utilization > 0.8 {
            tracing::warn!("Log queue utilization high: {:.1}%", utilization * 100.0);
        }

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

        // 将日志发送到异步队列，改进错误处理
        match self.sender.try_send(entry) {
            Ok(_) => Ok(()),
            Err(mpsc::error::TrySendError::Full(entry)) => {
                // 队列满时的处理策略
                tracing::warn!("Log queue full, applying backpressure");
                match tokio::time::timeout(
                    std::time::Duration::from_millis(500),
                    self.sender.send(entry),
                )
                .await
                {
                    Ok(Ok(_)) => Ok(()),
                    Ok(Err(_)) => Err(LogError::SendFailed("Channel closed")),
                    Err(_) => Err(LogError::Timeout("Queue backpressure timeout")),
                }
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                Err(LogError::SendFailed("Logger channel closed"))
            }
        }
    }

    // 添加便捷方法，简化日志记录
    // 修复 info() 方法，将 None 替换为空向量，而不是 Option 类型
    pub async fn info(&self, message: &str, source: &str) -> Result<(), LogError> {
        self.log(LogEntry::new(
            tracing::Level::INFO,
            message.to_string(),
            source.to_string(),
            None,
            None,
            Vec::new(), // 使用空向量代替 None
        ))
        .await
    }

    /// 添加 warn() 方法
    pub async fn error(&self, message: &str, source: &str) -> Result<(), LogError> {
        self.log(LogEntry::new(
            tracing::Level::ERROR,
            message.to_string(),
            source.to_string(),
            None,
            None,
            Vec::new(),
        ))
        .await
    }

    /// 添加 warn() 方法
    pub async fn warn(&self, message: &str, source: &str) -> Result<(), LogError> {
        self.log(LogEntry::new(
            tracing::Level::WARN,
            message.to_string(),
            source.to_string(),
            None,
            None,
            Vec::new(),
        ))
        .await
    }

    /// 添加 debug() 方法
    pub async fn debug(&self, message: &str, source: &str) -> Result<(), LogError> {
        self.log(LogEntry::new(
            tracing::Level::DEBUG,
            message.to_string(),
            source.to_string(),
            None,
            None,
            Vec::new(),
        ))
        .await
    }

    /// 添加 trace() 方法
    pub async fn trace(&self, message: &str, source: &str) -> Result<(), LogError> {
        self.log(LogEntry::new(
            tracing::Level::TRACE,
            message.to_string(),
            source.to_string(),
            None,
            None,
            Vec::new(),
        ))
        .await
    }

    // 添加带有上下文信息的扩展方法，更加灵活
    pub async fn info_with_context(
        &self,
        message: &str,
        source: &str,
        request_id: Option<String>,
        user_id: Option<String>,
        fields: Vec<(String, String)>,
    ) -> Result<(), LogError> {
        self.log(LogEntry::new(
            tracing::Level::INFO,
            message.to_string(),
            source.to_string(),
            request_id,
            user_id,
            fields,
        ))
        .await
    }

    // 添加优雅关闭方法
    pub async fn shutdown(self) -> Result<(), LogError> {
        drop(self.sender); // 关闭发送端，让接收端知道不再有新消息
        Ok(())
    }
}

// 定义自定义错误类型
#[derive(Debug, thiserror::Error)]
pub enum LogError {
    #[error("Failed to send log: {0}")]
    SendFailed(&'static str),
    #[error("Operation timed out: {0}")]
    Timeout(&'static str),
}

/// 启动日志模块
pub fn start_logger(config: &AppConfig, sinks: Vec<Arc<dyn Sink>>) -> Logger {
    let (logger, receiver) = Logger::new(config);
    tokio::spawn(crate::worker::start_worker(receiver, sinks));
    logger
}
