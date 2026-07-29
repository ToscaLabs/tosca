use core::num::NonZero;

use alloc::boxed::Box;

use embassy_net::{IpAddress, Stack, tcp::TcpSocket};

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;

use embassy_time::{Duration, with_timeout};

use rust_mqtt::Bytes;
use rust_mqtt::buffer::AllocBuffer;
use rust_mqtt::client::event::Event;
use rust_mqtt::client::options::{ConnectOptions, PublicationOptions, TopicReference};
use rust_mqtt::client::{Client, MqttError};
use rust_mqtt::types::{MqttString, ReasonCode, TopicName};

use log::{info, warn};

use crate::error::{Error, ErrorKind};

// Timeout duration for broker operations, in seconds.
const BROKER_TIMEOUT: u64 = 2;
// Maximum packet size, in bytes, accepted from the broker.
const MAX_PACKET_SIZE: u32 = 100;
// Size of the socket transmission and reception buffers.
const BUFFER_SIZE: usize = 1024;

// MQTT client configuration:
// - `TcpSocket<'static>`: TCP connection used to communicate with the MQTT broker.
// - `AllocBuffer`: dynamically allocated storage used while encoding and decoding MQTT packets.
// - `MAX_SUBSCRIBES = 1`: allows one SUBSCRIBE request to be waiting for a response at a time.
// - `RECEIVE_MAXIMUM = 1`: allows one incoming QoS 1/2 message to be waiting for acknowledgment.
// - `SEND_MAXIMUM = 1`: allows one outgoing QoS 1/2 message to be waiting for acknowledgment.
// - `MAX_SUBSCRIPTION_IDENTIFIERS = 1`: allows one subscription identifier to be attached to a received message.
type MqttClient = Client<'static, TcpSocket<'static>, AllocBuffer, 1, 1, 1, 1>;

pub(crate) struct Mqtt {
    pub(crate) client: Mutex<CriticalSectionRawMutex, MqttClient>,
}

impl Mqtt {
    #[inline]
    pub(crate) fn new() -> Self {
        let buffer = Box::leak(Box::new(AllocBuffer));
        let client = MqttClient::new(buffer);

        Self {
            client: Mutex::new(client),
        }
    }

    #[inline]
    pub(crate) async fn connect(
        &mut self,
        stack: Stack<'static>,
        remote_endpoint: (IpAddress, u16),
    ) -> Result<(), Error> {
        let rx_buffer = Box::leak(Box::new([0u8; BUFFER_SIZE]));
        let tx_buffer = Box::leak(Box::new([0u8; BUFFER_SIZE]));

        let mut socket = TcpSocket::new(stack, &mut rx_buffer[..], &mut tx_buffer[..]);

        info!(
            "Connecting to broker socket with address `{}` on port `{}`...",
            remote_endpoint.0, remote_endpoint.1
        );

        with_timeout(
            Duration::from_secs(BROKER_TIMEOUT),
            socket.connect(remote_endpoint),
        )
        .await
        .map_err(|_| Error::new(ErrorKind::Timeout, "Broker not available"))??;

        info!("Connected to broker socket");

        // Start a fresh MQTT session without restoring previous session state,
        // and tell the broker the maximum packet size this client can receive.
        let connect_options = ConnectOptions::new()
            .clean_start()
            .maximum_packet_size(NonZero::new(MAX_PACKET_SIZE).unwrap_or(NonZero::<u32>::MAX));

        let mut client = self.client.lock().await;

        match client.connect(socket, &connect_options, None).await {
            Ok(connect_info) => {
                info!("Connected to MQTT broker: {connect_info:?}");
                Ok(())
            }
            Err(error) => {
                // A failed MQTT exchange can leave the client in a recovery-required
                // state. Abort it so a later connection attempt can reuse this wrapper.
                client.abort().await;
                Err(error.into())
            }
        }
    }

    #[inline]
    pub(crate) async fn publish(&mut self, topic: &str, payload: &[u8]) -> Result<(), Error> {
        let mqtt_topic = MqttString::from_str(topic)
            .map_err(|_| Error::new(ErrorKind::Mqtt, "Invalid MQTT topic string"))?;
        let topic_name = TopicName::new(mqtt_topic)
            .ok_or_else(|| Error::new(ErrorKind::Mqtt, "Invalid MQTT topic name"))?;

        // Require the broker to acknowledge each published message,
        // and retain the latest value so new subscribers receive it immediately.
        let publication_options = PublicationOptions::new(TopicReference::Name(topic_name))
            .at_least_once()
            .retain();

        let mut client = self.client.lock().await;

        let packet_identifier = client
            .publish(&publication_options, Bytes::from(payload))
            .await
            .map_err(<MqttError<'_> as Into<Error>>::into)?
            .ok_or_else(|| Error::new(ErrorKind::Mqtt, "Missing MQTT packet identifier"))?;

        // Keep reading MQTT events until the broker replies to this specific publish.
        // Other events are ignored, while the matching acknowledgment or rejection
        // completes the operation.
        loop {
            // Wait for the next MQTT packet, failing if the broker does not respond in time.
            let header = with_timeout(Duration::from_secs(BROKER_TIMEOUT), client.poll_header())
                .await
                .map_err(|_| Error::new(ErrorKind::Timeout, "MQTT broker response timeout"))?
                .map_err(<MqttError<'_> as Into<Error>>::into)?;

            // Decode the packet and handle only events related to this publication.
            match client
                .poll_body(header)
                .await
                .map_err(<MqttError<'_> as Into<Error>>::into)?
            {
                Event::PublishAcknowledged(ack) if ack.packet_identifier == packet_identifier => {
                    // The broker accepted the publication, but no subscriber currently
                    // matches the topic. Treat it as successful and report it as a warning.
                    if ack.reason_code == ReasonCode::NoMatchingSubscribers {
                        warn!("{}", Error::from(ack.reason_code));
                    }

                    return Ok(());
                }
                Event::PublishRejected(rejection)
                    if rejection.packet_identifier == packet_identifier =>
                {
                    // The broker explicitly rejected this publication.
                    return Err(rejection.reason_code.into());
                }
                // Ignore events unrelated to the publication we are waiting for.
                _ => {}
            }
        }
    }

    #[inline]
    pub(crate) async fn send_ping(&mut self) -> Result<(), Error> {
        let mut client = self.client.lock().await;

        client
            .ping()
            .await
            .map_err(<MqttError<'_> as Into<Error>>::into)?;

        // Keep reading MQTT events until the broker replies to the ping request.
        // Other MQTT events are ignored until the expected PINGRESP is received.
        loop {
            // Wait for the next MQTT packet, failing if the broker does not respond in time.
            let header = with_timeout(Duration::from_secs(BROKER_TIMEOUT), client.poll_header())
                .await
                .map_err(|_| Error::new(ErrorKind::Timeout, "MQTT broker ping timeout"))?
                .map_err(<MqttError<'_> as Into<Error>>::into)?;

            // Decode the packet and complete the ping only when the broker replies with PINGRESP.
            if let Event::Pingresp = client
                .poll_body(header)
                .await
                .map_err(<MqttError<'_> as Into<Error>>::into)?
            {
                return Ok(());
            }
        }
    }
}
