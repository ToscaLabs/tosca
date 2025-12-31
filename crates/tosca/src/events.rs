use alloc::string::String;
use alloc::vec::Vec;

use core::fmt;
use core::net::IpAddr;
use core::time::Duration;

use serde::Serialize;

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
    /// Creates a [`BrokerData`] .
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

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(not(feature = "deserialize"), derive(Copy))]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
/// An event of a specific type.
pub struct Event<T: Clone + Copy + private::TypeName> {
    /// Event name.
    #[cfg(not(feature = "deserialize"))]
    pub name: &'static str,
    /// Event name.
    #[cfg(feature = "deserialize")]
    pub name: alloc::borrow::Cow<'static, str>,

    /// Event description.
    #[cfg(not(feature = "deserialize"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'static str>,
    /// Event description.
    #[cfg(feature = "deserialize")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<alloc::borrow::Cow<'static, str>>,

    /// Event value.
    pub value: T,
}

impl<T: Clone + Copy + fmt::Display + private::TypeName> fmt::Display for Event<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        writeln!(f, "Name: \"{}\"", self.name)?;
        if let Some(description) = &self.description {
            writeln!(f, "Description: \"{description}\"")?;
        }
        writeln!(f, "Type: {}", T::TYPE)?;
        writeln!(f, "Value: {}", self.value)
    }
}

impl Event<bool> {
    /// Creates an [`Event<bool>`].
    #[must_use]
    pub const fn bool(name: &'static str) -> Self {
        Self {
            #[cfg(not(feature = "deserialize"))]
            name,
            #[cfg(feature = "deserialize")]
            name: alloc::borrow::Cow::Borrowed(name),
            description: None,
            value: false,
        }
    }
}

impl private::TypeName for bool {
    const TYPE: &'static str = "bool";
}

impl Event<u8> {
    /// Creates an [`Event<u8>`].
    #[must_use]
    pub const fn u8(name: &'static str) -> Self {
        Self {
            #[cfg(not(feature = "deserialize"))]
            name,
            #[cfg(feature = "deserialize")]
            name: alloc::borrow::Cow::Borrowed(name),
            description: None,
            value: 0,
        }
    }
}

impl private::TypeName for u8 {
    const TYPE: &'static str = "u8";
}

impl Event<i32> {
    /// Creates an [`Event<i32>`].
    #[must_use]
    pub const fn i32(name: &'static str) -> Self {
        Self {
            #[cfg(not(feature = "deserialize"))]
            name,
            #[cfg(feature = "deserialize")]
            name: alloc::borrow::Cow::Borrowed(name),
            description: None,
            value: 0,
        }
    }
}

impl private::TypeName for i32 {
    const TYPE: &'static str = "i32";
}

impl Event<f32> {
    /// Creates an [`Event<f32>`].
    #[must_use]
    pub const fn f32(name: &'static str) -> Self {
        Self {
            #[cfg(not(feature = "deserialize"))]
            name,
            #[cfg(feature = "deserialize")]
            name: alloc::borrow::Cow::Borrowed(name),
            description: None,
            value: 0.,
        }
    }
}

impl private::TypeName for f32 {
    const TYPE: &'static str = "f32";
}

impl Event<f64> {
    /// Creates an [`Event<f64>`].
    #[must_use]
    pub const fn f64(name: &'static str) -> Self {
        Self {
            #[cfg(not(feature = "deserialize"))]
            name,
            #[cfg(feature = "deserialize")]
            name: alloc::borrow::Cow::Borrowed(name),
            description: None,
            value: 0.,
        }
    }
}

impl private::TypeName for f64 {
    const TYPE: &'static str = "f64";
}

impl<T: Clone + Copy + private::TypeName> Event<T> {
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

    /// Removes the event description.
    ///
    /// This method might be useful to reduce the payload sent over the network.
    #[cfg(not(feature = "deserialize"))]
    pub const fn remove_description(&mut self) {
        self.description = None;
    }

    /// Removes the event description.
    ///
    /// This method can help reduce the amount of data transmitted over
    /// the network.
    #[cfg(feature = "deserialize")]
    #[inline]
    pub fn remove_description(&mut self) {
        self.description = None;
    }

    // Updates the event value.
    pub(crate) const fn update_value(&mut self, value: T) {
        self.value = value;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(not(feature = "deserialize"), derive(Copy))]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
/// A periodic [`Event`].
///
/// An event is considered periodic when it is triggered or checked at regular,
/// fixed intervals of time.
pub struct PeriodicEvent<T: Clone + Copy + private::TypeName> {
    /// The [`Event`].
    pub event: Event<T>,
    /// Time interval for checking if the event has occurred.
    pub interval: Duration,
}

impl<T: Clone + Copy + fmt::Display + private::TypeName> fmt::Display for PeriodicEvent<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        writeln!(
            f,
            "Interval: {}s {}ms",
            self.interval.as_secs(),
            self.interval.subsec_millis()
        )?;
        self.event.fmt(f)
    }
}

