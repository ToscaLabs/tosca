use std::net::Ipv4Addr;
use std::time::Duration;

use tosca::device::{DeviceEnvironment, DeviceScheme, DeviceSchemeOwned, schemes::LIGHT_SCHEME};
use tosca::hazards::{Hazard, Hazards};
use tosca::parameters::{ParameterKind, Parameters, ParametersData};
use tosca::response::ResponseKind;
use tosca::route::{RestKind, Route};

use tosca_os::device::Device as ToscaOsDevice;
use tosca_os::extract::Path;
use tosca_os::responses::error::ErrorResponse;
use tosca_os::responses::ok::{OkResponse, ok_stateless};
use tosca_os::responses::serial::{SerialResponse, serial_stateless};
use tosca_os::server::Server;
use tosca_os::service::ServiceConfig;

use serde::{Deserialize, Serialize};

use tracing::info;

use crate::device::Device;
use crate::request::Request;

const PORT_ONE: u16 = 3000;
const PORT_TWO: u16 = 4000;

const FIRST_DEVICE_ROUTE: &str = "/with-toggle";
const SECOND_DEVICE_ROUTE: &str = "/without-toggle";

pub(crate) const DOMAIN: &str = "tosca";

async fn turn_light_on() -> Result<OkResponse, ErrorResponse> {
    println!("Light on");
    Ok(OkResponse::ok())
}

