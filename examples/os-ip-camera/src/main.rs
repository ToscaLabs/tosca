mod info;
mod parameters;
mod screenshot;
mod stream;

use std::net::Ipv4Addr;
use std::sync::Arc;

use tosca::hazards::Hazard;
use tosca::parameters::Parameters;
use tosca::route::Route;

use tosca_os::device::Device;
use tosca_os::responses::error::ErrorResponse;
use tosca_os::responses::ok::ok_stateful;
use tosca_os::responses::serial::{serial_stateful, serial_stateless};
use tosca_os::responses::stream::stream_stateful;
use tosca_os::server::Server;
use tosca_os::service::{ServiceConfig, TransportProtocol};

use clap::Parser;
use clap::builder::ValueParser;

use nokhwa::{
    NokhwaError, native_api_backend, query,
    utils::{CameraFormat, CameraIndex, FrameFormat, RequestedFormatType, Resolution},
};

use tokio::sync::Mutex;

use tracing::{error, info};
use tracing_subscriber::filter::LevelFilter;

use crate::info::{show_available_cameras, show_camera_info};
use crate::parameters::{
    change_camera, format_absolute_framerate, format_absolute_resolution, format_closest,
    format_exact, format_highest_framerate, format_highest_resolution, format_random,
};
use crate::screenshot::{
    screenshot_absolute_framerate, screenshot_absolute_resolution, screenshot_closest,
    screenshot_exact, screenshot_highest_framerate, screenshot_highest_resolution,
    screenshot_random,
};
use crate::stream::show_camera_stream;

fn camera_error(description: &'static str, error: impl std::error::Error) -> ErrorResponse {
    ErrorResponse::internal_with_error(description, &error.to_string())
}

fn thread_error<T: std::fmt::Display>(msg: &str, e: T) {
    error!("{msg}");
    error!("{e}");
}

#[derive(Clone)]
struct CameraConfig {
    index: CameraIndex,
    format_type: RequestedFormatType,
}

#[derive(Clone)]
struct InternalState {
    camera: Arc<Mutex<CameraConfig>>,
}

impl InternalState {
    fn new(camera: CameraConfig) -> Self {
        Self {
            camera: Arc::new(Mutex::new(camera)),
        }
    }
}

