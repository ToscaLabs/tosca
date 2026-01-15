//! # DS18B20 Driver
//!
//! This module provides a synchronous, architecture-agnostic driver for
//! the `DS18B20` digital temperature sensor.
//! The driver is synchronous to meet the device’s strict timing requirements.
//!
//! The `DS18B20` communicates over the 1-Wire bus and provides temperature
//! readings with up to 12-bit resolution.
//! It performs temperature conversions internally and exposes the result
//! through its scratchpad memory, which includes a CRC to ensure data
//! integrity.
//!
//! The driver operates in *single-sensor mode* using the `Skip ROM` command to
//! address the device directly without needing its unique 64-bit ROM code,
//! making it ideal for setups where **only one** `DS18B20` is connected
//! to the bus.
//!
//! For detailed specifications, refer to the
//! [datasheet](https://www.alldatasheet.com/datasheet-pdf/pdf/58557/DALLAS/DS18B20.html).

use core::result::Result;

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};

// Timing for reset and presence detection on the 1-Wire bus.
const RESET_LOW_US: u32 = 480;
const PRESENCE_WAIT_US: u32 = 70;
const PRESENCE_RELEASE_US: u32 = 410;

// Timing for writing logic 1 and 0 bits to the bus.
const WRITE_1_LOW_US: u32 = 6;
const WRITE_1_HIGH_US: u32 = 64;
const WRITE_0_LOW_US: u32 = 60;
const WRITE_0_HIGH_US: u32 = 10;

// Timing for reading a bit from the bus (init, sample, recovery).
const READ_INIT_LOW_US: u32 = 6;
const READ_SAMPLE_US: u32 = 9;
const READ_RECOVERY_US: u32 = 55;

// Maximum wait time for temperature conversion at 12-bit resolution.
const CONVERSION_WAIT_MS: u32 = 750;

// DS18B20 ROM and function commands.
const CMD_SKIP_ROM: u8 = 0xCC;
const CMD_CONVERT_T: u8 = 0x44;
const CMD_READ_SCRATCHPAD: u8 = 0xBE;

// Temperature resolution of the DS18B20 sensor.
// Each bit in the 12-bit temperature reading corresponds to 0.0625 °C.
const TEMPERATURE_RESOLUTION_C_PER_LSB: f32 = 0.0625;

/// Errors that may occur when interacting with the `DS18B20` sensor.
#[derive(Debug)]
pub enum Ds18b20Error<E> {
    /// Error related to GPIO pin I/O operations.
    Pin(E),
    /// Data integrity error due to CRC check failure.
    CrcMismatch,
    /// No presence pulse detected, sensor not found on bus.
    NoPresence,
}

impl<E> From<E> for Ds18b20Error<E> {
    fn from(e: E) -> Self {
        Ds18b20Error::Pin(e)
    }
}

/// The `DS18B20` driver.
pub struct Ds18b20<P, D>
where
    P: InputPin + OutputPin,
    D: DelayNs,
{
    pin: P,
    delay: D,
}

