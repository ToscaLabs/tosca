use std::time::Duration;

use tosca::events::{BrokerData, Events as ToscaEvents, EventsDescription};

use rumqttc::v5::{
    AsyncClient, ConnectionError, Event, EventLoop, MqttOptions, mqttbytes::QoS,
    mqttbytes::v5::Packet,
};

use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;

use tokio_util::sync::CancellationToken;

use tracing::{error, warn};

use crate::error::Result;

// The capacity of the bounded asynchronous channel.
const ASYNC_CHANNEL_CAPACITY: usize = 10;

// Keep alive time to send `pingreq` to broker when the connection is idle.
const KEEP_ALIVE_TIME: Duration = Duration::from_secs(5);

/// Event payload transmitted by the global asynchronous receiver task.
///
/// The payload consists of a device identifier and its associated event data.
#[derive(Debug)]
pub struct EventPayload {
    /// Device identifier.
    pub device_id: usize,
    /// Device events.
    pub events: ToscaEvents,
}

impl std::fmt::Display for EventPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        writeln!(f)?;
        writeln!(f, "Events for `Device {}`", self.device_id)?;
        writeln!(f)?;
        write!(f, "{}", self.events)
    }
}

impl EventPayload {
    pub(crate) const fn new(device_id: usize, events: ToscaEvents) -> Self {
        Self { device_id, events }
    }
}

#[derive(Debug)]
pub(crate) struct Events {
    // Events description.
    pub(crate) description: EventsDescription,
    // The token used to cancel the event task.
    pub(crate) cancellation_token: CancellationToken,
}

impl Events {
    pub(crate) fn new(description: EventsDescription) -> Self {
        Self {
            description,
            cancellation_token: CancellationToken::new(),
        }
    }
}

#[inline]
fn parse_event(event: &std::result::Result<Event, ConnectionError>) -> Option<ToscaEvents> {
    let event = match event {
        Ok(event) => event,
        Err(e) => {
            error!("Error in receiving the event, discard it: {e}");
            return None;
        }
    };

    let packet = match event {
        Event::Incoming(packet) => packet,
        Event::Outgoing(outgoing) => {
            warn!("Outgoing packet, discard it: {:?}", outgoing);
            return None;
        }
    };

    let Packet::Publish(packet) = packet else {
        warn!("Packet ignored: {:?}", packet);
        return None;
    };

    match serde_json::from_slice(&packet.payload) {
        Ok(tosca_events) => tosca_events,
        Err(e) => {
            error!("Error converting packet bytes into events: {e}");
            None
        }
    }
}

async fn run_global_event_subscriber(
    client: AsyncClient,
    mut eventloop: EventLoop,
    id: usize,
    cancellation_token: CancellationToken,
    sender: mpsc::Sender<EventPayload>,
) {
    loop {
        tokio::select! {
            // Use the cancellation token to stop the loop
            () = cancellation_token.cancelled() => { break; }
            // Poll the `MQTT` event coming from the network
            event = eventloop.poll() => {
                let Some(tosca_events) = parse_event(&event) else {
                    continue;
                };

                if let Err(e) = sender.send(EventPayload::new(id, tosca_events)).await {
                    error!(
                        "Stop sending events to the global receiver: {e}"
                    );
                    break;
                }
            }
        }
    }
    drop(sender);
    drop(eventloop);
    drop(client);
}

async fn run_event_subscriber(
    client: AsyncClient,
    mut eventloop: EventLoop,
    id: usize,
    cancellation_token: CancellationToken,
    sender: broadcast::Sender<ToscaEvents>,
) {
    loop {
        tokio::select! {
            // Use the cancellation token to stop the loop
            () = cancellation_token.cancelled() => { break; }
            // Poll the `MQTT` event coming from the network
            event = eventloop.poll() => {
                let Some(tosca_events) = parse_event(&event) else {
                    continue;
                };

                if let Err(e) = sender.send(tosca_events) {
                    error!(
                        "Stop sending events to the device receiver with id `{id}`: {e}"
                    );
                    break;
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    drop(sender);
    drop(eventloop);
    drop(client);
}

pub(crate) struct EventsRunner;

impl EventsRunner {
    pub(crate) async fn run_global_subscriber(
        events: &Events,
        id: usize,
        sender: mpsc::Sender<EventPayload>,
    ) -> Result<JoinHandle<()>> {
        let (client, eventloop) = Self::init(id, events).await?;

        Ok(tokio::spawn(run_global_event_subscriber(
            client,
            eventloop,
            id,
            events.cancellation_token.clone(),
            sender,
        )))
    }

    pub(crate) async fn run_device_subscriber(
        events: &Events,
        id: usize,
        sender: broadcast::Sender<ToscaEvents>,
    ) -> Result<JoinHandle<()>> {
        let (client, eventloop) = Self::init(id, events).await?;

        Ok(tokio::spawn(run_event_subscriber(
            client,
            eventloop,
            id,
            events.cancellation_token.clone(),
            sender,
        )))
    }

    #[inline]
    async fn init(id: usize, events: &Events) -> Result<(AsyncClient, EventLoop)> {
        let BrokerData { address, port } = events.description.broker_data;
        let topic = events.description.topic.as_str();

        let mut mqttoptions = MqttOptions::new(id.to_string(), address.to_string(), port);
        mqttoptions.set_keep_alive(KEEP_ALIVE_TIME);

        let (client, eventloop) = AsyncClient::new(mqttoptions, ASYNC_CHANNEL_CAPACITY);
        client
            .subscribe(topic, QoS::AtMostOnce)
            .await
            .map_err(|e| {
                error!("Impossible to subscribe to topic {topic} for device {id}: {e}");
                e
            })?;

        Ok((client, eventloop))
    }
}
