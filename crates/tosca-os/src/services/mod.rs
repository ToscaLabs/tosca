mod mdns_sd;

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};

use crate::error::Result;

use mdns_sd::run;

// Service domain.
//
// It defines the default domain for a service.
const DOMAIN: &str = "tosca";

// Service top-level domain.
//
// It defines the default top-level domain for a service.
const TOP_LEVEL_DOMAIN: &str = "local";

/// The discovery service transport protocol.
#[derive(Debug, Clone, Copy)]
pub enum TransportProtocol {
    /// TCP-based service.
    TCP,
    /// UDP-based service.
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

/// A discovery service configuration.
#[derive(Debug)]
pub struct ServiceConfig<'a> {
    // Instance name.
    pub(crate) instance_name: &'a str,
    // Service host name
    pub(crate) hostname: &'a str,
    // Service domain.
    pub(crate) domain: &'a str,
    // Service transport protocol.
    pub(crate) transport_protocol: TransportProtocol,
    // Top-level domain.
    pub(crate) top_level_domain: &'a str,
    // Service properties.
    pub(crate) properties: HashMap<String, String>,
    // Disable IPv6.
    pub(crate) disable_ipv6: bool,
    // Disable IP.
    pub(crate) disable_ip: Option<IpAddr>,
    // Disable network interface.
    pub(crate) disable_network_interface: Option<&'a str>,
}

impl<'a> ServiceConfig<'a> {
    /// Creates [`ServiceConfig`] for an `mDNS-SD` discovery service.
    #[must_use]
    pub fn mdns_sd(instance_name: &'a str) -> Self {
        Self {
            instance_name,
            hostname: instance_name,
            domain: DOMAIN,
            transport_protocol: TransportProtocol::TCP,
            top_level_domain: TOP_LEVEL_DOMAIN,
            properties: HashMap::new(),
            disable_ipv6: false,
            disable_ip: None,
            disable_network_interface: None,
        }
    }

    /// Sets a discovery service property.
    ///
    /// An example of property could be the server scheme.
    /// i.e. ("scheme", "http")
    #[must_use]
    pub fn property(mut self, property: (impl Into<String>, impl Into<String>)) -> Self {
        self.properties.insert(property.0.into(), property.1.into());
        self
    }

    /// Sets the service hostname.
    ///
    /// An example might be `tosca`.
    #[must_use]
    pub const fn hostname(mut self, hostname: &'a str) -> Self {
        self.hostname = hostname;
        self
    }

    /// Sets the service transport protocol.
    #[must_use]
    pub const fn transport_protocol(mut self, transport_protocol: TransportProtocol) -> Self {
        self.transport_protocol = transport_protocol;
        self
    }

    /// Sets the service domain.
    ///
    ///
    /// The domain searched by the client service. i.e. tosca
    #[must_use]
    pub const fn domain(mut self, domain: &'a str) -> Self {
        self.domain = domain;
        self
    }

    /// Sets the service top-level domain.
    ///
    /// A common top-level domain is `.local`.
    #[must_use]
    pub const fn top_level_domain(mut self, top_level_domain: &'a str) -> Self {
        self.top_level_domain = top_level_domain;
        self
    }

    /// Excludes devices with `IPv6` interfaces from the discovery service.
    #[must_use]
    pub const fn disable_ipv6(mut self) -> Self {
        self.disable_ipv6 = true;
        self
    }

    /// Excludes the device with the given `IP` from the discovery service.
    #[must_use]
    #[inline]
    pub fn disable_ip(mut self, ip: impl Into<IpAddr>) -> Self {
        self.disable_ip = Some(ip.into());
        self
    }

    /// Disables the given network interface from the discovery service.
    #[must_use]
    pub const fn disable_network_interface(mut self, network_interface: &'a str) -> Self {
        self.disable_network_interface = Some(network_interface);
        self
    }
}

// A new service.
pub(crate) struct Service;

impl Service {
    // Runs a service.
    #[inline]
    pub(crate) fn run(
        service_config: ServiceConfig,
        server_address: Ipv4Addr,
        port: u16,
    ) -> Result<()> {
        run(service_config, server_address, port)
    }
}
