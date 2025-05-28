use std::sync::OnceLock;

use opentelemetry::global;
use opentelemetry_sdk::{propagation::TraceContextPropagator, trace::SdkTracerProvider};
use opentelemetry_stdout::SpanExporter;
use tracing_subscriber::EnvFilter;

static INIT_LOGGER: OnceLock<SdkTracerProvider> = OnceLock::new();

pub fn init_tracer() -> &'static SdkTracerProvider {
    INIT_LOGGER.get_or_init(|| {
        global::set_text_map_propagator(TraceContextPropagator::new());
        // Install stdout exporter pipeline to be able to retrieve the collected spans.
        // For the demonstration, use `Sampler::AlwaysOn` sampler to sample all traces.
        let provider = SdkTracerProvider::builder()
            .with_simple_exporter(SpanExporter::default())
            .build();

        global::set_tracer_provider(provider.clone());

        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
            )
            .init();

        provider
    })
}
