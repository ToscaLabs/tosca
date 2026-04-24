use alloc::string::String;
use alloc::vec::Vec;

use core::fmt;
use core::net::IpAddr;
use core::time::Duration;

use serde::Serialize;

macro_rules! define_event_type {
    ($name:ident, $type:ty, $default:expr) => {
        impl private::TypeName for $type {
            const TYPE: &'static str = stringify!($type);
        }

        impl Event<$type> {
            #[doc = concat!("Creates and  `Event<", stringify!($type), ">`.")]
            pub(crate) const fn $name(name: &'static str) -> Self {
                Self {
                    #[cfg(not(feature = "deserialize"))]
                    name,
                    #[cfg(feature = "deserialize")]
                    name: alloc::borrow::Cow::Borrowed(name),
                    value: $default,
                }
            }
        }

        impl EventMetadata<$type> {
            #[doc = concat!("Creates and  `EventMetadata<", stringify!($type), ">`.")]
            #[must_use]
            pub const fn $name(name: &'static str) -> Self {
                Self {
                    event: Event::$name(name),
                    description: None,
                    interval: None,
                }
            }
        }
    };
}

/// Event broker data.
#[derive(Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub struct BrokerData {
    /// Broker address.
    pub address: IpAddr,
    /// Broker port number.
    pub port: u16,
}

impl BrokerData {
    /// Creates a [`BrokerData`].
    #[must_use]
    pub const fn new(address: IpAddr, port: u16) -> Self {
        Self { address, port }
    }
}

// A fake trait to print the type of an event.
mod private {
    #[doc(hidden)]
    pub trait TypeName {
        const TYPE: &'static str;
    }
}

/// A generic event.
///
/// An event is identified by a name and an associated value.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(not(feature = "deserialize"), derive(Copy))]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub struct Event<T: Clone + Copy + private::TypeName> {
    /// Event name.
    #[cfg(not(feature = "deserialize"))]
    pub name: &'static str,
    /// Event name.
    #[cfg(feature = "deserialize")]
    pub name: alloc::borrow::Cow<'static, str>,

    /// Event value.
    pub value: T,
}

impl<T: Clone + Copy + fmt::Display + private::TypeName> fmt::Display for Event<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        writeln!(f, "Name: \"{}\"", self.name)?;
        writeln!(f, "Type: {}", T::TYPE)?;
        writeln!(f, "Value: {}", self.value)
    }
}

impl<T: Clone + Copy + private::TypeName> Event<T> {
    // Updates the event value.
    pub(crate) const fn update_value(&mut self, value: T) {
        self.value = value;
    }
}

/// An event and its associated metadata.
///
/// This structure is used exclusively to contain all event information that
/// will be included in the device description.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(not(feature = "deserialize"), derive(Copy))]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub struct EventMetadata<T: Clone + Copy + private::TypeName> {
    /// The effective event.
    pub event: Event<T>,

    /// Event description.
    #[cfg(not(feature = "deserialize"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'static str>,
    /// Event description.
    #[cfg(feature = "deserialize")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<alloc::borrow::Cow<'static, str>>,

    /// Time interval used to make an event periodic.
    ///
    /// An event is considered periodic if it is triggered or checked at
    /// regular, fixed time intervals.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<Duration>,
}

impl<T: Clone + Copy + fmt::Display + private::TypeName> fmt::Display for EventMetadata<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        writeln!(f, "Name: \"{}\"", self.event.name)?;
        if let Some(description) = &self.description {
            writeln!(f, "Description: \"{description}\"")?;
        }
        if let Some(interval) = &self.interval {
            writeln!(
                f,
                "Interval: {}s {}ms",
                interval.as_secs(),
                interval.subsec_millis()
            )?;
        }
        writeln!(f, "Type: {}", T::TYPE)?;
        writeln!(f, "Value: {}", self.event.value)
    }
}

