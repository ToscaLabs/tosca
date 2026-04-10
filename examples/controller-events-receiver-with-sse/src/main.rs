use std::collections::HashMap;
use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use askama::Template;
use axum::{
    Router,
    extract::{Path, State},
    response::sse::{Event, KeepAlive, Sse},
    response::{ErrorResponse, Html, IntoResponse},
    routing::get,
};

use clap::Parser;

use futures::stream::Stream;

use tokio::signal;
use tokio::sync::broadcast::Receiver;

use tokio_stream::StreamExt as _;
use tokio_stream::wrappers::BroadcastStream;

use tower_http::services::ServeDir;

use tracing::{error, info};
use tracing_subscriber::filter::LevelFilter;

use tosca::events::Events;

use tosca_controller::controller::Controller;
use tosca_controller::discovery::{Discovery, TransportProtocol};

const THROTTLE: Duration = Duration::from_secs(1);
const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Parser)]
#[command(
    version,
    about,
    long_about = "A controller that scans the network for `tosca` devices, \
                  subscribes to their brokers to receive events, and \
                  and displays their data in real-time on a web page."
)]
struct Cli {
    /// Controller `IPv4` address (defaults to "localhost" address).
    ///
    /// Only `IPv4` addresses are accepted.
    #[arg(long, default_value_t = Ipv4Addr::LOCALHOST)]
    ip: Ipv4Addr,
    /// Web controller port (defaults to port 8123).
    #[arg(long, default_value_t = 8123)]
    port: u16,
    /// Service domain (defaults to "tosca").
    #[arg(short = 'd', long = "domain", default_value = "tosca")]
    service_domain: String,
}

#[derive(Clone, Template)]
#[template(path = "index.html")]
struct DevicesConsoles {
    devices_ids: Vec<String>,
}

impl DevicesConsoles {
    const fn new(devices_ids: Vec<String>) -> Self {
        Self { devices_ids }
    }
}

#[derive(Clone)]
struct AppState {
    devices_consoles: DevicesConsoles,
    devices_receivers: Arc<HashMap<usize, Receiver<Events>>>,
}

impl AppState {
    fn new(devices_receivers: HashMap<usize, Receiver<Events>>) -> Self {
        let devices_consoles = DevicesConsoles::new(
            devices_receivers
                .keys()
                .map(std::string::ToString::to_string)
                .collect(),
        );
        Self {
            devices_consoles,
            devices_receivers: Arc::new(devices_receivers),
        }
    }
}

#[inline]
async fn index(State(state): State<AppState>) -> impl IntoResponse {
    let rendered_data = match state.devices_consoles.render() {
        Ok(template_rendered) => template_rendered,
        Err(e) => format!("<html><body>Something went wrong: {e}</body></html>"),
    };

    Html(rendered_data)
}

async fn event_stream(
    Path(device_id): Path<usize>,
    State(state): State<AppState>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ErrorResponse> {
    let receiver = state.devices_receivers.get(&device_id).ok_or_else(|| {
        let err = format!("Device `{device_id}` does not exist");
        error!(err);
        ErrorResponse::from(err)
    })?;

    let receiver = receiver.resubscribe();

    let stream = BroadcastStream::new(receiver);

    // Convert the stream into SSE events
    let sse_stream = stream
        .filter_map(move |events| {
            let events = match events {
                Ok(events) => events,
                Err(e) => {
                    error!("Failed to receive the events: {e}");
                    return None;
                }
            };

            info!("{events}");

            Some(Ok(Event::default()
                .id(device_id.to_string())
                .data(format!("{events}"))))
        })
        .throttle(THROTTLE);

    Ok(Sse::new(sse_stream).keep_alive(KeepAlive::default().interval(KEEPALIVE_INTERVAL)))
}

#[inline]
async fn shutdown_controller() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("failed to run a `tosca-controller` method")]
    Tosca(#[source] tosca_controller::error::Error),
    #[error("failed to bind to the socket")]
    Bind(#[source] std::io::Error),
    #[error("failed to run the server")]
    Run(#[source] std::io::Error),
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing subscriber.
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .init();

    let cli = Cli::parse();

    let discovery = Discovery::new(cli.service_domain)
        .timeout(Duration::from_secs(2))
        // TODO: Implement a "TCP-UDP" strategy to retrieve both UDP and TCP
        // mDNS-SD sockets that may coexist within the same environment.
        .transport_protocol(TransportProtocol::UDP)
        .disable_ipv6()
        .disable_network_interface("docker0");

    // Create a controller.
    let mut controller = Controller::new(discovery);

    // Run discovery process.
    controller.discover().await.map_err(Error::Tosca)?;

    let devices = controller.devices_mut();

    info!("Number of discovered devices: {}", devices.len());

    // TODO: Ensure the process continues even if no devices are found.
    // Add a button in the web application to trigger device discovery and
    // update the controller state.
    if devices.is_empty() {
        info!("No devices discovered. Terminating the process without any errors.");
        return Ok(());
    }

    let mut devices_receivers = HashMap::new();
    // FIXME: Using usize is an hack because IDs have not implemented yet.
    for (id, device) in devices.iter_mut().enumerate() {
        let receiver = device
            .start_event_receiver(id, 100)
            .await
            .map_err(Error::Tosca)?;
        devices_receivers.insert(id, receiver);
    }

    let state = AppState::new(devices_receivers);
    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let static_files_service = ServeDir::new(assets_dir).append_index_html_on_directories(true);
    let app = Router::new()
        .fallback_service(static_files_service)
        .route("/", get(index))
        .route("/events/{device_id}", get(event_stream))
        .with_state(state);

    // Creates the web controller listener bind.
    let listener_bind = SocketAddr::new(IpAddr::V4(cli.ip), cli.port);

    // Creates listener.
    let listener = tokio::net::TcpListener::bind(&listener_bind)
        .await
        .map_err(Error::Bind)?;

    info!("Start running server on address: {}:{}", cli.ip, cli.port);

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_controller())
        .await
        .map_err(Error::Run)?;

    controller.shutdown().await;

    Ok(())
}
