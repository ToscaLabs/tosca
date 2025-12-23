mod mqtt;
mod topic;

/// All essential data needed to configure an event broker.
pub mod broker;
/// A set of notifiers designed to manage interrupt events.
pub mod interrupt;
/// A set of notifiers designed to manage periodic events.
pub mod periodic;

use core::net::IpAddr;
use core::time::Duration;

use alloc::boxed::Box;

use embassy_executor::{SpawnToken, Spawner};
use embassy_net::{IpAddress, Stack, dns::DnsQueryType};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use embassy_time::Timer;

use esp_hal::gpio::AnyPin;

use log::{error, info};

use tosca::device::DeviceKind;
use tosca::events::{
    BrokerData as ToscaBrokerData, Event, Events, EventsDescription, PeriodicEvent, Topic,
};

use crate::device::Device;
use crate::error::{Error, ErrorKind};
use crate::state::ValueFromRef;
use crate::wifi::WIFI_RECONNECT_DELAY;

use broker::BrokerData;
use mqtt::Mqtt;
use topic::TopicBuilder;

use super::events::interrupt::{
    Notifier,
    bool::{BoolFn, BoolFnPinless, monitor_bool_event, monitor_bool_event_pinless},
    f32::{F32Fn, F32FnPinless, monitor_f32_event, monitor_f32_event_pinless},
    f64::{F64Fn, F64FnPinless, monitor_f64_event, monitor_f64_event_pinless},
    i32::{I32Fn, I32FnPinless, monitor_i32_event, monitor_i32_event_pinless},
    u8::{U8Fn, U8FnPinless, monitor_u8_event, monitor_u8_event_pinless},
};
use super::events::periodic::{
    PeriodicNotifier,
    bool::{
        PeriodicBoolFn, PeriodicBoolFnPinless, monitor_periodic_bool_event,
        monitor_periodic_bool_event_pinless,
    },
    f32::{
        PeriodicF32Fn, PeriodicF32FnPinless, monitor_periodic_f32_event,
        monitor_periodic_f32_event_pinless,
    },
    f64::{
        PeriodicF64Fn, PeriodicF64FnPinless, monitor_periodic_f64_event,
        monitor_periodic_f64_event_pinless,
    },
    i32::{
        PeriodicI32Fn, PeriodicI32FnPinless, monitor_periodic_i32_event,
        monitor_periodic_i32_event_pinless,
    },
    u8::{
        PeriodicU8Fn, PeriodicU8FnPinless, monitor_periodic_u8_event,
        monitor_periodic_u8_event_pinless,
    },
};

// Internal array capacity
const CAPACITY: usize = 4;

// Time to wait, in milliseconds, after completing a task operation
const WAIT_FOR_MILLISECONDS: u64 = 200;

// Time to wait, in seconds, before reconnecting to the broker
const RETRY_INTERVAL: u64 = 120;

// Time, in seconds, to wait before starting the network write task.
//
// To be on the safe side, this value should be twice the WiFi reconnection
// interval (in seconds), plus an additional two
const LOWER_PRIORITY: u64 = (WIFI_RECONNECT_DELAY * 2) + 2;

// Time, in seconds, to wait before pinging the broker again to check if the
// connection is still active
const PING_BROKER_AGAIN: u64 = 10;

// All events to be transmitted over the network
static EVENTS: Mutex<CriticalSectionRawMutex, Events> = Mutex::new(Events::empty());
// Signal that enables network transmission
static WRITE_ON_NETWORK: Signal<CriticalSectionRawMutex, u8> = Signal::new();

/// Events configuration.
///
/// It defines all the data necessary to execute the events.
pub struct EventsConfig<S>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    spawner: Spawner,
    stack: Stack<'static>,
    broker: BrokerData,
    topic: Topic,
    device: Device<S>,
}

impl<S> EventsConfig<S>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    /// Creates a new [`EventsConfig`].
    #[inline]
    #[must_use]
    pub fn new(
        spawner: Spawner,
        stack: Stack<'static>,
        broker: BrokerData,
        device: Device<S>,
    ) -> Self {
        Self {
            spawner,
            stack,
            broker,
            topic: match device.description.kind {
                DeviceKind::Light => TopicBuilder::new().prefix("light"),
                _ => TopicBuilder::new().prefix("unknown"),
            }
            .suffix("events")
            .mac(device.wifi_mac)
            .build(),
            device,
        }
    }
}

