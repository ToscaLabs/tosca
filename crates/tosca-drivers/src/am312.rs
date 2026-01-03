//! # AM312 Driver
//!
//! This module provides an asynchronous, architecture-agnostic driver for the
//! `AM312 PIR` motion sensor, which detects motion via a digital pin:
//!
//! - **High** when movement is detected
//! - **Low** when no movement is detected
//!
//! After power-on, the `AM312` requires a calibration period of 10 to 60
//! seconds before motion readings become reliable.
//! Therefore, ensure this waiting period has passed before invoking any motion
//! detection methods.
//!
//! For detailed specifications, refer to the
//! [datasheet](https://www.alldatasheet.com/datasheet-pdf/pdf/1179499/ETC2/AM312.html).

use core::result::Result;

use embedded_hal::digital::InputPin;

use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::digital::Wait;

const DEBOUNCE_MS: u32 = 50;

/// The `AM312` driver.
pub struct Am312<P, D>
where
    P: InputPin + Wait,
    D: DelayNs,
{
    pin: P,
    delay: D,
}

impl<P, D> Am312<P, D>
where
    P: InputPin + Wait,
    D: DelayNs,
{
    /// Creates an [`Am312`] driver with the given input pin.
    #[must_use]
    #[inline]
    pub fn new(pin: P, delay: D) -> Self {
        Self { pin, delay }
    }

    /// Waits until motion is detected.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying pin fails while waiting for
    /// or reading the input state.
    pub async fn wait_for_motion_start(&mut self) -> Result<(), P::Error> {
        loop {
            self.pin.wait_for_rising_edge().await?;

            // Debounce.
            self.delay.delay_ms(DEBOUNCE_MS).await;

            if self.pin.is_high()? {
                return Ok(());
            }
        }
    }

    /// Waits until motion ends.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying pin fails while waiting for
    /// or reading the input state.
    pub async fn wait_for_motion_end(&mut self) -> Result<(), P::Error> {
        loop {
            self.pin.wait_for_falling_edge().await?;

            // Debounce.
            self.delay.delay_ms(DEBOUNCE_MS).await;

            if self.pin.is_low()? {
                return Ok(());
            }
        }
    }

    /// Returns `true` if motion is currently detected.
    ///
    /// # Errors
    ///
    /// Returns an error if reading the pin state fails.
    #[inline]
    pub fn is_motion_detected(&mut self) -> Result<bool, P::Error> {
        self.pin.is_high()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use embedded_hal_mock::eh1::delay::NoopDelay;
    use embedded_hal_mock::eh1::digital::{
        Edge, Mock as PinMock, State, Transaction as PinTransaction,
    };

    #[tokio::test]
    async fn test_wait_for_motion_start() {
        let expectations = [
            PinTransaction::wait_for_edge(Edge::Rising),
            PinTransaction::get(State::High),
        ];

        let pin = PinMock::new(&expectations);
        let delay = NoopDelay::new();
        let mut am312 = Am312::new(pin, delay);

        let res = am312.wait_for_motion_start().await;
        assert!(res.is_ok());

        am312.pin.done();
    }

    #[tokio::test]
    async fn test_wait_for_motion_end() {
        let expectations = [
            PinTransaction::wait_for_edge(Edge::Falling),
            PinTransaction::get(State::Low),
        ];

        let pin = PinMock::new(&expectations);
        let delay = NoopDelay::new();
        let mut am312 = Am312::new(pin, delay);

        let res = am312.wait_for_motion_end().await;
        assert!(res.is_ok());

        am312.pin.done();
    }

    #[test]
    fn test_is_motion_detected_true() {
        let expectations = [PinTransaction::get(State::High)];

        let pin = PinMock::new(&expectations);
        let delay = NoopDelay::new();
        let mut am312 = Am312::new(pin, delay);

        let res = am312.is_motion_detected().unwrap();
        assert!(res);

        am312.pin.done();
    }

    #[test]
    fn test_is_motion_detected_false() {
        let expectations = [PinTransaction::get(State::Low)];

        let pin = PinMock::new(&expectations);
        let delay = NoopDelay::new();
        let mut am312 = Am312::new(pin, delay);

        let res = am312.is_motion_detected().unwrap();
        assert!(!res);

        am312.pin.done();
    }
}
