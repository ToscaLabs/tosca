use core::num::NonZero;

use alloc::boxed::Box;

use embassy_net::{IpAddress, Stack, tcp::TcpSocket};

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;

use embassy_time::{Duration, with_timeout};

use rust_mqtt::client::Client;
use rust_mqtt::config::SessionExpiryInterval;
use rust_mqtt::buffer::AllocBuffer;
use rust_mqtt::client::options::ConnectOptions;

use log::{info, warn};

use crate::error::{Error, ErrorKind};
use crate::mk_static;

// Timeout duration for the socket connection, in seconds.
const SOCKET_TIMEOUT: u64 = 2;
// Maximum packet size, in bytes, sent by a socket.
const MAX_PACKET_SIZE: u32 = 100;
// Size of the socket and MQTT client transmission and reception buffers.
const BUFFER_SIZE: usize = 1024;

pub(crate) struct Mqtt {
    pub(crate) client:
        Mutex<CriticalSectionRawMutex, Client<'static, TcpSocket<'static>, AllocBuffer, 1, 1, 1, 1>>,
}

impl Mqtt {
    #[inline]
    pub(crate) fn new() -> Result<Self, Error> {
        let buffer = mk_static!(AllocBuffer, AllocBuffer);
        let client = Client::<_, _, _, 1, 1, 1>::new(
            buffer,
        );

        Ok(Self {
            client: Mutex::new(client),
        })
    }

    #[inline]
    pub(crate) async fn connect(&mut self,  stack: Stack<'static>,
        remote_endpoint: (IpAddress, u16)) -> Result<(), Error> {
         let rx_buffer = Box::leak(Box::new([0u8; BUFFER_SIZE]));
        let tx_buffer = Box::leak(Box::new([0u8; BUFFER_SIZE]));

        let mut socket = TcpSocket::new(stack, &mut rx_buffer[..], &mut tx_buffer[..]);

        info!(
            "Connecting to broker socket with address `{}` on port `{}`...",
            remote_endpoint.0, remote_endpoint.1
        );

        with_timeout(
            Duration::from_secs(SOCKET_TIMEOUT),
            socket.connect(remote_endpoint),
        )
        .await
        .map_err(|_| Error::new(ErrorKind::Timeout, "Broker not available"))??;

        info!("Connected to socket!");

        let connect_options = ConnectOptions::new()
                .clean_start()
                .session_expiry_interval(SessionExpiryInterval::Seconds(5))
                .maximum_packet_size(NonZero::new(MAX_PACKET_SIZE).unwrap_or(NonZero::<u32>::MAX));

        {

        let client = self.client.lock().await;

        let connect_info = client
            .connect(
                socket,
                &connect_options,
                None
            )
            .await
            .map_err(|e| e.into())?;

            info!("Connected to server: {connect_info:?}");
            info!("{:?}", client.client_config());
            info!("{:?}", client.server_config());
            info!("{:?}", client.shared_config());
            info!("{:?}", client.session());
        }

            Ok(())
    }

    #[inline]
    pub(crate) async fn publish(&mut self, topic: &str, payload: &[u8]) -> Result<(), Error> {
        /*let Err(e) = self
            .client
            .lock()
            .await
            .send_message(topic, payload, QualityOfService::QoS1, true)
            .await
        else {
            return Ok(());
        };

        match e {
            ReasonCode::NoMatchingSubscribers
            | ReasonCode::NoSubscriptionExisted
            | ReasonCode::SharedSubscriptionNotSupported => {
                warn!("{}", Error::from(e));
                Ok(())
            }
            _ => Err(e.into()),
        }*/
        Ok(())
    }

    #[inline]
    pub(crate) async fn send_ping(&mut self) -> Result<(), Error> {
        self.client
            .lock()
            .await
            .ping()
            .await
            .map_err(core::convert::Into::into)
    }
}