impl<T: Clone + Copy + private::TypeName> EventMetadata<T> {
    /// Sets the event description.
    #[must_use]
    #[cfg(not(feature = "deserialize"))]
    pub const fn description(mut self, description: &'static str) -> Self {
        self.description = Some(description);
        self
    }

    /// Sets the event description.
    #[must_use]
    #[inline]
    #[cfg(feature = "deserialize")]
    pub fn description(mut self, description: &'static str) -> Self {
        self.description = Some(alloc::borrow::Cow::Borrowed(description));
        self
    }

    /// Sets the time interval.
    ///
    /// This method makes the event periodic.
    #[must_use]
    #[inline]
    pub fn interval(mut self, interval: Duration) -> Self {
        self.interval = Some(interval);
        self
    }
}

define_event_type!(bool, bool, false);
define_event_type!(u8, u8, 0);
define_event_type!(i32, i32, 0);
define_event_type!(f32, f32, 0.);
define_event_type!(f64, f64, 0.);

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
/// The topic for event publication over the network.
///
/// This topic uniquely identifies all events coming from a device, allowing
/// controllers to retrieve all related event data using it as a reference.
pub struct Topic(String);

impl Topic {
    /// Creates an empty [`Topic`].
    #[must_use]
    pub const fn empty() -> Self {
        Self(String::new())
    }

    /// Creates a [`Topic`].
    #[must_use]
    pub const fn new(value: String) -> Self {
        Self(value)
    }

    /// Returns the [`Topic`] as a [`&str`].
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// All events types that can be generated by a device.
///
/// Events of the same type are stored and displayed sequentially.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub struct Events {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    bool_events: Vec<Event<bool>>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    u8_events: Vec<Event<u8>>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    i32_events: Vec<Event<i32>>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    f32_events: Vec<Event<f32>>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    f64_events: Vec<Event<f64>>,
}

impl fmt::Display for Events {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        if self.is_empty() {
            return writeln!(f, "No events available.");
        }

        for bool_event in &self.bool_events {
            bool_event.fmt(f)?;
        }

        for u8_event in &self.u8_events {
            u8_event.fmt(f)?;
        }

        for i32_event in &self.i32_events {
            i32_event.fmt(f)?;
        }

        for f32_event in &self.f32_events {
            f32_event.fmt(f)?;
        }

        for f64_event in &self.f64_events {
            f64_event.fmt(f)?;
        }

        Ok(())
    }
}