impl PeriodicEvent<bool> {
    /// Creates a [`PeriodicEvent<bool>`].
    #[must_use]
    pub const fn bool(event: Event<bool>, interval: Duration) -> Self {
        Self { event, interval }
    }
}

impl PeriodicEvent<u8> {
    /// Creates a [`PeriodicEvent<u8>`].
    #[must_use]
    pub const fn u8(event: Event<u8>, interval: Duration) -> Self {
        Self { event, interval }
    }
}

impl PeriodicEvent<i32> {
    /// Creates a [`PeriodicEvent<i32>`].
    #[must_use]
    pub const fn i32(event: Event<i32>, interval: Duration) -> Self {
        Self { event, interval }
    }
}

impl PeriodicEvent<f32> {
    /// Creates a [`PeriodicEvent<f32>`].
    #[must_use]
    pub const fn f32(event: Event<f32>, interval: Duration) -> Self {
        Self { event, interval }
    }
}

impl PeriodicEvent<f64> {
    /// Creates a [`PeriodicEvent<f64>`].
    #[must_use]
    pub const fn f64(event: Event<f64>, interval: Duration) -> Self {
        Self { event, interval }
    }
}

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

#[derive(Debug, Clone, PartialEq, Serialize)]
#[allow(clippy::struct_field_names)]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
/// All events types that can be generated by a device.
///
/// Events of the same type are stored and displayed sequentially.
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
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    periodic_bool_events: Vec<PeriodicEvent<bool>>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    periodic_u8_events: Vec<PeriodicEvent<u8>>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    periodic_i32_events: Vec<PeriodicEvent<i32>>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    periodic_f32_events: Vec<PeriodicEvent<f32>>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    periodic_f64_events: Vec<PeriodicEvent<f64>>,
}

impl fmt::Display for Events {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        if !self.bool_events.is_empty() {
            for bool_event in &self.bool_events {
                bool_event.fmt(f)?;
            }
        }

        if !self.u8_events.is_empty() {
            for u8_event in &self.u8_events {
                u8_event.fmt(f)?;
            }
        }

        if !self.i32_events.is_empty() {
            for i32_event in &self.i32_events {
                i32_event.fmt(f)?;
            }
        }

        if !self.f32_events.is_empty() {
            for f32_event in &self.f32_events {
                f32_event.fmt(f)?;
            }
        }

        if !self.f64_events.is_empty() {
            for f64_event in &self.f64_events {
                f64_event.fmt(f)?;
            }
        }

        if !self.periodic_bool_events.is_empty() {
            for periodic_bool_event in &self.periodic_bool_events {
                periodic_bool_event.fmt(f)?;
            }
        }

        if !self.periodic_u8_events.is_empty() {
            for periodic_u8_event in &self.periodic_u8_events {
                periodic_u8_event.fmt(f)?;
            }
        }

        if !self.periodic_i32_events.is_empty() {
            for periodic_i32_event in &self.periodic_i32_events {
                periodic_i32_event.fmt(f)?;
            }
        }

        if !self.periodic_f32_events.is_empty() {
            for periodic_f32_event in &self.periodic_f32_events {
                periodic_f32_event.fmt(f)?;
            }
        }

