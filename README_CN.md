# 可观测性

基于`opentelemetry`的指标、链接和日志记录器。

## 概述

`observability`是一个简单的 Rust 应用程序可观测性库，提供指标、追踪和日志记录功能。

## 安装

在您的`Cargo.toml`中添加以下内容：

```toml
[dependencies]
observability = { version = "0.1.0", git = "https://github.com/houseme/observability.git", branch = "main" }
```

## 使用

```rust
use observability::{init_logging, load_config};
use tracing::info;

#[tokio::main]
async fn main() {
    let config = load_config();
    let (logger, _guard) = init_logging(config);
    info!("日志模块初始化完成");
    // 您的应用程序代码
}
```

## 指标

- [Prometheus](https://prometheus.io/)
- [Grafana](https://grafana.com/)
- [Jaeger](https://www.jaegertracing.io/)
- [Zipkin](https://zipkin.io/)
- [OpenTelemetry](https://opentelemetry.io/)
- [OpenTracing](https://opentracing.io/)
- [OpenCensus](https://opencensus.io/)
- [OpenMetrics](https://openmetrics.io/)

## 链接

- [OpenTelemetry](https://opentelemetry.io/)
- [OpenTracing](https://opentracing.io/)
- [OpenCensus](https://opencensus.io/)

## 日志记录器

- [Loki]

## 贡献

欢迎贡献！请打开一个 issue 或提交一个 pull request。

## 许可证

本项目使用 MIT 或 Apache-2.0 许可证。