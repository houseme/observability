use crate::{AppConfig, LogEntry};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

/// Sink Trait 定义，异步写入日志
#[async_trait]
pub trait Sink: Send + Sync {
    async fn write(&self, entry: &LogEntry);
}

#[cfg(feature = "kafka")]
/// Kafka Sink 实现
pub struct KafkaSink {
    producer: rdkafka::producer::FutureProducer,
    topic: String,
}

#[cfg(feature = "kafka")]
#[async_trait]
impl Sink for KafkaSink {
    async fn write(&self, entry: &LogEntry) {
        let topic = self.topic.clone();
        let span_id = entry.timestamp.clone().to_rfc3339();
        let payload = serde_json::to_string(entry).unwrap();
        tokio::spawn({
            let topic = topic;
            let payload = payload;
            let span_id = span_id;
            let producer = self.producer.clone();
            async move {
                let _ = producer
                    .send(
                        rdkafka::producer::FutureRecord::to(&topic)
                            .payload(&payload)
                            .key(&span_id),
                        std::time::Duration::from_secs(0),
                    )
                    .await;
            }
        });
    }
}

#[cfg(feature = "webhook")]
/// Webhook Sink 实现
pub struct WebhookSink {
    url: String,
    client: reqwest::Client,
}

#[cfg(feature = "webhook")]
#[async_trait]
impl Sink for WebhookSink {
    async fn write(&self, entry: &LogEntry) {
        let _ = self.client.post(&self.url).json(entry).send().await;
    }
}

#[cfg(feature = "file")]
/// 文件 Sink 实现
pub struct FileSink {
    path: String,
}

#[cfg(feature = "file")]
#[async_trait]
impl Sink for FileSink {
    async fn write(&self, entry: &LogEntry) {
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.path)
            .await
            .unwrap();
        let line = format!("{:?}\n", entry);
        let _ = file.write_all(line.as_bytes()).await;
    }
}

/// 创建 Sink 实例列表
pub fn create_sinks(config: &AppConfig) -> Vec<Arc<dyn Sink>> {
    let mut sinks: Vec<Arc<dyn Sink>> = Vec::new();

    #[cfg(feature = "kafka")]
    if config.sinks.kafka.enabled {
        sinks.push(Arc::new(KafkaSink {
            producer: rdkafka::config::ClientConfig::new()
                .set("bootstrap.servers", &config.sinks.kafka.bootstrap_servers)
                .set("message.timeout.ms", "5000")
                .create()
                .unwrap(),
            topic: config.sinks.kafka.topic.clone(),
        }));
    }

    #[cfg(feature = "webhook")]
    if config.sinks.webhook.enabled {
        sinks.push(Arc::new(WebhookSink {
            url: config.sinks.webhook.url.clone(),
            client: reqwest::Client::new(),
        }));
    }

    #[cfg(feature = "file")]
    if config.sinks.file.enabled {
        sinks.push(Arc::new(FileSink {
            path: config.sinks.file.path.clone(),
        }));
    } else {
        sinks.push(Arc::new(FileSink {
            path: "default.log".to_string(),
        }));
    }

    sinks
}
