# Observability

Metrics, links, and loggers based on `opentelemetry`.

## Overview

`observability` is a simple observability library for Rust applications, providing metrics, tracing, and logging
capabilities.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
observability = { version = "0.1.0", git = "https://github.com/houseme/observability.git", branch = "main" }
```

## Usage

```rust
use observability::{init_logging, load_config};
use tracing::info;

#[tokio::main]
async fn main() {
    let config = load_config();
    let (logger, _guard) = init_logging(config);
    info!("Log module initialization is completed");
    // Your application code here
}
```

## Metrics

- [Prometheus](https://prometheus.io/)
- [Grafana](https://grafana.com/)
- [Jaeger](https://www.jaegertracing.io/)
- [Zipkin](https://zipkin.io/)
- [OpenTelemetry](https://opentelemetry.io/)
- [OpenTracing](https://opentracing.io/)
- [OpenCensus](https://opencensus.io/)
- [OpenMetrics](https://openmetrics.io/)

## Links

- [OpenTelemetry](https://opentelemetry.io/)
- [OpenTracing](https://opentracing.io/)
- [OpenCensus](https://opencensus.io/)

## Loggers

- [Loki]

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## License

This project is licensed under the MIT OR Apache-2.0 license.
