use logging::{init_logging, load_config, LogEntry};
use opentelemetry::global;
use opentelemetry::trace::TraceContextExt;
use std::time::{Duration, SystemTime};
use tracing::{debug, info, instrument, Span};
use tracing_core::Level;
use tracing_opentelemetry::OpenTelemetrySpanExt;

#[tokio::main]
async fn main() {
    let start_time = SystemTime::now();
    let config = load_config();
    println!("配置文件加载完成 {:?}", config.clone());
    let (logger, _guard) = init_logging(config);
    info!("日志模块初始化完成");
    // Simulate the operation
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Record Metrics
    let meter = global::meter("rustfs.rs");
    let request_duration = meter.f64_histogram("s3_request_duration_seconds").build();
    request_duration.record(
        start_time.elapsed().unwrap().as_secs_f64(),
        &[opentelemetry::KeyValue::new("operation", "put_object")],
    );

    // Gets the current span
    let span = Span::current();
    // Use 'OpenTelemetrySpanExt' to get 'SpanContext'
    let span_context = span.context(); // Get context via OpenTelemetrySpanExt
    let span_id = span_context.span().span_context().span_id().to_string(); // Get the SpanId

    logger
        .log(LogEntry::new(
            Level::INFO,
            "处理用户请求".to_string(),
            "api_handler".to_string(),
            Some("req-12345".to_string()),
            Some("user-6789".to_string()),
            vec![
                ("endpoint".to_string(), "/api/v1/data".to_string()),
                ("method".to_string(), "GET".to_string()),
                ("span_id".to_string(), span_id),
            ],
        ))
        .await;
    put_object(
        "bucket".to_string(),
        "object".to_string(),
        "user".to_string(),
    )
    .await;
    info!("日志记录完成");
    tokio::time::sleep(Duration::from_secs(2)).await;
    info!("程序结束");
}

#[instrument(fields(bucket, object, user))]
async fn put_object(bucket: String, object: String, user: String) {
    let start_time = SystemTime::now();
    info!("Starting PUT operation");
    // Gets the current span
    let span = Span::current();
    // Use 'OpenTelemetrySpanExt' to get 'SpanContext'
    let span_context = span.context(); // Get context via OpenTelemetrySpanExt
    let span_id = span_context.span().span_context().span_id().to_string(); // Get the SpanId
    debug!(
        "Starting PUT operation content: bucket = {}, object = {}, user = {},span_id = {},start_time = {}",
        bucket,
        object,
        user,
        span_id,
        start_time.elapsed().unwrap().as_secs_f64()
    );
    // Simulate the operation
    tokio::time::sleep(Duration::from_millis(100)).await;

    info!("PUT operation completed");
}
