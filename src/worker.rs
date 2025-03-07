use crate::{entry::LogEntry, sink::Sink};
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;

/// 启动日志处理工作线程
pub async fn start_worker(receiver: Receiver<LogEntry>, sinks: Vec<Arc<dyn Sink>>) {
    let mut receiver = receiver;
    while let Some(entry) = receiver.recv().await {
        for sink in &sinks {
            sink.write(&entry).await;
        }
    }
}
