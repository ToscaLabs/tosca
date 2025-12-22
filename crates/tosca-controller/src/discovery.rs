use std::borrow::Cow;
use std::net::IpAddr;
use std::time::Duration;

use tosca::device::DeviceData;

use flume::RecvTimeoutError;

use mdns_sd::{IfKind, Receiver, ResolvedService, ServiceDaemon, ServiceEvent};

use tokio::time::sleep;

use tracing::{info, warn};

use crate::device::{Description, Device, Devices, NetworkInformation, build_device_address};
use crate::error::Error;
use crate::events::Events;
use crate::request::create_requests;

// Service top-level domain.
//
// It defines the default top-level domain for a service.
const TOP_LEVEL_DOMAIN: &str = "local";

/// Service transport protocol.
#[derive(Debug, PartialEq)]
pub enum TransportProtocol {
    /// TCP service.
    TCP,
    /// UDP service.
    UDP,
}

impl std::fmt::Display for TransportProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.name().fmt(f)
    }
}

impl TransportProtocol {
    /// Returns the [`TransportProtocol`] name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::TCP => "tcp",
            Self::UDP => "udp",
        }
    }
}

/// Devices discovery.
///
/// It detects all `tosca`-compliant [`Device`]s in a network.
#[derive(Debug, PartialEq)]
pub struct Discovery {
    domain: Cow<'static, str>,
    transport_protocol: TransportProtocol,
    top_level_domain: Cow<'static, str>,
    timeout: Duration,
    disable_ipv6: bool,
    disable_ip: Option<IpAddr>,
    disable_network_interface: Option<&'static str>,
}

impl Discovery {
    /// Creates a [`Discovery`].
    #[must_use]
    #[inline]
    pub fn new(domain: impl Into<Cow<'static, str>>) -> Self {
        Self {
            domain: domain.into(),
            transport_protocol: TransportProtocol::TCP,
            top_level_domain: Cow::Borrowed(TOP_LEVEL_DOMAIN),
            timeout: Duration::from_secs(2), // Default timeout of 2s.
            disable_ipv6: false,
            disable_ip: None,
            disable_network_interface: None,
        }
    }

    /// Sets a different timeout.
    #[must_use]
    pub const fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the service transport protocol.
    #[must_use]
    pub const fn transport_protocol(mut self, transport_protocol: TransportProtocol) -> Self {
        self.transport_protocol = transport_protocol;
        self
    }

    /// Changes service domain.
    #[must_use]
    #[inline]
    pub fn domain(mut self, domain: impl Into<Cow<'static, str>>) -> Self {
        self.domain = domain.into();
        self
    }

    /// Sets the service top-level domain.
    #[must_use]
    #[inline]
    pub fn top_level_domain(mut self, top_level_domain: impl Into<Cow<'static, str>>) -> Self {
        self.top_level_domain = top_level_domain.into();
        self
    }

    /// Do not discover devices with `IPv6` interfaces.
    #[must_use]
    pub const fn disable_ipv6(mut self) -> Self {
        self.disable_ipv6 = true;
        self
    }

    /// Disables the given IP address.
    #[must_use]
    #[inline]
    pub fn disable_ip(mut self, ip: impl Into<IpAddr>) -> Self {
        self.disable_ip = Some(ip.into());
        self
    }

    /// Disables the given network interface.
    #[must_use]
    pub const fn disable_network_interface(mut self, network_interface: &'static str) -> Self {
        self.disable_network_interface = Some(network_interface);
        self
    }

    pub(crate) async fn discover(&self) -> Result<Devices, Error> {
        // Discover devices.
        let discovery_info = self.discover_devices().await?;

        Self::obtain_devices_data(discovery_info).await
    }

    async fn discover_devices(&self) -> Result<Vec<ResolvedService>, Error> {
        // Create a mdns daemon
        let mdns = ServiceDaemon::new()?;

        // Disable IPv6 interface.
        if self.disable_ipv6 {
            mdns.disable_interface(IfKind::IPv6)?;
        }

        // Disable IP.
        if let Some(ip) = self.disable_ip {
            mdns.disable_interface(ip)?;
        }

        // Disable network interface.
        if let Some(network_interface) = self.disable_network_interface {
            mdns.disable_interface(network_interface)?;
        }

        // Service type.
        let service_type = format!(
            "_{}._{}.{}.",
            self.domain,
            self.transport_protocol.name(),
            self.top_level_domain
        );

        // Detects devices.
        let receiver = mdns.browse(&service_type)?;

        // Discovery service.
        let mut discovery_service = Vec::new();

        // Run for n-seconds in search of devices and saves their information
        // in memory.
        while let Ok(event) = self.with_timeout(&receiver).await {
            if let ServiceEvent::ServiceResolved(info) = event {
                // Check whether there are device addresses.
                //
                // If no address has been found, prints a warning and
                // continue the loop.
                if info.get_addresses().is_empty() {
                    warn!("No device address available for {:?}", info);
                    continue;
                }

                // If two devices are equal, skip to the next one.
                if Self::check_device_duplicates(&discovery_service, &info) {
                    continue;
                }

                discovery_service.push(*info);
            }
        }

        // Stop detection.
        mdns.stop_browse(&service_type)?;

        Ok(discovery_service)
    }