fn parse_transport_protocol(protocol: &str) -> Result<TransportProtocol, std::io::Error> {
    match protocol {
        "tcp" | "TCP" => Ok(TransportProtocol::TCP),
        "udp" | "UDP" => Ok(TransportProtocol::UDP),
        _ => Err(std::io::Error::other(format!(
            "{protocol:?} is not a supported protocol."
        ))),
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Server address.
    ///
    /// Only an `Ipv4` address is accepted.
    #[arg(short, long, default_value_t = Ipv4Addr::UNSPECIFIED)]
    address: Ipv4Addr,

    /// Server host name.
    #[arg(short = 'n', long)]
    hostname: String,

    /// Server port.
    #[arg(short, long, default_value_t = 3000)]
    port: u16,

    /// Service domain.
    #[arg(short = 'd', long = "domain")]
    service_domain: String,

    /// Service transport protocol.
    #[arg(short = 't', long = "protocol", default_value_t = TransportProtocol::TCP, value_parser = ValueParser::new(parse_transport_protocol))]
    service_transport_protocol: TransportProtocol,
}

fn camera_format(
    x: u32,
    y: u32,
    fps: u32,
    fourcc: impl AsRef<str>,
) -> Result<CameraFormat, NokhwaError> {
    let fourcc = fourcc.as_ref().parse::<FrameFormat>()?;
    let resolution = Resolution::new(x, y);
    Ok(CameraFormat::new(resolution, fourcc, fps))
}

fn change_format(device: Device<InternalState>) -> Device<InternalState> {
    // Route to change camera index.
    let change_camera_route = Route::get("Change camera", "/change-camera")
        .description("Change camera.")
        .with_parameters(Parameters::new().characters_sequence("index", "0"));

    // Route to change format type to random.
    let change_format_random_route =
        Route::get("Random", "/random").description("Change stream format type to random.");

    // Route to change format type to highest resolution.
    let change_format_absolute_highest_resolution_route = Route::get(
        "Absolute highest resolution",
        "/absolute-highest-resolution",
    )
    .description("Change stream format type to absolute highest resolution.");

    // Route to change format type to highest frame rate.
    let change_format_absolute_highest_framerate_route =
        Route::get("Absolute highest framerate", "/absolute-highest-framerate")
            .description("Change stream format to absolute highest framerate.");

    // Route to change format type to highest resolution.
    let change_format_highest_resolution_route =
        Route::post("Highest resolution", "/highest-resolution")
            .description("Change stream format to highest resolution.")
            .with_parameters(Parameters::new().u32("x", 1920).u32("y", 1080));

    // Route to change format type to highest framerate.
    let change_format_highest_framerate_route =
        Route::post("Highest framerate", "/highest-framerate")
            .description("Change stream format to highest framerate.")
            .with_parameters(Parameters::new().u32("fps", 30));

    // Route to change format type to exact type.
    let change_format_exact_route = Route::post("Exact", "/exact")
        .description("Change stream format to exact type.")
        .with_parameters(
            Parameters::new()
                .u32("x", 1920)
                .u32("y", 1080)
                .u32("fps", 30)
                .characters_sequence("fourcc", "YUYV"),
        );

    // Route to change format type to closest type.
    let change_format_closest_route = Route::post("Closest", "/closest")
        .description("Change stream to closest type.")
        .with_parameters(
            Parameters::new()
                .u32("x", 1920)
                .u32("y", 1080)
                .u32("fps", 30)
                .characters_sequence("fourcc", "YUYV"),
        );

    device
        .route(serial_stateful(change_camera_route, change_camera))
        .route(ok_stateful(change_format_random_route, format_random))
        .route(ok_stateful(
            change_format_absolute_highest_resolution_route,
            format_absolute_resolution,
        ))
        .route(ok_stateful(
            change_format_absolute_highest_framerate_route,
            format_absolute_framerate,
        ))
        .route(ok_stateful(
            change_format_highest_resolution_route,
            format_highest_resolution,
        ))
        .route(ok_stateful(
            change_format_highest_framerate_route,
            format_highest_framerate,
        ))
        .route(ok_stateful(change_format_exact_route, format_exact))
        .route(ok_stateful(change_format_closest_route, format_closest))
}

fn screenshot(device: Device<InternalState>) -> Device<InternalState> {
    // Route to take a screenshot with a random format.
    let screenshot_random_route = Route::get("Screenshot random", "/screenshot-random")
        .description("Screenshot with a random camera format.")
        .with_array_of_hazards([
            Hazard::ElectricEnergyConsumption,
            Hazard::TakeDeviceScreenshots,
            Hazard::TakePictures,
        ]);

    // Route to view screenshot with absolute resolution.
    let screenshot_absolute_resolution_route = Route::get(
        "Screenshot absolute resolution",
        "/screenshot-absolute-resolution",
    )
    .description("Screenshot from a camera with absolute resolution.")
    .with_array_of_hazards([
        Hazard::ElectricEnergyConsumption,
        Hazard::TakeDeviceScreenshots,
        Hazard::TakePictures,
    ]);

    // Route to view screenshot with absolute framerate.
    let screenshot_absolute_framerate_route = Route::get(
        "Screenshot absolute framerate",
        "/screenshot-absolute-framerate",
    )
    .description("Screenshot from a camera with absolute framerate.")
    .with_array_of_hazards([
        Hazard::ElectricEnergyConsumption,
        Hazard::TakeDeviceScreenshots,
        Hazard::TakePictures,
    ]);

    // Route to view screenshot with highest resolution.
    let screenshot_highest_resolution_route = Route::post(
        "Screenshot highest resolution",
        "/screenshot-highest-resolution",
    )
    .description("Screenshot from a camera with highest resolution.")
    .with_array_of_hazards([
        Hazard::ElectricEnergyConsumption,
        Hazard::TakeDeviceScreenshots,
        Hazard::TakePictures,
    ])
    .with_parameters(Parameters::new().u32("x", 1920).u32("y", 1080));

    // Route to view screenshot with highest framerate.
    let screenshot_highest_framerate_route = Route::post(
        "Screenshot highest framerate",
        "/screenshot-highest-framerate",
    )
    .description("Screenshot from a camera with highest framerate.")
    .with_array_of_hazards([
        Hazard::ElectricEnergyConsumption,
        Hazard::TakeDeviceScreenshots,
        Hazard::TakePictures,
    ])
    .with_parameters(Parameters::new().u32("fps", 30));

    // Route to view screenshot with exact approach.
    let screenshot_exact_route = Route::post("Screenshot exact", "/screenshot-exact")
        .description("Screenshot from a camera with exact type.")
        .with_array_of_hazards([
            Hazard::ElectricEnergyConsumption,
            Hazard::TakeDeviceScreenshots,
            Hazard::TakePictures,
        ])
        .with_parameters(
            Parameters::new()
                .u32("x", 1920)
                .u32("y", 1080)
                .u32("fps", 30)
                .characters_sequence("fourcc", "YUYV"),
        );

    // Route to view screenshot with closest type.
    let screenshot_closest_route = Route::post("Screenshot closest", "/screenshot-closest")
        .description("Screenshot from a camera with closest type.")
        .with_array_of_hazards([
            Hazard::ElectricEnergyConsumption,
            Hazard::TakeDeviceScreenshots,
            Hazard::TakePictures,
        ])
        .with_parameters(
            Parameters::new()
                .u32("x", 1920)
                .u32("y", 1080)
                .u32("fps", 30)
                .characters_sequence("fourcc", "YUYV"),
        );

    device
        .route(stream_stateful(screenshot_random_route, screenshot_random))
        .route(stream_stateful(
            screenshot_absolute_resolution_route,
            screenshot_absolute_resolution,
        ))
        .route(stream_stateful(
            screenshot_absolute_framerate_route,
            screenshot_absolute_framerate,
        ))
        .route(stream_stateful(
            screenshot_highest_resolution_route,
            screenshot_highest_resolution,
        ))
        .route(stream_stateful(
            screenshot_highest_framerate_route,
            screenshot_highest_framerate,
        ))
        .route(stream_stateful(screenshot_exact_route, screenshot_exact))
        .route(stream_stateful(
            screenshot_closest_route,
            screenshot_closest,
        ))
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("failed to run a `tosca-os` method")]
    Tosca(tosca_os::error::Error),
    #[error("no camera backend found")]
    NoCameraBackend,
    #[error("failed to find any camera in the system")]
    NoCameraFound(#[source] NokhwaError),
    #[error("failed to retrieve the first camera available")]
    MissingFirstCamera,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing subscriber.
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .init();

    // Command line parser.
    let cli = Cli::parse();

    // This initialization is necessary only on MacOS, but we are also going
    // to use this call to verify if everything went well.
    nokhwa::nokhwa_initialize(|granted| {
        if granted {
            info!("Nokhwa initialized correctly.");
        } else {
            info!("Nokhwa not initialized correctly. Exiting the process.");
            std::process::exit(1);
        }
    });

    // Retrieve native API camera backend
    let camera_backend = native_api_backend().ok_or(Error::NoCameraBackend)?;

    // Retrieve all cameras present on a system
    let cameras = query(camera_backend).map_err(Error::NoCameraFound)?;

    // Retrieve first camera present in the system
    let first_camera = cameras.first().ok_or(Error::MissingFirstCamera)?;

    // Camera configuration.
    let camera = CameraConfig {
        index: first_camera.index().clone(),
        format_type: RequestedFormatType::None,
    };

    // Route to view camera stream.
    let camera_stream_route = Route::get("Stream", "/stream")
        .description("View camera stream.")
        .with_array_of_hazards([
            Hazard::ElectricEnergyConsumption,
            Hazard::VideoDisplay,
            Hazard::VideoRecordAndStore,
        ]);

    // Route to view all available cameras.
    let view_cameras_route =
        Route::get("View all", "/view-all").description("View all system cameras.");

    // Route to view camera info.
    let camera_info_route =
        Route::get("View info", "/view-info").description("View current camera data.");

    // A camera device which is going to be run on the server.
    let device = Device::with_state(InternalState::new(camera))
        .main_route("/camera")
        .route(stream_stateful(camera_stream_route, show_camera_stream))
        .route(serial_stateless(view_cameras_route, show_available_cameras))
        .route(serial_stateful(camera_info_route, show_camera_info));

    let device = change_format(device);
    let device = screenshot(device);

    Server::new(device)
        .address(cli.address)
        .port(cli.port)
        .discovery_service(
            ServiceConfig::mdns_sd("camera")
                .hostname(&cli.hostname)
                .domain(&cli.service_domain)
                .transport_protocol(cli.service_transport_protocol),
        )
        .run()
        .await
        .map_err(Error::Tosca)
}
