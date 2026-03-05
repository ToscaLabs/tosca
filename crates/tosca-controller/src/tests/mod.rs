use std::net::Ipv4Addr;
use std::time::Duration;

use tosca::device::{DeviceEnvironment, DeviceKindId};
use tosca::hazards::{Hazard, Hazards};
use tosca::parameters::{ParameterKind, Parameters, ParametersData};
use tosca::response::ResponseKind;
use tosca::route::{LightOffRoute, LightOnRoute, RestKind, Route};

use tosca_os::devices::light::Light;
use tosca_os::extract::Path;
use tosca_os::responses::error::ErrorResponse;
use tosca_os::responses::ok::{OkResponse, mandatory_ok_stateless};
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
    let light_on_route = LightOnRoute::put("On")
        .description("Turn light on.")
        .with_hazard(Hazard::ElectricEnergyConsumption);

    // Turn light off `PUT` route.
    let light_off_route = LightOffRoute::put("Off")
        .description("Turn light off.")
        .with_hazard(Hazard::LogEnergyConsumption);

    // A light device which is going to be run on the server.
    let light = Light::new()
        // This method is mandatory, if not called, a compiler error is raised.
        .turn_light_on(light_on_route, mandatory_ok_stateless(turn_light_on))
        // This method is mandatory, if not called, a compiler error is raised.
        .turn_light_off(light_off_route, mandatory_ok_stateless(turn_light_off));

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
            .unwrap()
    } else {
        light.main_route(SECOND_DEVICE_ROUTE)
    };

    info!(
        "Inside the light device {} `toggle` route and port {port}",
        if with_toggle { "with" } else { "without" }
    );

    // Run a discovery service and the device on the server.
    Server::new(light.build())
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

pub(crate) async fn light_with_toggle(close_rx: tokio::sync::oneshot::Receiver<()>) {
    light(PORT_ONE, "light-with-toggle", true, close_rx).await;
}

pub(crate) async fn light_without_toggle(close_rx: tokio::sync::oneshot::Receiver<()>) {
    light(PORT_TWO, "light-without-toggle", false, close_rx).await;
}

fn build_route(device: &Device, route: &str) -> String {
    format!(
        "{}{}{}",
        device.network_info().last_reachable_address,
        device.description().main_route,
        route
    )
}

fn check_request(
    device: &Device,
    route: &str,
    description: &str,
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
            description: Some(description.to_string()),
            parameters_data,
            response_kind,
            device_environment: DeviceEnvironment::Os,
        })
    );
}

// Device addresses are not considered in the comparisons, because they
// depend on the machine this test is being run on.
pub(crate) fn compare_device_data(device: &Device) {
    // Check port.
    assert!(device.network_info().port == PORT_ONE || device.network_info().port == PORT_TWO);

    // Check scheme.
    let scheme = device.network_info().properties.get("scheme");
    assert!(scheme.is_some_and(|scheme| scheme == "http"));

    // Check path.
    let path = device.network_info().properties.get("path");
    assert!(
        path.is_some_and(|path| path == "/.well-known/light-with-toggle"
            || path == "/.well-known/light-without-toggle")
    );

    // Check device main route.
    assert!(
        device.description().main_route == FIRST_DEVICE_ROUTE
            || device.description().main_route == SECOND_DEVICE_ROUTE
    );

    // Check device information.
    assert_eq!(device.description().kind, DeviceKindId::new("Light"));
    assert_eq!(device.description().environment, DeviceEnvironment::Os);

    // Check requests number.
    assert!(
        device.description().main_route == FIRST_DEVICE_ROUTE && device.requests_count() == 3
            || device.description().main_route == SECOND_DEVICE_ROUTE
                && device.requests_count() == 2
    );

    if device.description().main_route == FIRST_DEVICE_ROUTE {
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
        "Turn light off.",
        RestKind::Put,
        Hazards::init(Hazard::LogEnergyConsumption),
        ParametersData::new(),
        ResponseKind::Ok,
    );
}

pub(crate) async fn check_function_with_device<F>(function: F)
where
    F: AsyncFnOnce(),
{
    let _ = tracing_subscriber::fmt().try_init();

    let (close_tx, close_rx) = tokio::sync::oneshot::channel();

    // Run a device task.
    let device_handle = tokio::spawn(async { light_with_toggle(close_rx).await });

    // Wait for device task to be configured.
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Run function.
    function().await;

    // Shutdown device server.
    _ = close_tx.send(());

    // Wait for device server to gracefully shutdown.
    _ = device_handle.await;
}

pub(crate) async fn check_function_with_two_devices<F>(function: F)
where
    F: AsyncFnOnce(),
{
    let _ = tracing_subscriber::fmt().try_init();

    let (device1_tx, device1_rx) = tokio::sync::oneshot::channel();
    let (device2_tx, device2_rx) = tokio::sync::oneshot::channel();

    // Run first device task.
    let device1_handle = tokio::spawn(async { light_without_toggle(device1_rx).await });

    // Wait for first device task to be configured.
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Run second device task.
    let device2_handle = tokio::spawn(async { light_with_toggle(device2_rx).await });

    // Wait for second device task to be configured.
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Run function.
    function().await;

    // Shutdown first device server.
    _ = device1_tx.send(());

    // Wait for first device server to gracefully shutdown.
    _ = device1_handle.await;

    // Shutdown second device server.
    _ = device2_tx.send(());

    // Wait for second device server to gracefully shutdown.
    _ = device2_handle.await;
}
