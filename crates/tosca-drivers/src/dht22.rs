//! # DHT22 Driver
//!
//! This module provides an architecture-agnostic driver for the `DHT22`
//! temperature and humidity sensor.
//! The driver is synchronous to meet the strict timing requirements of the
//! sensor's single-wire protocol.
//! The initial start signal uses a brief asynchronous wait to initiate
//! communication without blocking the executor, while all subsequent
//! timing-critical operations use precise blocking delays to ensure accurate
//! measurements.
//!
//! The `DHT22` sensor provides the following measurements:
//! - **Humidity**: Relative humidity as a percentage (% RH)
//! - **Temperature**: Temperature in degrees Celsius (°C)
//!
//! For detailed specifications, refer to the
//! [datasheet](https://www.alldatasheet.com/datasheet-pdf/pdf/1132459/ETC2/DHT22.html)
//! and the description of the proprietary
//! [communication protocol](https://www.ocfreaks.com/basics-interfacing-dht11-dht22-humidity-temperature-sensor-mcu/).

use core::result::Result::{self, Err, Ok};

use embedded_hal::delay::DelayNs as SyncDelay;
use embedded_hal::digital::{InputPin, OutputPin, PinState};

use embedded_hal_async::delay::DelayNs as AsyncDelay;

// Protocol-specific timing constants.
const START_SIGNAL_LOW_MS: u32 = 18; // MCU pulls line low for at least 18 ms to initiate communication.
const START_SIGNAL_HIGH_US: u32 = 40; // Then releases the line (high) for ~20–40 µs.
const BIT_SAMPLE_DELAY_US: u32 = 35; // Time after which to sample the data bit.
const POLL_DELAY_US: u32 = 1; // Delay between pin state polls when waiting for edges.
const MAX_ATTEMPTS: usize = 100; // Maximum polling iterations before timeout.

/// A single humidity and temperature measurement.
#[derive(Debug, Clone, Copy)]
pub struct Measurement {
    /// Relative humidity as a percentage (% RH).
    pub humidity: f32,
    /// Temperature in degrees Celsius (°C).
    pub temperature: f32,
}

/// Errors that may occur when interacting with the `DHT22` sensor.
#[derive(Debug)]
pub enum Dht22Error<E> {
    /// GPIO pin errors.
    Pin(E),
    /// Data checksum mismatch.
    ChecksumMismatch,
    /// Timeout waiting for sensor response.
    Timeout,
}

impl<E> From<E> for Dht22Error<E> {
    fn from(e: E) -> Self {
        Dht22Error::Pin(e)
    }
}

/// The `DHT22` driver.
pub struct Dht22<P, D>
where
    P: InputPin + OutputPin,
    D: SyncDelay + AsyncDelay,
{
    pin: P,
    delay: D,
}

// Raw sensor data: (humidity high, humidity low, temperature high, temperature low, checksum).
type RawData = (u8, u8, u8, u8, u8);

