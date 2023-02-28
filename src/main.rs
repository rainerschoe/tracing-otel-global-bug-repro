use std::net::ToSocketAddrs;
use opentelemetry::trace::TracerProvider;
use tracing::instrument;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{prelude::*, EnvFilter};
use tokio::time::{sleep, Duration};


/// This is a minimum example to reproduce a presumed buf in one of the
/// opentelemetry crates.
/// Can be tested against jaegertracing/all-in-one docker container:
/// 1. Run docker container
///     docker run -d --name jaeger \     
///                                   -e COLLECTOR_ZIPKIN_HTTP_PORT=9411 \
///                                   -p 5775:5775/udp \
///                                   -p 6831:6831/udp \
///                                   -p 6832:6832/udp \
///                                   -p 5778:5778 \
///                                   -p 16686:16686 \
///                                   -p 14268:14268 \
///                                   -p 9411:9411 \
///                                   jaegertracing/all-in-one:1.6
/// 2. Modify this code to have correct Jaeger Agent addresses for your host
/// 3. Execute this program
/// 4. Look at http://localhost:16686/search
///     (or just watch network traffic on udp port 6831 with wireshark)
/// 5. Observe no spans arriving :(
/// 6. Modify this code to include the line
///     let _ = global::set_tracer_provider(tracer_provider);
///    in the FIXME section below.
/// 7. Repeat the experiment and see: now traces arrive
#[tokio::main]
async fn main() {
    // add an opentelemetry layer to tracing:
    setup_instrumentation("192.168.0.162".into(), 6831);

    // call an instrumented function
    foo();

    // need to wait some time to give batch processor time to export:
    sleep(Duration::from_millis(8000)).await;
}

#[instrument(level = "info", skip_all)]
fn foo()
{
    println!("foo!");
}

pub fn setup_instrumentation(
    jaeger_agent_host: String,
    jaeger_agent_port: u16,
){
    // NOTE: I am not directly using the build_simple() builder, as I need/want
    // to integrate a custom span processor, which is not possible otherwise.

    // build an exporter which is responsible for sending batches to a jaeger agent:
    let jaeger_agent_address = (jaeger_agent_host.clone(), jaeger_agent_port)
        .to_socket_addrs().unwrap()
        .next()
        .ok_or(format!(
            "Could not resolve jaeger agent address {}:{}",
            jaeger_agent_host, jaeger_agent_port
        ));
    let exporter = opentelemetry_jaeger::config::agent::new_agent_pipeline()
        .with_endpoint(jaeger_agent_address.unwrap())
        .with_service_name("global-test")
        .with_auto_split_batch(true)
        .build_async_agent_exporter(opentelemetry::runtime::Tokio).unwrap();

    // Batch span processor:
    // Actually I want to use custon span processor here, but for simplifying
    // the bug report, I use Batch Processor:
    let span_processor = opentelemetry::sdk::trace::BatchSpanProcessor::builder(exporter, opentelemetry::runtime::Tokio).build();

    // The tracer provider is used to create tracers with different names
    // (e.g. for different tracer names for different libs)
    let tracer_provider = opentelemetry::sdk::trace::TracerProvider::builder()
        .with_span_processor(span_processor)
        .build();
    // However we just use one tracer for everything:
    let tracer = tracer_provider.tracer("tracing-opentelemetry-jaeger");

    // FIXME:
    // Without this line no spans are sent to the agent.
    // traging-opentelemetry should only use the tracer passed in OpenTelemetryLayer. However
    // this seems not to be the case.
    // I am beginning to suspect a bug in one of the OTEL libs here...
    // let _ = global::set_tracer_provider(tracer_provider);

    let opentelemetry_layer = Some(OpenTelemetryLayer::new(tracer));

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(opentelemetry_layer)
        .init();
}