#[embassy_executor::task]
async fn write_on_network(stack: Stack<'static>, remote_endpoint: (IpAddress, u16), topic: Topic) {
    // This task is scheduled to run last, so it is assigned a lower priority.
    Timer::after_secs(LOWER_PRIORITY).await;

    let mut mqtt_publisher = loop {
        // Create a `MQTT` publisher.
        //
        // If an error occurs, retry creation after a specified time interval.
        match Mqtt::new(stack, remote_endpoint).await {
            Ok(mqtt_publisher) => {
                info!("Created the `MQTT` publisher");
                break mqtt_publisher;
            }
            Err(e) => {
                error!("Error while creating the `MQTT` publisher: {e}");
            }
        }
        Timer::after_secs(RETRY_INTERVAL).await;
    };

    loop {
        // Connect to the broker.
        //
        // If an error occurs, retry the connection after
        // a specified time interval.
        match mqtt_publisher.connect().await {
            Ok(()) => {
                info!("`MQTT` publisher connected to the broker");
                break;
            }
            Err(e) => {
                error!("Error while connecting the `MQTT` publisher to the broker: {e}");
            }
        }
        Timer::after_secs(RETRY_INTERVAL).await;
    }

    // Count the number of ping failures
    let mut ping_failure_counter: u8 = 0;
    loop {
        // Ping the broker to check if it is still alive
        if let Err(e) = mqtt_publisher.send_ping().await {
            error!("Error while pinging the `MQTT` broker: {e}");

            // After five consecutive ping failures, reinitialize the `MQTT`
            // publisher, as the socket may have been closed.
            if ping_failure_counter == 5 {
                mqtt_publisher = match Mqtt::new(stack, remote_endpoint).await {
                    Ok(mqtt_publisher) => {
                        info!("Reinitialize the `MQTT` publisher");
                        mqtt_publisher
                    }
                    Err(e) => {
                        error!("Error while reinitializing the `MQTT` publisher: {e}");
                        Timer::after_secs(RETRY_INTERVAL).await;
                        continue;
                    }
                };

                match mqtt_publisher.connect().await {
                    Ok(()) => {
                        info!("`MQTT` publisher reconnected to the broker");
                    }
                    Err(e) => {
                        error!("Error while reconnecting the `MQTT` publisher to broker: {e}");
                        Timer::after_secs(PING_BROKER_AGAIN).await;
                        continue;
                    }
                }
                ping_failure_counter = 0;
                continue;
            }
            ping_failure_counter += 1;
            Timer::after_secs(PING_BROKER_AGAIN).await;
            continue;
        }

        // The lock will be released at the end of this scope.
        {
            // Wait until a signal is received.
            let _ = WRITE_ON_NETWORK.wait().await;
        }
        // The lock will be released at the end of this scope,
        // once the JSON data has been retrieved.
        let json_data = { serde_json::to_vec(&*EVENTS.lock().await) };

        // Serialize data
        let data = match json_data {
            Ok(data) => data,
            Err(e) => {
                error!("Error retrieving data: {e}");
                continue;
            }
        };

        info!("Data capacity: {} bytes", data.capacity());

        // Transmit the data over the network.
        //
        // Skip the operation if any subscriber errors are detected, and issue
        // a warning
        if let Err(e) = mqtt_publisher.publish(topic.as_str(), &data).await {
            error!("Error while publishing data over the network: {e}");
        }

        // Wait briefly after transmitting data over the network
        Timer::after_millis(WAIT_FOR_MILLISECONDS).await;
    }
}

/// An events manager.
///
/// It checks whether events are correct and runs them.
pub struct EventsManager<S>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    config: EventsConfig<S>,
    events: Events,
}