impl<P, D> Dht22<P, D>
where
    P: InputPin + OutputPin,
    D: SyncDelay + AsyncDelay,
{
    /// Creates a [`Dht22`] driver for the given pin and delay provider.
    #[must_use]
    pub fn new(pin: P, delay: D) -> Self {
        Self { pin, delay }
    }

    /// Reads a single humidity and temperature measurement.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Reading from the pin fails
    /// - The sensor does not respond within the expected timing window
    /// - The received data fails checksum validation
    pub fn read(&mut self) -> Result<Measurement, Dht22Error<P::Error>> {
        // Initiate communication by sending the start signal to the sensor.
        self.send_start_signal()?;

        // Wait for the sensor’s response (low → high handshake).
        self.wait_for_sensor_response()?;

        // Read 5 bytes: humidity high + low, temperature high + low, and checksum.
        let (hh, hl, th, tl, checksum) = self.read_raw_data()?;

        // Validate that the transmitted checksum matches the calculated one.
        Self::validate_checksum(hh, hl, th, tl, checksum)?;

        Ok(Measurement {
            humidity: Self::decode_humidity(hh, hl),
            temperature: Self::decode_temperature(th, tl),
        })
    }

    fn send_start_signal(&mut self) -> Result<(), Dht22Error<P::Error>> {
        // Pull the line low for at least 18 ms to signal the sensor.
        self.pin.set_low()?;
        SyncDelay::delay_ms(&mut self.delay, START_SIGNAL_LOW_MS);

        // Release the line high briefly before the sensor takes control of it.
        self.pin.set_high()?;
        SyncDelay::delay_us(&mut self.delay, START_SIGNAL_HIGH_US);

        Ok(())
    }

    fn wait_for_sensor_response(&mut self) -> Result<(), Dht22Error<P::Error>> {
        // The sensor pulls the line low and then high to acknowledge.
        self.wait_until_state(PinState::Low)?;
        self.wait_until_state(PinState::High)?;

        Ok(())
    }

    fn read_raw_data(&mut self) -> Result<RawData, Dht22Error<P::Error>> {
        // Sequentially read 5 bytes from the sensor.
        Ok((
            self.read_byte()?,
            self.read_byte()?,
            self.read_byte()?,
            self.read_byte()?,
            self.read_byte()?,
        ))
    }

    #[inline]
    fn validate_checksum(
        hh: u8,
        hl: u8,
        th: u8,
        tl: u8,
        checksum: u8,
    ) -> Result<(), Dht22Error<P::Error>> {
        // The checksum is the low 8 bits of the sum of the first four bytes.
        let sum = hh.wrapping_add(hl).wrapping_add(th).wrapping_add(tl);

        if sum == checksum {
            Ok(())
        } else {
            Err(Dht22Error::ChecksumMismatch)
        }
    }

    #[inline]
    fn decode_humidity(high: u8, low: u8) -> f32 {
        // Combine two bytes into a 16-bit integer and divide by 10 (sensor sends humidity * 10).
        f32::from((u16::from(high) << 8) | u16::from(low)) / 10.0
    }

    #[inline]
    fn decode_temperature(high: u8, low: u8) -> f32 {
        // The 16-bit temperature value has its sign bit at bit 15 (high byte’s MSB).
        let raw = (u16::from(high & 0x7F) << 8) | u16::from(low);
        let mut t = f32::from(raw) / 10.0;

        // If the sign bit is set, temperature is negative.
        if high & 0x80 != 0 {
            t = -t;
        }

        t
    }

    fn wait_until_state(&mut self, state: PinState) -> Result<(), Dht22Error<P::Error>> {
        // Poll the pin until it matches the desired state or timeout occurs.
        for _ in 0..MAX_ATTEMPTS {
            let reached = match state {
                PinState::High => self.pin.is_high()?,
                PinState::Low => self.pin.is_low()?,
            };
            if reached {
                return Ok(());
            }
            SyncDelay::delay_us(&mut self.delay, POLL_DELAY_US);
        }

        Err(Dht22Error::Timeout)
    }

    fn read_byte(&mut self) -> Result<u8, Dht22Error<P::Error>> {
        let mut byte = 0;

        // Each bit transmission consists of a low pulse followed by a high pulse.
        // The duration of the high pulse determines whether the bit is 0 or 1.
        for i in 0..8 {
            self.wait_until_state(PinState::Low)?; // Wait for the start of bit transmission.
            self.wait_until_state(PinState::High)?; // Wait for the high phase.

            // Sample after ~30 µs to determine bit value.
            SyncDelay::delay_us(&mut self.delay, BIT_SAMPLE_DELAY_US);

            // If the line is still high, the bit is 1; otherwise, it's 0.
            if self.pin.is_high()? {
                byte |= 1 << (7 - i); // Bits are transmitted MSB first.
            }
        }

        Ok(byte)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    extern crate std;
    use std::vec;

    use embedded_hal_mock::eh1::delay::NoopDelay;
    use embedded_hal_mock::eh1::digital::{Mock as PinMock, State, Transaction as PinTransaction};

    #[test]
    fn test_send_start_signal() {
        let expectations = [
            PinTransaction::set(State::Low),
            PinTransaction::set(State::High),
        ];

        let pin = PinMock::new(&expectations);
        let delay = NoopDelay::new();
        let mut dht22 = Dht22::new(pin, delay);

        dht22.send_start_signal().unwrap();

        dht22.pin.done();
    }

    #[test]
    fn test_wait_for_sensor_response_success() {
        let expectations = [
            PinTransaction::get(State::Low),
            PinTransaction::get(State::High),
        ];

        let pin = PinMock::new(&expectations);
        let delay = NoopDelay::new();
        let mut dht22 = Dht22::new(pin, delay);

        dht22.wait_for_sensor_response().unwrap();

        dht22.pin.done();
    }

    #[test]
    fn test_wait_until_state_timeout() {
        // Simulate all MAX_ATTEMPTS calls without ever reaching the desired state.
        let expectations = vec![PinTransaction::get(State::High); MAX_ATTEMPTS];

        let pin = PinMock::new(&expectations);
        let delay = NoopDelay::new();
        let mut dht22 = Dht22::new(pin, delay);

        let result = dht22.wait_until_state(PinState::Low);
        assert!(matches!(result, Err(Dht22Error::Timeout)));

        dht22.pin.done();
    }

    #[test]
    fn test_read_byte_all_zeros() {
        let mut expectations = vec![];

        // 8 bits: for each bit, wait for line low (start of bit), then high (bit signal), then sample line to determine 0.
        for _ in 0..8 {
            expectations.push(PinTransaction::get(State::Low)); // Falling edge.
            expectations.push(PinTransaction::get(State::High)); // Rising edge.
            expectations.push(PinTransaction::get(State::Low)); // Sampling: line low → bit 0.
        }

        let pin = PinMock::new(&expectations);
        let delay = NoopDelay::new();
        let mut dht22 = Dht22::new(pin, delay);

        let byte = dht22.read_byte().unwrap();
        assert_eq!(byte, 0x00);

        dht22.pin.done();
    }

    #[test]
    fn test_read_byte_all_ones() {
        let mut expectations = vec![];

        // 8 bits: for each bit, wait for line low (start of bit), then high (bit signal), then sample line to determine 1.
        for _ in 0..8 {
            expectations.push(PinTransaction::get(State::Low)); // Falling edge.
            expectations.push(PinTransaction::get(State::High)); // Rising edge.
            expectations.push(PinTransaction::get(State::High)); // Sampling: line high → bit 1.
        }

        let pin = PinMock::new(&expectations);
        let delay = NoopDelay::new();
        let mut dht22 = Dht22::new(pin, delay);

        let byte = dht22.read_byte().unwrap();
        assert_eq!(byte, 0xFF);

        dht22.pin.done();
    }

    #[test]
    fn test_decode_humidity_temperature() {
        let humidity = Dht22::<PinMock, NoopDelay>::decode_humidity(0x02, 0x58); // 600 → 60.0%.
        let temperature = Dht22::<PinMock, NoopDelay>::decode_temperature(0x00, 0xFA); // 250 → 25.0°C.
        let temperature_neg = Dht22::<PinMock, NoopDelay>::decode_temperature(0x80, 0xFA); // Negative: -25.0°C.

        assert!((humidity - 60.0).abs() < f32::EPSILON);
        assert!((temperature - 25.0).abs() < f32::EPSILON);
        assert!((temperature_neg + 25.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_validate_checksum() {
        let result_ok = Dht22::<PinMock, NoopDelay>::validate_checksum(1, 2, 3, 4, 10);
        assert!(result_ok.is_ok());

        let result_err = Dht22::<PinMock, NoopDelay>::validate_checksum(1, 2, 3, 4, 9);
        assert!(matches!(result_err, Err(Dht22Error::ChecksumMismatch)));
    }
}