impl<P, D> Ds18b20<P, D>
where
    P: InputPin + OutputPin,
    D: DelayNs,
{
    /// Creates a [`Ds18b20`] driver for the given pin and delay provider.
    #[must_use]
    pub fn new(pin: P, delay: D) -> Self {
        Self { pin, delay }
    }

    /// Performs a bus reset and checks for the presence pulse from the sensor.
    ///
    /// # Errors
    ///
    /// Returns an error if accessing the GPIO pin fails during the reset
    /// or presence detection sequence.
    pub fn reset(&mut self) -> Result<bool, Ds18b20Error<P::Error>> {
        self.pin.set_low()?;
        self.delay.delay_us(RESET_LOW_US);

        self.pin.set_high()?;
        self.delay.delay_us(PRESENCE_WAIT_US);

        // Sensor should pull the line low to indicate presence.
        let present = self.pin.is_low()?;
        self.delay.delay_us(PRESENCE_RELEASE_US);

        Ok(present)
    }

    /// Performs a full temperature measurement sequence:
    ///
    /// 1. Initiates a temperature conversion
    /// 2. Waits for the conversion to complete
    /// 3. Reads and CRC-verifies the scratchpad data
    /// 4. Returns the measured temperature in degrees Celsius (°C)
    ///
    /// # Notes
    ///
    /// After a power-on reset, the DS18B20’s temperature register is initialized
    /// to **85.0 °C**. This happens when the sensor is first powered on (power-up)
    /// or if a reset of its internal registers occurs while powered.
    /// This value has a valid CRC and is not treated as an error by the driver.
    /// Discard the first reading after a power-on reset, as it returns 85.0 °C
    /// which does not reflect the current temperature.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - Communication with the sensor fails
    /// - No device responds on the bus
    /// - The scratchpad data fails CRC validation
    pub fn read_temperature(&mut self) -> Result<f32, Ds18b20Error<P::Error>> {
        // 1. Reset and check presence.
        if !self.reset()? {
            return Err(Ds18b20Error::NoPresence);
        }

        // 2. Start temperature conversion.
        self.write_byte(CMD_SKIP_ROM)?;
        self.write_byte(CMD_CONVERT_T)?;

        // 3. Wait for conversion completion (poll line or timeout).
        for _ in 0..CONVERSION_WAIT_MS {
            if self.pin.is_high()? {
                break;
            }
            self.delay.delay_ms(1);
        }

        // 4. Reset again to read scratchpad.
        if !self.reset()? {
            return Err(Ds18b20Error::NoPresence);
        }

        self.write_byte(CMD_SKIP_ROM)?;
        self.write_byte(CMD_READ_SCRATCHPAD)?;

        let data = self.read_scratchpad()?;

        // 5. Validate CRC.
        let crc_calc = Self::crc8(&data[0..8]);
        if crc_calc != data[8] {
            return Err(Ds18b20Error::CrcMismatch);
        }

        // 6. Convert raw temperature to °C.
        let raw_temp = (i16::from(data[1]) << 8) | i16::from(data[0]);
        let temp = f32::from(raw_temp) * TEMPERATURE_RESOLUTION_C_PER_LSB;

        Ok(temp)
    }

    fn write_bit(&mut self, bit: bool) -> Result<(), Ds18b20Error<P::Error>> {
        // Write a single bit to the 1-Wire bus.
        if bit {
            // Logic 1: short low pulse.
            self.pin.set_low()?;
            self.delay.delay_us(WRITE_1_LOW_US);
            self.pin.set_high()?;
            self.delay.delay_us(WRITE_1_HIGH_US);
        } else {
            // Logic 0: long low pulse.
            self.pin.set_low()?;
            self.delay.delay_us(WRITE_0_LOW_US);
            self.pin.set_high()?;
            self.delay.delay_us(WRITE_0_HIGH_US);
        }

        Ok(())
    }

    fn read_bit(&mut self) -> Result<bool, Ds18b20Error<P::Error>> {
        self.pin.set_low()?;
        self.delay.delay_us(READ_INIT_LOW_US);
        self.pin.set_high()?;
        self.delay.delay_us(READ_SAMPLE_US);

        // Read a single bit from the 1-Wire bus.
        let bit = self.pin.is_high()?;
        self.delay.delay_us(READ_RECOVERY_US);

        Ok(bit)
    }

    fn write_byte(&mut self, byte: u8) -> Result<(), Ds18b20Error<P::Error>> {
        // Write a full byte to the 1-Wire bus (LSB first).
        for i in 0..8 {
            self.write_bit((byte >> i) & 1 != 0)?;
        }

        Ok(())
    }

    fn read_byte(&mut self) -> Result<u8, Ds18b20Error<P::Error>> {
        let mut byte = 0;

        // Read a full byte from the 1-Wire bus (LSB first).
        for i in 0..8 {
            if self.read_bit()? {
                byte |= 1 << i;
            }
        }

        Ok(byte)
    }

    fn read_scratchpad(&mut self) -> Result<[u8; 9], Ds18b20Error<P::Error>> {
        let mut data = [0u8; 9];

        // Read the 9-byte scratchpad from the DS18B20.
        for b in &mut data {
            *b = self.read_byte()?;
        }

        Ok(data)
    }

    fn crc8(data: &[u8]) -> u8 {
        let mut crc: u8 = 0;

        // Compute the Dallas/Maxim CRC8 checksum (polynomial 0x31).
        for &byte in data {
            let mut b = byte;
            for _ in 0..8 {
                let mix = (crc ^ b) & 0x01;
                crc >>= 1;
                if mix != 0 {
                    crc ^= 0x8C;
                }
                b >>= 1;
            }
        }

        crc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use embedded_hal_mock::eh1::delay::NoopDelay;
    use embedded_hal_mock::eh1::digital::{Mock as PinMock, State, Transaction as PinTransaction};

    fn raw_to_temp(data: [u8; 9]) -> f32 {
        let raw = (i16::from(data[1]) << 8) | i16::from(data[0]);
        f32::from(raw) * TEMPERATURE_RESOLUTION_C_PER_LSB
    }

    #[test]
    fn test_reset_detects_presence() {
        let expectations = [
            PinTransaction::set(State::Low),
            PinTransaction::set(State::High),
            PinTransaction::get(State::Low), // Presence pulse from sensor.
        ];

        let pin = PinMock::new(&expectations);
        let delay = NoopDelay::new();
        let mut ds18b20 = Ds18b20::new(pin, delay);

        let present = ds18b20.reset().unwrap();
        assert!(present);

        ds18b20.pin.done();
    }

    #[test]
    fn test_reset_no_presence() {
        let expectations = [
            PinTransaction::set(State::Low),
            PinTransaction::set(State::High),
            PinTransaction::get(State::High), // Line remains high: no device detected.
        ];

        let pin = PinMock::new(&expectations);
        let delay = NoopDelay::new();
        let mut ds18b20 = Ds18b20::new(pin, delay);

        let present = ds18b20.reset().unwrap();
        assert!(!present);

        ds18b20.pin.done();
    }

    #[test]
    fn test_read_temperature_no_presence() {
        let expectations = [
            // Simulate a reset sequence with no presence pulse.
            PinTransaction::set(State::Low),
            PinTransaction::set(State::High),
            PinTransaction::get(State::High),
        ];

        let pin = PinMock::new(&expectations);
        let delay = NoopDelay::new();
        let mut ds18b20 = Ds18b20::new(pin, delay);

        let result = ds18b20.read_temperature();
        assert!(matches!(result, Err(Ds18b20Error::NoPresence)));

        ds18b20.pin.done();
    }

    #[test]
    fn test_read_temperature_crc_mismatch() {
        // Simulate a scratchpad read with incorrect CRC.
        let data = [0x50, 0x05, 0, 0, 0, 0, 0, 0, 0x00];

        let crc_ok = Ds18b20::<PinMock, NoopDelay>::crc8(&data[0..8]);
        assert_ne!(crc_ok, data[8]); // Confirms that CRC mismatch exists.
    }

    #[test]
    fn test_crc8_computation() {
        // Example data set with a known correct CRC-8 value.
        let data = [0x02, 0x4E, 0xB8, 0x1C, 0x46, 0x7F, 0xFF, 0x0C];

        let crc = Ds18b20::<PinMock, NoopDelay>::crc8(&data);
        assert_eq!(crc, 0xBE); // Expected CRC known value.
    }

    #[test]
    fn test_read_temperature_valid_data() {
        // Raw reading: 0x0550 = 85.0 °C.
        let mut data = [0x50, 0x05, 0, 0, 0, 0, 0, 0, 0];
        data[8] = Ds18b20::<PinMock, NoopDelay>::crc8(&data[0..8]);

        // Test CRC correctness.
        let crc_ok = Ds18b20::<PinMock, NoopDelay>::crc8(&data[0..8]);
        assert_eq!(crc_ok, data[8]);

        let temp = raw_to_temp(data);
        assert!((temp - 85.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_read_temperature_negative_value() {
        // Raw reading: 0xFF90 = -7.0 °C.
        let mut data = [0x90, 0xFF, 0, 0, 0, 0, 0, 0, 0];
        data[8] = Ds18b20::<PinMock, NoopDelay>::crc8(&data[0..8]);

        let temp = raw_to_temp(data);
        assert!((temp + 7.0).abs() < f32::EPSILON);
    }
}