impl<S> EventsManager<S>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    /// Configures the [`EventsManager`].
    ///
    /// It allocates its internal structures with a fixed amount of memory.
    #[inline]
    #[must_use]
    pub fn config(config: EventsConfig<S>) -> Self {
        Self {
            config,
            events: Events::with_capacity(CAPACITY),
        }
    }

    /// Monitors a pin with an [`Event<bool>`] notifier.
    ///
    /// Discard the event if it matches another.
    #[inline]
    #[must_use]
    pub fn bool_event<F, Fut>(
        self,
        name: &'static str,
        description: &'static str,
        func: F,
        pin: AnyPin<'static>,
    ) -> Self
    where
        F: Fn(AnyPin<'static>, Notifier<bool>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let events_ref = self.events.bool_events_as_slice();
        let len = events_ref.len();

        for value in events_ref {
            if value.name == name {
                info!(
                    "The event `{}` is equal to `{}`, discard it.",
                    value.name, name
                );
                return self;
            }
        }

        let event = Event::bool(name).description(description);
        let bool_notifier = Notifier::bool(len);
        // We need to do this because embassy tasks do not support generics.
        let func: BoolFn = Box::new(move |pin, bool_notifier| Box::pin(func(pin, bool_notifier)));
        let task = monitor_bool_event(event, pin, bool_notifier, func);

        self.spawn(name, task, |events| events.add_bool_event(event))
    }

    /// Monitors an [`Event<bool>`] notifier without requiring a pin.
    ///
    /// Discard the event if it matches another.
    #[inline]
    #[must_use]
    pub fn bool_event_pinless<F, Fut>(
        self,
        name: &'static str,
        description: &'static str,
        func: F,
    ) -> Self
    where
        F: Fn(Notifier<bool>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let events_ref = self.events.bool_events_as_slice();
        let len = events_ref.len();

        for value in events_ref {
            if value.name == name {
                info!(
                    "The event `{}` is equal to `{}`, discard it.",
                    value.name, name
                );
                return self;
            }
        }

        let event = Event::bool(name).description(description);
        let bool_notifier = Notifier::bool(len);
        // We need to do this because embassy tasks do not support generics.
        let func: BoolFnPinless = Box::new(move |bool_notifier| Box::pin(func(bool_notifier)));
        let task = monitor_bool_event_pinless(event, bool_notifier, func);

        self.spawn(name, task, |events| events.add_bool_event(event))
    }

    /// Monitors a pin with a [`PeriodicEvent<bool>`] notifier.
    ///
    /// Discard the event if it matches another.
    #[inline]
    #[must_use]
    pub fn periodic_bool<F, Fut>(
        self,
        name: &'static str,
        description: &'static str,
        interval: Duration,
        func: F,
        pin: AnyPin<'static>,
    ) -> Self
    where
        F: Fn(AnyPin<'static>, PeriodicNotifier<bool>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let events_ref = self.events.periodic_bool_events_as_slice();
        let len = events_ref.len();
        for value in events_ref {
            if value.event.name == name {
                info!(
                    "The event `{}` is equal to `{}`, discard it.",
                    value.event.name, name
                );
                return self;
            }
        }

        let event = PeriodicEvent::bool(Event::bool(name).description(description), interval);
        let periodic_bool_notifier = PeriodicNotifier::bool(len, interval);
        // We need to do this because embassy tasks do not support generics.
        let func: PeriodicBoolFn =
            Box::new(move |pin, bool_notifier| Box::pin(func(pin, bool_notifier)));
        let task = monitor_periodic_bool_event(event, pin, periodic_bool_notifier, func);

        self.spawn(name, task, |events| events.add_periodic_bool_event(event))
    }

    /// Monitors a [`PeriodicEvent<bool>`] notifier without requiring a pin.
    ///
    /// Discard the event if it matches another.
    #[inline]
    #[must_use]
    pub fn periodic_bool_pinless<F, Fut>(
        self,
        name: &'static str,
        description: &'static str,
        interval: Duration,
        func: F,
    ) -> Self
    where
        F: Fn(PeriodicNotifier<bool>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let events_ref = self.events.periodic_bool_events_as_slice();
        let len = events_ref.len();

        for value in events_ref {
            if value.event.name == name {
                info!(
                    "The event `{}` is equal to `{}`, discard it.",
                    value.event.name, name
                );
                return self;
            }
        }

        let event = PeriodicEvent::bool(Event::bool(name).description(description), interval);
        let periodic_bool_notifier = PeriodicNotifier::bool(len, interval);
        // We need to do this because embassy tasks do not support generics.
        let func: PeriodicBoolFnPinless = Box::new(move |notifier| Box::pin(func(notifier)));
        let task = monitor_periodic_bool_event_pinless(event, periodic_bool_notifier, func);

        self.spawn(name, task, |events| events.add_periodic_bool_event(event))
    }

    /// Monitors a pin with an [`Event<u8>`] notifier.
    ///
    /// Discard the event if it matches another.
    #[inline]
    #[must_use]
    pub fn u8_event<F, Fut>(
        self,
        name: &'static str,
        description: &'static str,
        func: F,
        pin: AnyPin<'static>,
    ) -> Self
    where
        F: Fn(AnyPin<'static>, Notifier<u8>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let events_ref = self.events.u8_events_as_slice();
        let len = events_ref.len();

        for value in events_ref {
            if value.name == name {
                info!(
                    "The event `{}` is equal to `{}`, discard it.",
                    value.name, name
                );
                return self;
            }
        }

        let event = Event::u8(name).description(description);
        let u8_notifier = Notifier::u8(len);
        // We need to do this because embassy tasks do not support generics.
        let func: U8Fn = Box::new(move |pin, u8_notifier| Box::pin(func(pin, u8_notifier)));
        let task = monitor_u8_event(event, pin, u8_notifier, func);

        self.spawn(name, task, |events| events.add_u8_event(event))
    }

    /// Monitors an [`Event<u8>`] notifier without requiring a pin.
    ///
    /// Discard the event if it matches another.
    #[inline]
    #[must_use]
    pub fn u8_event_pinless<F, Fut>(
        self,
        name: &'static str,
        description: &'static str,
        func: F,
    ) -> Self
    where
        F: Fn(Notifier<u8>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let events_ref = self.events.u8_events_as_slice();
        let len = events_ref.len();

        for value in events_ref {
            if value.name == name {
                info!(
                    "The event `{}` is equal to `{}`, discard it.",
                    value.name, name
                );
                return self;
            }
        }

        let event = Event::u8(name).description(description);
        let u8_notifier = Notifier::u8(len);
        // We need to do this because embassy tasks do not support generics.
        let func: U8FnPinless = Box::new(move |u8_notifier| Box::pin(func(u8_notifier)));
        let task = monitor_u8_event_pinless(event, u8_notifier, func);

        self.spawn(name, task, |events| events.add_u8_event(event))
    }

    /// Monitors a pin with a [`PeriodicEvent<u8>`] notifier.
    ///
    /// Discard the event if it matches another.
    #[inline]
    #[must_use]
    pub fn periodic_u8<F, Fut>(
        self,
        name: &'static str,
        description: &'static str,
        interval: Duration,
        func: F,
        pin: AnyPin<'static>,
    ) -> Self
    where
        F: Fn(AnyPin<'static>, PeriodicNotifier<u8>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let events_ref = self.events.periodic_u8_events_as_slice();
        let len = events_ref.len();

        for value in events_ref {
            if value.event.name == name {
                info!(
                    "The event `{}` is equal to `{}`, discard it.",
                    value.event.name, name
                );
                return self;
            }
        }

        let event = PeriodicEvent::u8(Event::u8(name).description(description), interval);
        let periodic_u8_notifier = PeriodicNotifier::u8(len, interval);
        // We need to do this because embassy tasks do not support generics.
        let func: PeriodicU8Fn = Box::new(move |pin, u8_notifier| Box::pin(func(pin, u8_notifier)));
        let task = monitor_periodic_u8_event(event, pin, periodic_u8_notifier, func);

        self.spawn(name, task, |events| events.add_periodic_u8_event(event))
    }

    /// Monitors a [`PeriodicEvent<u8>`] notifier without requiring a pin.
    ///
    /// Discard the event if it matches another.
    #[inline]
    #[must_use]
    pub fn periodic_u8_pinless<F, Fut>(
        self,
        name: &'static str,
        description: &'static str,
        interval: Duration,
        func: F,
    ) -> Self
    where
        F: Fn(PeriodicNotifier<u8>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let events_ref = self.events.periodic_u8_events_as_slice();
        let len = events_ref.len();

        for value in events_ref {
            if value.event.name == name {
                info!(
                    "The event `{}` is equal to `{}`, discard it.",
                    value.event.name, name
                );
                return self;
            }
        }

        let event = PeriodicEvent::u8(Event::u8(name).description(description), interval);
        let periodic_u8_notifier = PeriodicNotifier::u8(len, interval);
        // We need to do this because embassy tasks do not support generics.
        let func: PeriodicU8FnPinless = Box::new(move |notifier| Box::pin(func(notifier)));
        let task = monitor_periodic_u8_event_pinless(event, periodic_u8_notifier, func);

        self.spawn(name, task, |events| events.add_periodic_u8_event(event))
    }

    /// Monitors a pin with an [`Event<i32>`] notifier.
    ///
    /// Discard the event if it matches another.
    #[inline]
    #[must_use]
    pub fn i32_event<F, Fut>(
        self,
        name: &'static str,
        description: &'static str,
        func: F,
        pin: AnyPin<'static>,
    ) -> Self
    where
        F: Fn(AnyPin<'static>, Notifier<i32>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let events_ref = self.events.i32_events_as_slice();
        let len = events_ref.len();

        for value in events_ref {
            if value.name == name {
                info!(
                    "The event `{}` is equal to `{}`, discard it.",
                    value.name, name
                );
                return self;
            }
        }

        let event = Event::i32(name).description(description);
        let i32_notifier = Notifier::i32(len);
        // We need to do this because embassy tasks do not support generics.
        let func: I32Fn = Box::new(move |pin, i32_notifier| Box::pin(func(pin, i32_notifier)));
        let task = monitor_i32_event(event, pin, i32_notifier, func);

        self.spawn(name, task, |events| events.add_i32_event(event))
    }

    /// Monitors an [`Event<i32>`] notifier without requiring a pin.
    ///
    /// Discard the event if it matches another.
    #[inline]
    #[must_use]
    pub fn i32_event_pinless<F, Fut>(
        self,
        name: &'static str,
        description: &'static str,
        func: F,
    ) -> Self
    where
        F: Fn(Notifier<i32>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let events_ref = self.events.i32_events_as_slice();
        let len = events_ref.len();

        for value in events_ref {
            if value.name == name {
                info!(
                    "The event `{}` is equal to `{}`, discard it.",
                    value.name, name
                );
                return self;
            }
        }

        let event = Event::i32(name).description(description);
        let i32_notifier = Notifier::i32(len);
        // We need to do this because embassy tasks do not support generics.
        let func: I32FnPinless = Box::new(move |i32_notifier| Box::pin(func(i32_notifier)));
        let task = monitor_i32_event_pinless(event, i32_notifier, func);

        self.spawn(name, task, |events| events.add_i32_event(event))
    }

    /// Monitors a pin with a [`PeriodicEvent<i32>`] notifier.
    ///
    /// Discard the event if it matches another.
    #[inline]
    #[must_use]
    pub fn periodic_i32<F, Fut>(
        self,
        name: &'static str,
        description: &'static str,
        interval: Duration,
        func: F,
        pin: AnyPin<'static>,
    ) -> Self
    where
        F: Fn(AnyPin<'static>, PeriodicNotifier<i32>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let events_ref = self.events.periodic_i32_events_as_slice();
        let len = events_ref.len();

        for value in events_ref {
            if value.event.name == name {
                info!(
                    "The event `{}` is equal to `{}`, discard it.",
                    value.event.name, name
                );
                return self;
            }
        }

        let event = PeriodicEvent::i32(Event::i32(name).description(description), interval);
        let periodic_i32_notifier = PeriodicNotifier::i32(len, interval);
        // We need to do this because embassy tasks do not support generics.
        let func: PeriodicI32Fn =
            Box::new(move |pin, i32_notifier| Box::pin(func(pin, i32_notifier)));
        let task = monitor_periodic_i32_event(event, pin, periodic_i32_notifier, func);

        self.spawn(name, task, |events| events.add_periodic_i32_event(event))
    }

    /// Monitors a [`PeriodicEvent<i32>`] notifier without requiring a pin.
    ///
    /// Discard the event if it matches another.
    #[inline]
    #[must_use]
    pub fn periodic_i32_pinless<F, Fut>(
        self,
        name: &'static str,
        description: &'static str,
        interval: Duration,
        func: F,
    ) -> Self
    where
        F: Fn(PeriodicNotifier<i32>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let events_ref = self.events.periodic_i32_events_as_slice();
        let len = events_ref.len();

        for value in events_ref {
            if value.event.name == name {
                info!(
                    "The event `{}` is equal to `{}`, discard it.",
                    value.event.name, name
                );
                return self;
            }
        }

        let event = PeriodicEvent::i32(Event::i32(name).description(description), interval);
        let periodic_i32_notifier = PeriodicNotifier::i32(len, interval);
        // We need to do this because embassy tasks do not support generics.
        let func: PeriodicI32FnPinless = Box::new(move |notifier| Box::pin(func(notifier)));
        let task = monitor_periodic_i32_event_pinless(event, periodic_i32_notifier, func);

        self.spawn(name, task, |events| events.add_periodic_i32_event(event))
    }

    /// Monitors a pin with an [`Event<f32>`] notifier.
    ///
    /// Discard the event if it matches another.
    #[inline]
    #[must_use]
    pub fn f32_event<F, Fut>(
        self,
        name: &'static str,
        description: &'static str,
        func: F,
        pin: AnyPin<'static>,
    ) -> Self
    where
        F: Fn(AnyPin<'static>, Notifier<f32>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let events_ref = self.events.f32_events_as_slice();
        let len = events_ref.len();

        for value in events_ref {
            if value.name == name {
                info!(
                    "The event `{}` is equal to `{}`, discard it.",
                    value.name, name
                );
                return self;
            }
        }

        let event = Event::f32(name).description(description);
        let f32_notifier = Notifier::f32(len);
        // We need to do this because embassy tasks do not support generics.
        let func: F32Fn = Box::new(move |pin, f32_notifier| Box::pin(func(pin, f32_notifier)));
        let task = monitor_f32_event(event, pin, f32_notifier, func);

        self.spawn(name, task, |events| events.add_f32_event(event))
    }

    /// Monitors an [`Event<f32>`] notifier without requiring a pin.
    ///
    /// Discard the event if it matches another.
    #[inline]
    #[must_use]
    pub fn f32_event_pinless<F, Fut>(
        self,
        name: &'static str,
        description: &'static str,
        func: F,
    ) -> Self
    where
        F: Fn(Notifier<f32>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let events_ref = self.events.f32_events_as_slice();
        let len = events_ref.len();

        for value in events_ref {
            if value.name == name {
                info!(
                    "The event `{}` is equal to `{}`, discard it.",
                    value.name, name
                );
                return self;
            }
        }

        let event = Event::f32(name).description(description);
        let f32_notifier = Notifier::f32(len);
        // We need to do this because embassy tasks do not support generics.
        let func: F32FnPinless = Box::new(move |f32_notifier| Box::pin(func(f32_notifier)));
        let task = monitor_f32_event_pinless(event, f32_notifier, func);

        self.spawn(name, task, |events| events.add_f32_event(event))
    }

    /// Monitors a pin with a [`PeriodicEvent<f32>`] notifier.
    ///
    /// Discard the event if it matches another.
    #[inline]
    #[must_use]
    pub fn periodic_f32<F, Fut>(
        self,
        name: &'static str,
        description: &'static str,
        interval: Duration,
        func: F,
        pin: AnyPin<'static>,
    ) -> Self
    where
        F: Fn(AnyPin<'static>, PeriodicNotifier<f32>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let events_ref = self.events.periodic_f32_events_as_slice();
        let len = events_ref.len();

        for value in events_ref {
            if value.event.name == name {
                info!(
                    "The event `{}` is equal to `{}`, discard it.",
                    value.event.name, name
                );
                return self;
            }
        }

        let event = PeriodicEvent::f32(Event::f32(name).description(description), interval);
        let periodic_f32_notifier = PeriodicNotifier::f32(len, interval);
        // We need to do this because embassy tasks do not support generics.
        let func: PeriodicF32Fn =
            Box::new(move |pin, f32_notifier| Box::pin(func(pin, f32_notifier)));
        let task = monitor_periodic_f32_event(event, pin, periodic_f32_notifier, func);

        self.spawn(name, task, |events| events.add_periodic_f32_event(event))
    }

    /// Monitors a [`PeriodicEvent<f32>`] notifier without requiring a pin.
    ///
    /// Discard the event if it matches another.
    #[inline]
    #[must_use]
    pub fn periodic_f32_pinless<F, Fut>(
        self,
        name: &'static str,
        description: &'static str,
        interval: Duration,
        func: F,
    ) -> Self
    where
        F: Fn(PeriodicNotifier<f32>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let events_ref = self.events.periodic_f32_events_as_slice();
        let len = events_ref.len();

        for value in events_ref {
            if value.event.name == name {
                info!(
                    "The event `{}` is equal to `{}`, discard it.",
                    value.event.name, name
                );
                return self;
            }
        }

        let event = PeriodicEvent::f32(Event::f32(name).description(description), interval);
        let periodic_f32_notifier = PeriodicNotifier::f32(len, interval);
        // We need to do this because embassy tasks do not support generics.
        let func: PeriodicF32FnPinless = Box::new(move |notifier| Box::pin(func(notifier)));
        let task = monitor_periodic_f32_event_pinless(event, periodic_f32_notifier, func);

        self.spawn(name, task, |events| events.add_periodic_f32_event(event))
    }

    /// Monitors a pin with an [`Event<f64>`] notifier.
    ///
    /// Discard the event if it matches another.
    #[inline]
    #[must_use]
    pub fn f64_event<F, Fut>(
        self,
        name: &'static str,
        description: &'static str,
        func: F,
        pin: AnyPin<'static>,
    ) -> Self
    where
        F: Fn(AnyPin<'static>, Notifier<f64>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let events_ref = self.events.f64_events_as_slice();
        let len = events_ref.len();

        for value in events_ref {
            if value.name == name {
                info!(
                    "The event `{}` is equal to `{}`, discard it.",
                    value.name, name
                );
                return self;
            }
        }

        let event = Event::f64(name).description(description);
        let f64_notifier = Notifier::f64(len);
        // We need to do this because embassy tasks do not support generics.
        let func: F64Fn = Box::new(move |pin, f64_notifier| Box::pin(func(pin, f64_notifier)));
        let task = monitor_f64_event(event, pin, f64_notifier, func);

        self.spawn(name, task, |events| events.add_f64_event(event))
    }

    /// Monitors an [`Event<f64>`] notifier without requiring a pin.
    ///
    /// Discard the event if it matches another.
    #[inline]
    #[must_use]
    pub fn f64_event_pinless<F, Fut>(
        self,
        name: &'static str,
        description: &'static str,
        func: F,
    ) -> Self
    where
        F: Fn(Notifier<f64>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let events_ref = self.events.f64_events_as_slice();
        let len = events_ref.len();

        for value in events_ref {
            if value.name == name {
                info!(
                    "The event `{}` is equal to `{}`, discard it.",
                    value.name, name
                );
                return self;
            }
        }

        let event = Event::f64(name).description(description);
        let f64_notifier = Notifier::f64(len);
        // We need to do this because embassy tasks do not support generics.
        let func: F64FnPinless = Box::new(move |f64_notifier| Box::pin(func(f64_notifier)));
        let task = monitor_f64_event_pinless(event, f64_notifier, func);

        self.spawn(name, task, |events| events.add_f64_event(event))
    }

    /// Monitors a pin with a [`PeriodicEvent<f64>`] notifier.
    ///
    /// Discard the event if it matches another.
    #[inline]
    #[must_use]
    pub fn periodic_f64<F, Fut>(
        self,
        name: &'static str,
        description: &'static str,
        interval: Duration,
        func: F,
        pin: AnyPin<'static>,
    ) -> Self
    where
        F: Fn(AnyPin<'static>, PeriodicNotifier<f64>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let events_ref = self.events.periodic_f64_events_as_slice();
        let len = events_ref.len();

        for value in events_ref {
            if value.event.name == name {
                info!(
                    "The event `{}` is equal to `{}`, discard it.",
                    value.event.name, name
                );
                return self;
            }
        }

        let event = PeriodicEvent::f64(Event::f64(name).description(description), interval);
        let periodic_f64_notifier = PeriodicNotifier::f64(len, interval);
        // We need to do this because embassy tasks do not support generics.
        let func: PeriodicF64Fn =
            Box::new(move |pin, f64_notifier| Box::pin(func(pin, f64_notifier)));
        let task = monitor_periodic_f64_event(event, pin, periodic_f64_notifier, func);

        self.spawn(name, task, |events| events.add_periodic_f64_event(event))
    }

    /// Monitors a [`PeriodicEvent<f64>`] notifier without requiring a pin.
    ///
    /// Discard the event if it matches another.
    #[inline]
    #[must_use]
    pub fn periodic_f64_pinless<F, Fut>(
        self,
        name: &'static str,
        description: &'static str,
        interval: Duration,
        func: F,
    ) -> Self
    where
        F: Fn(PeriodicNotifier<f64>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let events_ref = self.events.periodic_f64_events_as_slice();
        let len = events_ref.len();

        for value in events_ref {
            if value.event.name == name {
                info!(
                    "The event `{}` is equal to `{}`, discard it.",
                    value.event.name, name
                );
                return self;
            }
        }

        let event = PeriodicEvent::f64(Event::f64(name).description(description), interval);
        let periodic_f64_notifier = PeriodicNotifier::f64(len, interval);
        // We need to do this because embassy tasks do not support generics.
        let func: PeriodicF64FnPinless = Box::new(move |notifier| Box::pin(func(notifier)));
        let task = monitor_periodic_f64_event_pinless(event, periodic_f64_notifier, func);

        self.spawn(name, task, |events| events.add_periodic_f64_event(event))
    }

    /// Runs the task that transmits events over the network.
    ///
    /// Returns a [`Device`] updated with [`EventsDescription`] data.
    ///
    /// # Errors
    ///
    /// It fails when:
    /// - The events manager is empty, meaning no events have been inserted.
    /// - The broker domain cannot be resolved through a `DNS` query.
    /// - The task responsible for network transmission cannot interact with its
    ///   scheduler or the network.
    pub async fn run_network_task(self) -> Result<Device<S>, Error> {
        if self.events.is_empty() {
            return Err(Error::new(
                ErrorKind::EmptyEventsManager,
                "No events in the event manager",
            ));
        }

        let remote_endpoint = match self.config.broker {
            BrokerData::Url(url, port) => {
                let address = self
                    .config
                    .stack
                    .dns_query(url, DnsQueryType::A)
                    .await
                    .map(|a| a[0])?;
                (address, port)
            }

            BrokerData::Ip(ip, port) => (ip, port),
        };

        self.config.spawner.spawn(write_on_network(
            self.config.stack,
            remote_endpoint,
            self.config.topic.clone(),
        ))?;

        Ok(self
            .config
            .device
            .events_description(EventsDescription::new(
                ToscaBrokerData::new(IpAddr::from(remote_endpoint.0), remote_endpoint.1),
                self.config.topic,
                self.events,
            )))
    }

    fn spawn<F, T>(mut self, name: &'static str, task: SpawnToken<T>, add_event: F) -> Self
    where
        F: FnOnce(&mut Events),
    {
        if let Err(e) = self.config.spawner.spawn(task) {
            error!("Impossible to spawn the event `{name}`: {e}");
            return self;
        }
        add_event(&mut self.events);
        info!("Spawned the task for event `{name}`");
        self
    }
}
