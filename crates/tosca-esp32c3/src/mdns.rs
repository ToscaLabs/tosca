use core::cell::OnceCell;
use core::net::{Ipv4Addr, Ipv6Addr};

use esp_hal::rng::Rng;

use embassy_executor::Spawner;

use embassy_sync::blocking_mutex::CriticalSectionMutex;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::signal::Signal;

use embassy_net::Stack;

use edge_mdns::HostAnswersMdnsHandler;
use edge_mdns::buf::VecBufAccess;
use edge_mdns::domain::base::Ttl;
use edge_mdns::host::{Host, Service, ServiceAnswers};
use edge_mdns::io::{self, IPV4_DEFAULT_SOCKET};

use edge_nal::UdpSplit;
use edge_nal_embassy::{Udp, UdpBuffers};

use log::info;

use crate::error::Result;

// Hostname
const HOSTNAME: &str = "tosca";
// Service name
const SERVICE: &str = "tosca";
// Service type
const SERVICE_TYPE: &str = "_tosca";
// Transport protocol
const TRANSPORT_PROTOCOL: &str = "_udp";
// Time-to-live for answers in seconds
const TIME_TO_LIVE: u32 = 60;

// mDNS buffer pool size
const MDNS_BUFFER_POOL_SIZE: usize = 2;
// Buffer length
const BUFFER_LENGTH: usize = 1500;
// Packet metadata length
const PACKET_METADATA_LENGTH: usize = 2;

static RNG: CriticalSectionMutex<OnceCell<Rng>> = CriticalSectionMutex::new(OnceCell::new());

/// The `mDNS-SD` discovery service.
pub struct Mdns {
    hostname: &'static str,
    service: &'static str,
    service_type: &'static str,
    time_to_live: u32,
    properties: &'static [(&'static str, &'static str)],
    rng: Rng,
}

impl Mdns {
    /// Creates the [`Mdns`] discovery service.
    #[must_use]
    pub const fn new(rng: Rng) -> Self {
        Self {
            hostname: HOSTNAME,
            service: SERVICE,
            service_type: SERVICE_TYPE,
            time_to_live: TIME_TO_LIVE,
            properties: &[],
            rng,
        }
    }

    /// Sets the service hostname.
    ///
    /// An example might be `tosca`.
    #[must_use]
    pub const fn hostname(mut self, hostname: &'static str) -> Self {
        self.hostname = hostname;
        self
    }

    /// Sets the service.
    ///
    ///
    /// The service is typically the name of the device to be discovered.
    /// i.e. device
    #[must_use]
    pub const fn service(mut self, service: &'static str) -> Self {
        self.service = service;
        self
    }

    /// Sets the service type.
    ///
    /// The service type searched by the client. i.e. _tosca
    #[must_use]
    pub const fn service_type(mut self, service_type: &'static str) -> Self {
        self.service_type = service_type;
        self
    }

    /// Time-to-live (TTL) in seconds for the discovery service responses.
    #[must_use]
    pub const fn time_to_live(mut self, seconds: u32) -> Self {
        self.time_to_live = if seconds == 0 { 1 } else { seconds };
        self
    }

    /// Sets the service properties.
    ///
    /// An example of property could be the server scheme.
    /// i.e. [("scheme", "http")]
    #[must_use]
    pub const fn properties(mut self, properties: &'static [(&'static str, &'static str)]) -> Self {
        self.properties = properties;
        self
    }

    pub(crate) fn run(
        self,
        stack: Stack<'static>,
        address: Ipv4Addr,
        port: u16,
        spawner: Spawner,
    ) -> Result<()> {
        RNG.lock(|c| _ = c.set(self.rng));

        info!(
            "About to run an mDNS responder on IPV4 address `{}`. \
             It will be accessible via `{}.local`, \
             so try to run the command `ping {}.local`.",
            address, self.hostname, self.hostname
        );

        let host = Host {
            hostname: self.hostname,
            ipv4: address,
            ipv6: Ipv6Addr::UNSPECIFIED,
            ttl: Ttl::from_secs(self.time_to_live),
        };

        info!(
            "About to run a mDNS service with name `{}` and type `{}` \
             on port `{port}`.",
            self.service, self.service_type
        );

        let service = Service {
            name: self.service,
            priority: 1,
            weight: 5,
            service: self.service_type,
            protocol: TRANSPORT_PROTOCOL,
            port,
            service_subtypes: &[],
            txt_kvs: self.properties,
        };

        spawner
            .spawn(run_mdns_task(stack, host, service))
            .map_err(core::convert::Into::into)
    }
}

#[embassy_executor::task]
async fn run_mdns_task(stack: Stack<'static>, host: Host<'static>, service: Service<'static>) {
    let (recv_buf, send_buf) = (
        VecBufAccess::<NoopRawMutex, BUFFER_LENGTH>::new(),
        VecBufAccess::<NoopRawMutex, BUFFER_LENGTH>::new(),
    );

    let buffers: UdpBuffers<
        MDNS_BUFFER_POOL_SIZE,
        BUFFER_LENGTH,
        BUFFER_LENGTH,
        PACKET_METADATA_LENGTH,
    > = UdpBuffers::new();
    let udp = Udp::new(stack, &buffers);

    let mut socket = io::bind(&udp, IPV4_DEFAULT_SOCKET, Some(Ipv4Addr::UNSPECIFIED), None)
        .await
        .expect("Impossible to create the `UDP` socket");

    let (recv, send) = socket.split();

    // A way to notify the mDNS responder that the data in `Host` has changed.
    // Not needed for this example, as the data is hard-coded.
    let signal = Signal::new();

    let mdns = io::Mdns::<NoopRawMutex, _, _, _, _>::new(
        Some(Ipv4Addr::UNSPECIFIED),
        // No IPv6 network is up and running
        None,
        recv,
        send,
        recv_buf,
        send_buf,
        |buf| {
            RNG.lock(|c| c.get().map(|r| r.clone().read(buf)));
        },
        &signal,
    );

    mdns.run(HostAnswersMdnsHandler::new(ServiceAnswers::new(
        &host, &service,
    )))
    .await
    .expect("mDNS-SD task failed");
}
