use std::net::{IpAddr, Ipv4Addr};

use mdns_sd::{IfKind, ServiceDaemon, ServiceInfo};

use tracing::info;

use crate::error::{Error, ErrorKind};

use super::ServiceConfig;

impl From<mdns_sd::Error> for Error {
    fn from(e: mdns_sd::Error) -> Self {
        Self::new(ErrorKind::Service, e.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::new(ErrorKind::NotFoundAddress, e.to_string())
    }
}

pub(crate) fn run(
    service_config: ServiceConfig<'_>,
    server_address: Ipv4Addr,
    server_port: u16,
) -> std::result::Result<(), Error> {
    // Create a new mDNS service daemon
    let mdns = ServiceDaemon::new()?;

    // Disable IPv6.
    if service_config.disable_ipv6 {
        mdns.disable_interface(IfKind::IPv6)?;
    }

    // Disable IP address.
    if let Some(ip) = service_config.disable_ip {
        mdns.disable_interface(ip)?;
    }

    // Disable network interface.
    if let Some(network_interface) = service_config.disable_network_interface {
        mdns.disable_interface(network_interface)?;
    }

    // Create a hostname.
    let hostname = format!(
        "{}.{}.",
        service_config.hostname, service_config.top_level_domain
    );

    // Create a service type.
    let service_type = format!(
        "_{}._{}.{}.",
        service_config.domain,
        service_config.transport_protocol.name(),
        service_config.top_level_domain
    );

    info!("Service instance name: {}", service_config.instance_name);
    info!("Service port: {}", server_port);
    info!("Service domain: {}", service_config.domain);
    info!(
        "Service transport protocol: {}",
        service_config.transport_protocol.name()
    );
    info!(
        "Service top-level domain: {}",
        service_config.top_level_domain
    );
    info!("Service type: {}", service_type);
    info!(
        "Device reachable at this hostname: {}:{}",
        &hostname[0..hostname.len() - 1],
        server_port
    );

    let service = ServiceInfo::new(
        // Service type
        &service_type,
        // Service instance name
        service_config.instance_name,
        // DNS hostname.
        //
        // For the same hostname in the same local network, the service resolves
        // in the same addresses. It is used for A (IPv4) and AAAA (IPv6)
        // records.
        &hostname,
        // Considered IP address which allow to reach out the service.
        IpAddr::V4(server_address),
        // Port on which the service listens to. It has to be same of the
        // server.
        server_port,
        // Service properties
        service_config.properties,
    )?
    .enable_addr_auto();

    mdns.register(service)?;

    Ok(())
}