impl Events {
    /// Creates an empty [`Events`].
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            bool_events: Vec::new(),
            u8_events: Vec::new(),
            i32_events: Vec::new(),
            f32_events: Vec::new(),
            f64_events: Vec::new(),
        }
    }

    /// Creates an [`Events`] with equal memory capacity for each of its
    /// internal event sequences.
    #[inline]
    #[must_use]
    pub fn with_capacity(size: usize) -> Self {
        Self {
            bool_events: Vec::with_capacity(size),
            u8_events: Vec::with_capacity(size),
            i32_events: Vec::with_capacity(size),
            f32_events: Vec::with_capacity(size),
            f64_events: Vec::with_capacity(size),
        }
    }

    /// Adds [`Event<bool>`].
    #[inline]
    pub fn add_bool(&mut self, bool_event: Event<bool>) {
        self.bool_events.push(bool_event);
    }

    /// Adds [`Event<u8>`].
    #[inline]
    pub fn add_u8(&mut self, u8_event: Event<u8>) {
        self.u8_events.push(u8_event);
    }

    /// Adds [`Event<i32>`].
    #[inline]
    pub fn add_i32(&mut self, i32_event: Event<i32>) {
        self.i32_events.push(i32_event);
    }

    /// Adds [`Event<f32>`].
    #[inline]
    pub fn add_f32(&mut self, f32_event: Event<f32>) {
        self.f32_events.push(f32_event);
    }

    /// Adds [`Event<f64>`].
    #[inline]
    pub fn add_f64(&mut self, f64_event: Event<f64>) {
        self.f64_events.push(f64_event);
    }

    /// Updates the [`Event<bool>`] value with the given name.
    #[inline]
    pub fn update_bool_value(&mut self, name: &str, value: bool) {
        //self.bool_events[index].update_value(value);
    }

    /*/// Updates the [`Event<u8>`] value located at the given index.
    #[inline]
    pub fn update_u8_value(&mut self, index: usize, value: u8) {
        self.u8_events[index].update_value(value);
    }

    /// Updates the [`Event<i32>`] value located at the given index.
    #[inline]
    pub fn update_i32_value(&mut self, index: usize, value: i32) {
        self.i32_events[index].update_value(value);
    }

    /// Updates the [`Event<f32>`] value located at the given index.
    #[inline]
    pub fn update_f32_value(&mut self, index: usize, value: f32) {
        self.f32_events[index].update_value(value);
    }

    /// Updates the [`Event<f64>`] value located at the given index.
    #[inline]
    pub fn update_f64_value(&mut self, index: usize, value: f64) {
        self.f64_events[index].update_value(value);
    }*/

    /// Returns an immutable slice of the [`Event<bool>`] sequence.
    #[inline]
    #[must_use]
    pub fn bool_events_as_slice(&self) -> &[Event<bool>] {
        &self.bool_events
    }

    /// Returns an immutable slice of the [`Event<u8>`] sequence.
    #[inline]
    #[must_use]
    pub fn u8_events_as_slice(&self) -> &[Event<u8>] {
        &self.u8_events
    }

    /// Returns an immutable slice of the [`Event<i32>`] sequence.
    #[inline]
    #[must_use]
    pub fn i32_events_as_slice(&self) -> &[Event<i32>] {
        &self.i32_events
    }

    /// Returns an immutable slice of the [`Event<f32>`] sequence.
    #[inline]
    #[must_use]
    pub fn f32_events_as_slice(&self) -> &[Event<f32>] {
        &self.f32_events
    }

    /// Returns an immutable slice of the [`Event<f64>`] sequence.
    #[inline]
    #[must_use]
    pub fn f64_events_as_slice(&self) -> &[Event<f64>] {
        self.f64_events.as_slice()
    }

    /// Checks if [`Events`] is **entirely** empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.bool_events.is_empty()
            && self.u8_events.is_empty()
            && self.i32_events.is_empty()
            && self.f32_events.is_empty()
            && self.f64_events.is_empty()
    }
}

#[derive(Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
/// All events to be published over the network, including their associated
/// topic and broker data.
pub struct EventsDescription {
    /// Broker data.
    pub broker_data: BrokerData,
    /// Topic information.
    pub topic: Topic,
    /// All device events.
    pub events: Events,
}

impl EventsDescription {
    /// Creates an [`EventsDescription`].
    #[must_use]
    pub const fn new(broker_data: BrokerData, topic: Topic, events: Events) -> Self {
        Self {
            broker_data,
            topic,
            events,
        }
    }
}

#[cfg(test)]
#[cfg(feature = "deserialize")]
mod tests {
    use core::net::Ipv4Addr;
    use core::time::Duration;

    use crate::{deserialize, serialize};

    use super::{BrokerData, Event, Events, EventsDescription, PeriodicEvent, Topic};

    const DEFAULT_DURATION: Duration = Duration::from_secs(1);