        if !self.periodic_f64_events.is_empty() {
            for periodic_f64_event in &self.periodic_f64_events {
                periodic_f64_event.fmt(f)?;
            }
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
            periodic_bool_events: Vec::new(),
            periodic_u8_events: Vec::new(),
            periodic_i32_events: Vec::new(),
            periodic_f32_events: Vec::new(),
            periodic_f64_events: Vec::new(),
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
            periodic_bool_events: Vec::with_capacity(size),
            periodic_u8_events: Vec::with_capacity(size),
            periodic_i32_events: Vec::with_capacity(size),
            periodic_f32_events: Vec::with_capacity(size),
            periodic_f64_events: Vec::with_capacity(size),
        }
    }

    /// Adds a sequence of [`Event<bool>`].
    #[inline]
    #[must_use]
    pub fn bool_events(mut self, bool_events: Vec<Event<bool>>) -> Self {
        self.bool_events = bool_events;
        self
    }

    /// Adds a sequence of [`Event<u8>`].
    #[inline]
    #[must_use]
    pub fn u8_events(mut self, u8_events: Vec<Event<u8>>) -> Self {
        self.u8_events = u8_events;
        self
    }

    /// Adds a sequence of [`Event<i32>`].
    #[inline]
    #[must_use]
    pub fn i32_events(mut self, i32_events: Vec<Event<i32>>) -> Self {
        self.i32_events = i32_events;
        self
    }

    /// Adds a sequence of [`Event<f32>`].
    #[inline]
    #[must_use]
    pub fn f32_events(mut self, f32_events: Vec<Event<f32>>) -> Self {
        self.f32_events = f32_events;
        self
    }

    /// Adds a sequence of [`Event<f64>`].
    #[inline]
    #[must_use]
    pub fn f64_events(mut self, f64_events: Vec<Event<f64>>) -> Self {
        self.f64_events = f64_events;
        self
    }

    /// Adds a sequence of [`PeriodicEvent<bool>`].
    #[inline]
    #[must_use]
    pub fn periodic_bool_events(mut self, periodic_bool_events: Vec<PeriodicEvent<bool>>) -> Self {
        self.periodic_bool_events = periodic_bool_events;
        self
    }

    /// Adds a sequence of [`PeriodicEvent<u8>`].
    #[inline]
    #[must_use]
    pub fn periodic_u8_events(mut self, periodic_u8_events: Vec<PeriodicEvent<u8>>) -> Self {
        self.periodic_u8_events = periodic_u8_events;
        self
    }

    /// Adds a sequence of [`PeriodicEvent<i32>`].
    #[inline]
    #[must_use]
    pub fn periodic_i32_events(mut self, periodic_i32_events: Vec<PeriodicEvent<i32>>) -> Self {
        self.periodic_i32_events = periodic_i32_events;
        self
    }

    /// Adds a sequence of [`PeriodicEvent<f32>`].
    #[inline]
    #[must_use]
    pub fn periodic_f32_events(mut self, periodic_f32_events: Vec<PeriodicEvent<f32>>) -> Self {
        self.periodic_f32_events = periodic_f32_events;
        self
    }

    /// Adds a sequence of [`PeriodicEvent<f64>`].
    #[inline]
    #[must_use]
    pub fn periodic_f64_events(mut self, periodic_f64_events: Vec<PeriodicEvent<f64>>) -> Self {
        self.periodic_f64_events = periodic_f64_events;
        self
    }

    /// Adds a single [`Event<bool>`].
    #[inline]
    pub fn add_bool_event(&mut self, bool_event: Event<bool>) {
        self.bool_events.push(bool_event);
    }

    /// Adds a single [`Event<u8>`].
    #[inline]
    pub fn add_u8_event(&mut self, u8_event: Event<u8>) {
        self.u8_events.push(u8_event);
    }

    /// Adds a single [`Event<i32>`].
    #[inline]
    pub fn add_i32_event(&mut self, i32_event: Event<i32>) {
        self.i32_events.push(i32_event);
    }

    /// Adds a single [`Event<f32>`].
    #[inline]
    pub fn add_f32_event(&mut self, f32_event: Event<f32>) {
        self.f32_events.push(f32_event);
    }

    /// Adds a single [`Event<f64>`].
    #[inline]
    pub fn add_f64_event(&mut self, f64_event: Event<f64>) {
        self.f64_events.push(f64_event);
    }

    /// Adds a single [`PeriodicEvent<bool>`].
    #[inline]
    pub fn add_periodic_bool_event(&mut self, periodic_bool_event: PeriodicEvent<bool>) {
        self.periodic_bool_events.push(periodic_bool_event);
    }

    /// Adds a single [`PeriodicEvent<u8>`].
    #[inline]
    pub fn add_periodic_u8_event(&mut self, periodic_u8_event: PeriodicEvent<u8>) {
        self.periodic_u8_events.push(periodic_u8_event);
    }

    /// Adds a single [`PeriodicEvent<i32>`].
    #[inline]
    pub fn add_periodic_i32_event(&mut self, periodic_i32_event: PeriodicEvent<i32>) {
        self.periodic_i32_events.push(periodic_i32_event);
    }

    /// Adds a single [`PeriodicEvent<f32>`].
    #[inline]
    pub fn add_periodic_f32_event(&mut self, periodic_f32_event: PeriodicEvent<f32>) {
        self.periodic_f32_events.push(periodic_f32_event);
    }

    /// Adds a single [`PeriodicEvent<f64>`].
    #[inline]
    pub fn add_periodic_f64_event(&mut self, periodic_f64_event: PeriodicEvent<f64>) {
        self.periodic_f64_events.push(periodic_f64_event);
    }

    /// Updates the [`Event<bool>`] value located at the given index.
    #[inline]
    pub fn update_bool_value(&mut self, index: usize, value: bool) {
        self.bool_events[index].update_value(value);
    }

    /// Updates the [`Event<u8>`] value located at the given index.
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
    }

    /// Updates the [`PeriodicEvent<bool>`] value located at the given index.
    #[inline]
    pub fn update_periodic_bool_value(&mut self, index: usize, value: bool) {
        self.periodic_bool_events[index].event.update_value(value);
    }

    /// Updates the [`PeriodicEvent<u8>`] value located at the given index.
    #[inline]
    pub fn update_periodic_u8_value(&mut self, index: usize, value: u8) {
        self.periodic_u8_events[index].event.update_value(value);
    }

    /// Updates the [`PeriodicEvent<i32>`] value located at the given index.
    #[inline]
    pub fn update_periodic_i32_value(&mut self, index: usize, value: i32) {
        self.periodic_i32_events[index].event.update_value(value);
    }

    /// Updates the [`PeriodicEvent<f32>`] value located at the given index.
    #[inline]
    pub fn update_periodic_f32_value(&mut self, index: usize, value: f32) {
        self.periodic_f32_events[index].event.update_value(value);
    }

    /// Updates the [`PeriodicEvent<f64>`] value located at the given index.
    #[inline]
    pub fn update_periodic_f64_value(&mut self, index: usize, value: f64) {
        self.periodic_f64_events[index].event.update_value(value);
    }

    /// Returns an immutable slice of the [`Event<bool>`] sequence.
    #[inline]
    #[must_use]
    pub fn bool_events_as_slice(&self) -> &[Event<bool>] {
        self.bool_events.as_slice()
    }

    /// Returns an immutable slice of the [`Event<u8>`] sequence.
    #[inline]
    #[must_use]
    pub fn u8_events_as_slice(&self) -> &[Event<u8>] {
        self.u8_events.as_slice()
    }

    /// Returns an immutable slice of the [`Event<i32>`] sequence.
    #[inline]
    #[must_use]
    pub fn i32_events_as_slice(&self) -> &[Event<i32>] {
        self.i32_events.as_slice()
    }

    /// Returns an immutable slice of the [`Event<f32>`] sequence.
    #[inline]
    #[must_use]
    pub fn f32_events_as_slice(&self) -> &[Event<f32>] {
        self.f32_events.as_slice()
    }

    /// Returns an immutable slice of the [`Event<f64>`] sequence.
    #[inline]
    #[must_use]
    pub fn f64_events_as_slice(&self) -> &[Event<f64>] {
        self.f64_events.as_slice()
    }

    /// Returns an immutable slice of the [`PeriodicEvent<bool>`] sequence.
    #[inline]
    #[must_use]
    pub fn periodic_bool_events_as_slice(&self) -> &[PeriodicEvent<bool>] {
        self.periodic_bool_events.as_slice()
    }

    /// Returns an immutable slice of the [`PeriodicEvent<u8>`] sequence.
    #[inline]
    #[must_use]
    pub fn periodic_u8_events_as_slice(&self) -> &[PeriodicEvent<u8>] {
        self.periodic_u8_events.as_slice()
    }

    /// Returns an immutable slice of the [`PeriodicEvent<i32>`] sequence.
    #[inline]
    #[must_use]
    pub fn periodic_i32_events_as_slice(&self) -> &[PeriodicEvent<i32>] {
        self.periodic_i32_events.as_slice()
    }

    /// Returns an immutable slice of the [`PeriodicEvent<f32>`] sequence.
    #[inline]
    #[must_use]
    pub fn periodic_f32_events_as_slice(&self) -> &[PeriodicEvent<f32>] {
        self.periodic_f32_events.as_slice()
    }

    /// Returns an immutable slice of the [`PeriodicEvent<f64>`] sequence.
    #[inline]
    #[must_use]
    pub fn periodic_f64_events_as_slice(&self) -> &[PeriodicEvent<f64>] {
        self.periodic_f64_events.as_slice()
    }

    /// Checks if [`Events`] is **entirely** empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.bool_events.is_empty()
            && self.u8_events.is_empty()
            && self.i32_events.is_empty()
            && self.f32_events.is_empty()
            && self.f64_events.is_empty()
            && self.periodic_bool_events.is_empty()
            && self.periodic_u8_events.is_empty()
            && self.periodic_i32_events.is_empty()
            && self.periodic_f32_events.is_empty()
            && self.periodic_f64_events.is_empty()
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
