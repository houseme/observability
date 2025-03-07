use crate::config::OtelConfig;
use opentelemetry::trace::TracerProvider;
use opentelemetry::{global, KeyValue};
use opentelemetry_appender_tracing::layer;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::{
    metrics::{MeterProviderBuilder, PeriodicReader, SdkMeterProvider},
    trace::{RandomIdGenerator, Sampler, SdkTracerProvider},
    Resource,
};
use opentelemetry_semantic_conventions::{
    attribute::{
        DEPLOYMENT_ENVIRONMENT_NAME, NETWORK_LOCAL_ADDRESS, SERVICE_NAME, SERVICE_VERSION,
    },
    SCHEMA_URL,
};
use tracing_opentelemetry::{MetricsLayer, OpenTelemetryLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// OpenTelemetry 清理守卫
pub struct OtelGuard {
    tracer_provider: SdkTracerProvider,
    meter_provider: SdkMeterProvider,
}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        if let Err(err) = self.tracer_provider.shutdown() {
            eprintln!("Tracer shutdown error: {:?}", err);
        }
        if let Err(err) = self.meter_provider.shutdown() {
            eprintln!("Meter shutdown error: {:?}", err);
        }
    }
}

/// 创建 OpenTelemetry Resource
fn resource(config: &OtelConfig) -> Resource {
    Resource::builder()
        .with_service_name(config.service_name.clone())
        .with_schema_url(
            [
                KeyValue::new(SERVICE_NAME, config.service_name.clone()),
                KeyValue::new(SERVICE_VERSION, config.service_version.clone()),
                KeyValue::new(
                    DEPLOYMENT_ENVIRONMENT_NAME,
                    config.deployment_environment.clone(),
                ),
                KeyValue::new(NETWORK_LOCAL_ADDRESS, "127.0.0.1"),
            ],
            SCHEMA_URL,
        )
        .build()
}

/// 初始化 Meter Provider
fn init_meter_provider(config: &OtelConfig) -> SdkMeterProvider {
    let mut builder = MeterProviderBuilder::default().with_resource(resource(config));
    if config.meter_enabled && config.use_stdout {
        let exporter = opentelemetry_otlp::MetricExporter::builder()
            .with_tonic()
            .with_endpoint(&config.endpoint)
            .with_temporality(opentelemetry_sdk::metrics::Temporality::default())
            .build()
            .unwrap();
        builder = builder
            .with_reader(
                PeriodicReader::builder(exporter)
                    .with_interval(std::time::Duration::from_secs(config.meter_interval))
                    .build(),
            )
            .with_reader(
                PeriodicReader::builder(opentelemetry_stdout::MetricExporter::default())
                    .with_interval(std::time::Duration::from_secs(config.meter_interval))
                    .build(),
            );
        println!("Meter enabled with stdout");
    } else if config.meter_enabled {
        let exporter = opentelemetry_otlp::MetricExporter::builder()
            .with_tonic()
            .with_endpoint(&config.endpoint)
            .with_temporality(opentelemetry_sdk::metrics::Temporality::default())
            .build()
            .unwrap();
        builder = builder.with_reader(
            PeriodicReader::builder(exporter)
                .with_interval(std::time::Duration::from_secs(config.meter_interval))
                .build(),
        );
    } else if config.use_stdout {
        builder = builder.with_reader(
            PeriodicReader::builder(opentelemetry_stdout::MetricExporter::default())
                .with_interval(std::time::Duration::from_secs(config.meter_interval))
                .build(),
        );
    } else {
        println!("Meter disabled");
    }

    let meter_provider = builder.build();
    global::set_meter_provider(meter_provider.clone());
    meter_provider
}

/// 初始化 Tracer Provider
fn init_tracer_provider(config: &OtelConfig) -> SdkTracerProvider {
    let sampler = if config.sample_ratio > 0.0 && config.sample_ratio < 1.0 {
        Sampler::TraceIdRatioBased(config.sample_ratio)
    } else {
        Sampler::AlwaysOn
    };
    let builder = SdkTracerProvider::builder()
        .with_sampler(sampler)
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(resource(config));

    let tracer_provider = if config.tracer_enabled && config.use_stdout {
        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(&config.endpoint)
            .build()
            .unwrap();
        println!("Tracer enabled with stdout");
        builder
            .with_batch_exporter(exporter)
            .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
            .build()
    } else if config.tracer_enabled {
        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(&config.endpoint)
            .build()
            .unwrap();
        builder.with_batch_exporter(exporter).build()
    } else if config.use_stdout {
        builder
            .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
            .build()
    } else {
        builder.build()
    };

    global::set_tracer_provider(tracer_provider.clone());
    tracer_provider
}

/// 初始化 Telemetry
pub fn init_telemetry(config: &OtelConfig) -> OtelGuard {
    let tracer_provider = init_tracer_provider(config);
    let meter_provider = init_meter_provider(config);
    let tracer = tracer_provider.tracer("logger");
    let registry =
        tracing_subscriber::registry().with(tracing_subscriber::filter::LevelFilter::INFO);

    let exporter = opentelemetry_stdout::LogExporter::default();
    let provider: SdkLoggerProvider = SdkLoggerProvider::builder()
        .with_simple_exporter(exporter)
        .with_resource(resource(config))
        .build();
    let layer = layer::OpenTelemetryTracingBridge::new(&provider);

    if config.logs_enabled && config.meter_enabled {
        registry
            .with(tracing_subscriber::fmt::layer().with_ansi(false))
            .with(OpenTelemetryLayer::new(tracer))
            .with(MetricsLayer::new(meter_provider.clone()))
            .with(layer)
            .init();
        println!("Logs and meter,tracer enabled");
    } else if config.logs_enabled {
        registry
            .with(tracing_subscriber::fmt::layer().with_ansi(false))
            .with(OpenTelemetryLayer::new(tracer))
            .init();
    } else if config.meter_enabled {
        registry
            .with(tracing_subscriber::fmt::layer().with_ansi(false))
            .with(MetricsLayer::new(meter_provider.clone()))
            .init();
    } else {
        registry
            .with(tracing_subscriber::fmt::layer().with_ansi(true))
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    OtelGuard {
        tracer_provider,
        meter_provider,
    }
}