    #[inline]
    async fn with_timeout<T>(&self, receiver: &Receiver<T>) -> Result<T, RecvTimeoutError> {
        let timeout_future = sleep(self.timeout);

        tokio::select! {
            () = timeout_future => {
                // This is the same error returned by the `recv_timeout`
                // function in case of a timeout.
                Err(RecvTimeoutError::Timeout)
            }
            result = receiver.recv_async() => {
                result.map_err(|_| RecvTimeoutError::Disconnected)
            }
        }
    }

    async fn obtain_devices_data(
        discovery_service: Vec<ResolvedService>,
    ) -> Result<Devices, Error> {
        // Devices collection.
        let mut devices = Devices::new();

        // Iterate over discovered metadata
        for service in discovery_service {
            // Try to contact each available address for a device
            // to retrieve data.
            for address in &service.addresses {
                let complete_address = build_device_address(
                    service
                        .txt_properties
                        .get_property_val_str("scheme")
                        // If the scheme is not specified as a property,
                        // fall back to `http` as default.
                        .unwrap_or("http"),
                    &address.to_ip_addr(),
                    service.port,
                );
                info!("Complete address: {complete_address}");

                // Contact devices to retrieve their data
                match reqwest::get(&complete_address).await {
                    Ok(response) => {
                        let device_data: DeviceData = response.json().await?;

                        if device_data.wifi_mac.is_none() && device_data.ethernet_mac.is_none() {
                            warn!(
                                "Ignoring device {complete_address} because no valid MAC addresses have been found"
                            );
                            continue;
                        }

                        let requests = create_requests(
                            device_data.route_configs,
                            &complete_address,
                            &device_data.main_route,
                            device_data.environment,
                        );

                        let description = Description::new(
                            device_data.kind,
                            device_data.environment,
                            device_data.main_route.into_owned(),
                        );

                        let mut network_info = NetworkInformation::new(
                            service.fullname,
                            service
                                .addresses
                                .into_iter()
                                .map(|address| address.to_ip_addr())
                                .collect(),
                            service.port,
                            service.txt_properties.into_property_map_str(),
                            complete_address,
                        );

                        if let Some(mac) = device_data.wifi_mac {
                            network_info = network_info.wifi_mac(mac);
                        }

                        if let Some(mac) = device_data.ethernet_mac {
                            network_info = network_info.ethernet_mac(mac);
                        }

                        let events = device_data.events_description.map(Events::new);

                        devices.add(Device::init(network_info, description, requests, events));

                        // Only a single address is necessary.
                        break;
                    }
                    Err(e) => {
                        warn!("Impossible to contact address {complete_address}: {e}");
                    }
                }
            }
        }

        Ok(devices)
    }

    // A discovered device is equal to another device when:
    //
    // - It has an address with IP and port identical to the ones of
    //   another device address.
    //   Devices belonging to the same local network CANNOT HAVE any IP
    //   and port in common.
    //
    //   OR
    //
    // - It has the same full name of another device belonging to the same
    //   network. A full name, in this case, represents the device ID.
    //   Two devices belonging to the same network CANNOT HAVE the same ID.
    fn check_device_duplicates(
        discovery_service: &[ResolvedService],
        info: &ResolvedService,
    ) -> bool {
        for disco_service in discovery_service {
            // When the addresses have distinct ports, they are always
            // different, so they are not considered.
            if disco_service.port != info.get_port() {
                continue;
            }

            for address in &disco_service.addresses {
                if info.get_addresses().contains(address) {
                    return true;
                }
            }

            if disco_service.fullname == info.get_fullname() {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::time::Duration;

    use tracing::warn;

    use serial_test::serial;

    use crate::tests::{
        DOMAIN, check_function_with_device, check_function_with_two_devices, compare_device_data,
    };

    use super::Discovery;

    pub(crate) fn configure_discovery() -> Discovery {
        Discovery::new(DOMAIN)
            .timeout(Duration::from_secs(1))
            .disable_ipv6()
            .disable_network_interface("docker0")
    }

    async fn discovery_comparison(devices_len: usize) {
        let devices = configure_discovery().discover().await.unwrap();

        // Count devices.
        assert_eq!(devices.len(), devices_len);

        // Iterate over devices and compare data.
        for device in devices {
            compare_device_data(&device);
        }
    }

    #[inline]
    async fn run_discovery_function<F, Fut>(name: &str, function: F)
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = ()>,
    {
        if option_env!("CI").is_some() {
            warn!(
                "Skipping test on CI: {} can run only on systems that expose physical MAC addresses.",
                name
            );
        } else {
            function().await;
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[serial]
    async fn test_single_device_discovery() {
        run_discovery_function("discovery_with_single_device", || async {
            check_function_with_device(|| async {
                discovery_comparison(1).await;
            })
            .await;
        })
        .await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    #[serial]
    async fn test_more_devices_discovery() {
        run_discovery_function("discovery_with_more_devices", || async {
            check_function_with_two_devices(|| async {
                discovery_comparison(2).await;
            })
            .await;
        })
        .await;
    }
}