async fn turn_light_off() -> Result<OkResponse, ErrorResponse> {
    println!("Light off");
    Ok(OkResponse::ok())
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Brightness {
    pub(crate) brightness: u64,
}

async fn toggle(Path(brightness): Path<u64>) -> Result<SerialResponse<Brightness>, ErrorResponse> {
    println!("Brightness: {brightness}");
    Ok(SerialResponse::new(Brightness { brightness }))
}

async fn light(
    port: u16,
    id: &str,
    with_toggle: bool,
    close_rx: tokio::sync::oneshot::Receiver<()>,
) {
    // Turn light on `PUT` route.
    let light_on_route = Route::put("On", "/on")
        .description("Turn light on.")
        .with_hazard(Hazard::ElectricEnergyConsumption);

    // Turn light off `PUT` route.
    let light_off_route = Route::put("Off", "/off")
        .description("Turn light off.")
        .with_hazard(Hazard::LogEnergyConsumption);

    // A light device which is going to be run on the server.
    let light = ToscaOsDevice::new(LIGHT_SCHEME)
        .route(ok_stateless(light_on_route, turn_light_on))
        .route(ok_stateless(light_off_route, turn_light_off));

    let light = if with_toggle {
        // Toggle `PUT` route.
        let toggle_route = Route::get("Toggle", "/toggle")
            .description("Toggle a light.")
            .with_hazards(
                Hazards::new()
                    .insert(Hazard::FireHazard)
                    .insert(Hazard::ElectricEnergyConsumption),
            )
            .with_parameters(Parameters::new().rangeu64("brightness", (0, 20, 1)));

        light
            .main_route(FIRST_DEVICE_ROUTE)
            .route(serial_stateless(toggle_route, toggle))
    } else {
        light.main_route(SECOND_DEVICE_ROUTE)
    };

    info!(
        "Inside the light device {} `toggle` route and port {port}",
        if with_toggle { "with" } else { "without" }
    );

    // Run a discovery service and the device on the server.
    Server::new(light.build().expect("Failed to validate device data"))
        .address(Ipv4Addr::UNSPECIFIED)
        .port(port)
        .well_known_service(id)
        .discovery_service(ServiceConfig::mdns_sd(id).hostname("tosca").domain(DOMAIN))
        .with_graceful_shutdown(async move {
            _ = close_rx.await;
        })
        .run()
        .await
        .expect("Error in running a device server.");
}

#[inline]
pub(crate) async fn light_with_toggle(close_rx: tokio::sync::oneshot::Receiver<()>) {
    light(PORT_ONE, "light-with-toggle", true, close_rx).await;
}

#[inline]
pub(crate) async fn light_without_toggle(close_rx: tokio::sync::oneshot::Receiver<()>) {
    light(PORT_TWO, "light-without-toggle", false, close_rx).await;
}

async fn turn_thermostat_on() -> Result<OkResponse, ErrorResponse> {
    println!("Thermostat on");
    Ok(OkResponse::ok())
}

async fn turn_thermostat_off() -> Result<OkResponse, ErrorResponse> {
    println!("Thermostat off");
    Ok(OkResponse::ok())
}

pub(crate) async fn custom_device(close_rx: tokio::sync::oneshot::Receiver<()>) {
    // Turn thermostat on `PUT` route.
    let thermostat_on_route = Route::put("On", "/on")
        .description("Turn thermostat on.")
        .with_hazard(Hazard::ElectricEnergyConsumption);

    // Turn thermostat off `PUT` route.
    let thermostat_off_route = Route::put("Off", "/off")
        .description("Turn thermostat off.")
        .with_hazard(Hazard::LogEnergyConsumption);

    // Create base scheme for custom thermostat.
    let scheme = DeviceScheme::base_custom_scheme("Thermostat");

    // A custom thermostat device which is going to be run on the server.
    let light = ToscaOsDevice::new(scheme)
        .route(ok_stateless(thermostat_on_route, turn_thermostat_on))
        .route(ok_stateless(thermostat_off_route, turn_thermostat_off))
        .main_route("/thermostat");

    info!("Inside the custom thermostat device with port {PORT_TWO}");

    // Run a discovery service and the device on the server.
    Server::new(light.build().expect("Failed to validate device data"))
        .address(Ipv4Addr::UNSPECIFIED)
        .port(PORT_TWO)
        .well_known_service("thermostat")
        .discovery_service(
            ServiceConfig::mdns_sd("thermostat")
                .hostname("tosca")
                .domain(DOMAIN),
        )
        .with_graceful_shutdown(async move {
            _ = close_rx.await;
        })
        .run()
        .await
        .expect("Error in running a device server.");
}

fn build_route(device: &Device, route: &str) -> String {
    format!(
        "{}{}{}",
        device.network_info().last_reachable_address,
        device.metadata().main_route,
        route
    )
}

fn check_request(
    device: &Device,
    route: &str,
    #[cfg(feature = "metadata")] description: &str,
    kind: RestKind,
    hazards: Hazards,
    parameters_data: ParametersData,
    response_kind: ResponseKind,
) {
    let request_sender = device.request(route);

    assert_eq!(
        request_sender,
        Some(&Request {
            kind,
            hazards,
            route: build_route(device, route),
            #[cfg(feature = "metadata")]
            description: Some(description.to_string()),
            parameters_data,
            response_kind,
            device_environment: DeviceEnvironment::Os,
        })
    );
}

// Device addresses are not considered in the comparisons, because they
// depend on the machine this test is being run on.
pub(crate) fn analyze_light_data(device: &Device) {
    // Check port.
    assert!(device.network_info().port == PORT_ONE || device.network_info().port == PORT_TWO);

    // Check protocol scheme.
    let protocol_scheme = device.network_info().properties.get("scheme");
    assert!(protocol_scheme.is_some_and(|protocol_scheme| protocol_scheme == "http"));

    // Check path.
    let path = device.network_info().properties.get("path");
    assert!(
        path.is_some_and(|path| path == "/.well-known/light-with-toggle"
            || path == "/.well-known/light-without-toggle")
    );

    // Check device main route.
    assert!(
        device.metadata().main_route == FIRST_DEVICE_ROUTE
            || device.metadata().main_route == SECOND_DEVICE_ROUTE
    );

    // Check device information.
    assert_eq!(
        device.metadata().scheme,
        DeviceSchemeOwned::from(LIGHT_SCHEME)
    );
    assert_eq!(device.metadata().environment, DeviceEnvironment::Os);

    // Check requests number.
    assert!(
        device.metadata().main_route == FIRST_DEVICE_ROUTE && device.requests_count() == 3
            || device.metadata().main_route == SECOND_DEVICE_ROUTE && device.requests_count() == 2
    );

    if device.metadata().main_route == FIRST_DEVICE_ROUTE {
        let parameters_data = ParametersData::new().insert(
            "brightness".into(),
            ParameterKind::RangeU64 {
                min: 0,
                max: 20,
                step: 1,
                default: 0,
            },
        );
        // Check "/toggle" request
        check_request(
            device,
            "/toggle",
            #[cfg(feature = "metadata")]
            "Toggle a light.",
            RestKind::Get,
            Hazards::new()
                .insert(Hazard::FireHazard)
                .insert(Hazard::ElectricEnergyConsumption),
            parameters_data,
            ResponseKind::Serial,
        );
    }

    // Check "/on" request
    check_request(
        device,
        "/on",
        #[cfg(feature = "metadata")]
        "Turn light on.",
        RestKind::Put,
        Hazards::init(Hazard::ElectricEnergyConsumption),
        ParametersData::new(),
        ResponseKind::Ok,
    );

    // Check "/off" request
    check_request(
        device,
        "/off",
        #[cfg(feature = "metadata")]
        "Turn light off.",
        RestKind::Put,
        Hazards::init(Hazard::LogEnergyConsumption),
        ParametersData::new(),
        ResponseKind::Ok,
    );
}

pub(crate) async fn run_one_device<D, Fut, F>(device: D, task: F)
where
    D: Fn(tokio::sync::oneshot::Receiver<()>) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
    F: AsyncFnOnce(),
{
    let _ = tracing_subscriber::fmt().try_init();

    let (device_tx, device_rx) = tokio::sync::oneshot::channel();

    // Run a device task.
    let device_handle = tokio::spawn(device(device_rx));

    // Wait for device task to be configured.
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Run task.
    task().await;

    // Shutdown device server.
    _ = device_tx.send(());

    // Wait for device server to gracefully shutdown.
    _ = device_handle.await;
}

pub(crate) async fn run_two_devices<D1, D2, Fut1, Fut2, F>(device1: D1, device2: D2, task: F)
where
    D1: Fn(tokio::sync::oneshot::Receiver<()>) -> Fut1 + Send + 'static,
    D2: Fn(tokio::sync::oneshot::Receiver<()>) -> Fut2 + Send + 'static,
    Fut1: Future<Output = ()> + Send + 'static,
    Fut2: Future<Output = ()> + Send + 'static,
    F: AsyncFnOnce(),
{
    let _ = tracing_subscriber::fmt().try_init();

    let (device1_tx, device1_rx) = tokio::sync::oneshot::channel();
    let (device2_tx, device2_rx) = tokio::sync::oneshot::channel();

    // Run first device task.
    let device1_handle = tokio::spawn(device1(device1_rx));

    // Wait for first device task to be configured.
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Run second device task.
    let device2_handle = tokio::spawn(device2(device2_rx));

    // Wait for second device task to be configured.
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Run task.
    task().await;

    // Shutdown first device server.
    _ = device1_tx.send(());

    // Wait for first device server to gracefully shutdown.
    _ = device1_handle.await;

    // Shutdown second device server.
    _ = device2_tx.send(());

    // Wait for second device server to gracefully shutdown.
    _ = device2_handle.await;
}
