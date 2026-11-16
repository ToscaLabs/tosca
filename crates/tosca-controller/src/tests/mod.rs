use std::future::Future;
use std::net::Ipv4Addr;

use tosca::device::{
    DeviceDescription, DeviceEnvironment, DeviceScheme, DeviceSchemeOwned, schemes::LIGHT_SCHEME,
};
use tosca::hazards::{Hazard, Hazards};
use tosca::parameters::{ParameterKind, Parameters, ParametersData};
use tosca::response::{OkResponse, ResponseKind, SerialResponse};
use tosca::route::{RestKind, Route, RouteConfigs};

use axum::{
    Json, Router,
    extract::Path,
    routing::{get, put},
};

use mdns_sd::{ResolvedService, ServiceInfo};

use serde::{Deserialize, Serialize};

use crate::device::Device;
use crate::request::Request;

const FIRST_DEVICE_ROUTE: &str = "/with-toggle";
const SECOND_DEVICE_ROUTE: &str = "/without-toggle";

pub(crate) const DOMAIN: &str = "tosca";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Brightness {
    pub(crate) brightness: u64,
}

#[derive(Clone, Copy)]
pub(crate) enum TestDevice {
    LightWithToggle,
    LightWithoutToggle,
    Thermostat,
}

impl TestDevice {
    const fn name(self) -> &'static str {
        match self {
            Self::LightWithToggle => "light-with-toggle",
            Self::LightWithoutToggle => "light-without-toggle",
            Self::Thermostat => "thermostat",
        }
    }

    const fn main_route(self) -> &'static str {
        match self {
            Self::LightWithToggle => FIRST_DEVICE_ROUTE,
            Self::LightWithoutToggle => SECOND_DEVICE_ROUTE,
            Self::Thermostat => "/thermostat",
        }
    }
}

fn test_router(device: TestDevice) -> Router {
    let main_route = device.main_route();
    let on = Route::put("On", "/on")
        .description("Turn light on.")
        .with_hazard(Hazard::ElectricEnergyConsumption);
    let off = Route::put("Off", "/off")
        .description("Turn light off.")
        .with_hazard(Hazard::LogEnergyConsumption);

    let mut route_configs = RouteConfigs::new();
    route_configs.add(on.serialize_data());
    route_configs.add(off.serialize_data());

    if matches!(device, TestDevice::LightWithToggle) {
        let toggle = Route::get("Toggle", "/toggle")
            .description("Toggle a light.")
            .with_hazards(
                Hazards::new()
                    .insert(Hazard::FireHazard)
                    .insert(Hazard::ElectricEnergyConsumption),
            )
            .with_parameters(Parameters::new().rangeu64("brightness", (0, 20, 1)));
        route_configs.add(
            toggle
                .serialize_data()
                .change_response_kind(ResponseKind::Serial),
        );
    }

    let scheme = if matches!(device, TestDevice::Thermostat) {
        DeviceScheme::base_custom_scheme("Thermostat")
    } else {
        LIGHT_SCHEME
    };
    let description = DeviceDescription::new(scheme, main_route, route_configs)
        .environment(DeviceEnvironment::Os)
        .ethernet_mac([0x02, 0, 0, 0, 0, 1]);

    let mut router = Router::new()
        .route(
            "/",
            get(move || {
                let description = description.clone();
                async move { Json(description) }
            }),
        )
        .route(
            &format!("{main_route}/on"),
            put(|| async { Json(OkResponse::ok()) }),
        )
        .route(
            &format!("{main_route}/off"),
            put(|| async { Json(OkResponse::ok()) }),
        );

    if matches!(device, TestDevice::LightWithToggle) {
        router = router.route(
            &format!("{main_route}/toggle/{{brightness}}"),
            get(|Path(brightness): Path<u64>| async move {
                Json(SerialResponse::new(Brightness { brightness }))
            }),
        );
    }
    router
}

pub(crate) async fn run_test_devices<F, Fut>(devices: &[TestDevice], test: F)
where
    F: FnOnce(Vec<ResolvedService>) -> Fut,
    Fut: Future<Output = ()>,
{
    let mut servers = Vec::new();
    let mut services = Vec::new();

    for &device in devices {
        // The listener uses a free local port; mDNS data is supplied directly.
        let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let port = listener.local_addr().unwrap().port();
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
        let server = tokio::spawn(async move {
            axum::serve(listener, test_router(device))
                .with_graceful_shutdown(async move {
                    let _ = shutdown_rx.await;
                })
                .await
                .unwrap();
        });

        let path = format!("/.well-known/{}", device.name());
        let service = ServiceInfo::new(
            "_tosca._tcp.local.",
            device.name(),
            "tosca.local.",
            "127.0.0.1",
            port,
            &[("scheme", "http"), ("path", path.as_str())][..],
        )
        .unwrap()
        .as_resolved_service();
        services.push(service);
        servers.push((shutdown_tx, server));
    }

    test(services).await;

    for (shutdown, server) in servers {
        let _ = shutdown.send(());
        server.await.unwrap();
    }
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

pub(crate) fn analyze_light_data(device: &Device) {
    assert_ne!(device.network_info().port, 0);

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