    #[test]
    #[allow(clippy::similar_names)]
    fn test_all_event_kinds() {
        let bool_event = Event::bool("bool_event").description("A bool event");
        assert_eq!(
            deserialize::<Event<bool>>(serialize(&bool_event)),
            bool_event
        );

        let periodic_bool_event = PeriodicEvent::bool(bool_event, DEFAULT_DURATION);
        assert_eq!(
            deserialize::<PeriodicEvent<bool>>(serialize(&periodic_bool_event)),
            periodic_bool_event
        );

        let u8_event = Event::u8("u8_event").description("An u8 event");
        assert_eq!(deserialize::<Event<u8>>(serialize(&u8_event)), u8_event);

        let periodic_u8_event = PeriodicEvent::u8(u8_event, DEFAULT_DURATION);
        assert_eq!(
            deserialize::<PeriodicEvent<u8>>(serialize(&periodic_u8_event)),
            periodic_u8_event
        );

        let i32_event = Event::i32("i32_event").description("An i32 event");
        assert_eq!(deserialize::<Event<i32>>(serialize(&i32_event)), i32_event);

        let periodic_i32_event = PeriodicEvent::i32(i32_event, DEFAULT_DURATION);
        assert_eq!(
            deserialize::<PeriodicEvent<i32>>(serialize(&periodic_i32_event)),
            periodic_i32_event
        );

        let f32_event = Event::f32("f32_event").description("An f32 event");
        assert_eq!(deserialize::<Event<f32>>(serialize(&f32_event)), f32_event);

        let periodic_f32_event = PeriodicEvent::f32(f32_event, DEFAULT_DURATION);
        assert_eq!(
            deserialize::<PeriodicEvent<f32>>(serialize(&periodic_f32_event)),
            periodic_f32_event
        );

        let f64_event = Event::f64("f64_event").description("An f64 event");
        assert_eq!(deserialize::<Event<f64>>(serialize(&f64_event)), f64_event);

        let periodic_f64_event = PeriodicEvent::f64(f64_event, DEFAULT_DURATION);
        assert_eq!(
            deserialize::<PeriodicEvent<f64>>(serialize(&periodic_f64_event)),
            periodic_f64_event
        );
    }

    #[test]
    fn test_events_with_single_event_kind() {
        let bool_event = Event::bool("bool_event").description("A bool event");

        let mut events = Events::empty();
        events.add_bool_event(bool_event);

        assert_eq!(deserialize::<Events>(serialize(&events)), events);
    }

    #[test]
    #[allow(clippy::similar_names)]
    fn test_events_with_all_event_kinds() {
        let bool_event = Event::bool("bool_event").description("A bool event");
        let periodic_bool_event = PeriodicEvent::bool(bool_event.clone(), DEFAULT_DURATION);
        let u8_event = Event::u8("u8_event").description("An u8 event");
        let periodic_u8_event = PeriodicEvent::u8(u8_event.clone(), DEFAULT_DURATION);
        let i32_event = Event::i32("i32_event").description("An i32 event");
        let periodic_i32_event = PeriodicEvent::i32(i32_event.clone(), DEFAULT_DURATION);
        let f32_event = Event::f32("f32_event").description("An f32 event");
        let periodic_f32_event = PeriodicEvent::f32(f32_event.clone(), DEFAULT_DURATION);
        let f64_event = Event::f64("f64_event").description("An f64 event");
        let periodic_f64_event = PeriodicEvent::f64(f64_event.clone(), DEFAULT_DURATION);

        let mut events = Events::empty();
        events.add_bool_event(bool_event);
        events.add_periodic_bool_event(periodic_bool_event);
        events.add_u8_event(u8_event);
        events.add_periodic_u8_event(periodic_u8_event);
        events.add_i32_event(i32_event);
        events.add_periodic_i32_event(periodic_i32_event);
        events.add_f32_event(f32_event);
        events.add_periodic_f32_event(periodic_f32_event);
        events.add_f64_event(f64_event);
        events.add_periodic_f64_event(periodic_f64_event);

        assert_eq!(deserialize::<Events>(serialize(&events)), events);
    }

    #[test]
    fn test_events_description() {
        let broker_data = BrokerData::new(Ipv4Addr::LOCALHOST.into(), 80);
        assert_eq!(
            deserialize::<BrokerData>(serialize(&broker_data)),
            broker_data
        );

        let topic = Topic::new("test".into());
        assert_eq!(deserialize::<Topic>(serialize(&topic)), topic);

        let bool_event = Event::bool("bool_event").description("A bool event");
        let mut events = Events::empty();
        events.add_bool_event(bool_event);

        let events_description = EventsDescription::new(broker_data, topic, events);
        assert_eq!(
            deserialize::<EventsDescription>(serialize(&events_description)),
            events_description
        );
    }
}
