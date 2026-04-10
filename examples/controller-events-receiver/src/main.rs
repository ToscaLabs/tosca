use std::time::Duration;

use tosca_controller::controller::Controller;
use tosca_controller::discovery::{Discovery, TransportProtocol};
use tosca_controller::error::Error;

use clap::Parser;

use tracing::{error, info};
use tracing_subscriber::filter::LevelFilter;

#[derive(Parser)]
#[command(
    version,
    about,
    long_about = "A controller that scans the network for `tosca` devices and \
                  subscribes to their brokers to receive events."
)]
struct Cli {
    /// Service domain (defaults to "tosca").
    #[arg(short = 'd', long = "domain", default_value = "tosca")]
    service_domain: String,
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
    controller.discover().await?;

    // Get devices.
    let devices = controller.devices();

    info!("Number of discovered devices: {}", devices.len());

    // TODO: Ensure the process continues even if no devices are found.
    // Implement an asynchronous task to run device discovery
    // every `n` milliseconds, updating the controller state when new devices
    // are detected or existing devices are removed.
    if devices.is_empty() {
        info!("No devices discovered. Terminating the process without any errors.");
        return Ok(());
    }

    let mut receiver = controller
        .start_event_receivers(100)
        .await
        .expect("failed to retrieve the global events receiver");

    loop {
        tokio::select! {
            result = tokio::signal::ctrl_c() => {
                if let Err(e) = result {
                    error!("Error while stopping the process: {e}");
                }
                break;
            }
            event = receiver.recv() => {
                if let Some(event) = event {
                     info!("{event}");
                } else {
                    error!("No more events received, stopping the process");
                    break;
                }
            }
        }
    }

    controller.shutdown().await;

    Ok(())
}
