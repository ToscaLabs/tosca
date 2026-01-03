use embassy_executor::Spawner;

use esp_hal::peripherals::WIFI;

use esp_radio::Controller;
use esp_radio::wifi::{
    ClientConfig, Config, Interfaces, ModeConfig, WifiController, WifiEvent, WifiStaState,
    sta_state,
};

use log::{error, info};

use crate::error::{Error, ErrorKind, Result};
use crate::mk_static;

pub(crate) const WIFI_RECONNECT_DELAY: u64 = 2;

/// The `Wi-Fi` controller.
///
/// Configures and establishes a conncection to a `Wi-Fi` access point.
pub struct Wifi {
    _esp_radio_controller: &'static Controller<'static>,
    controller: WifiController<'static>,
    interfaces: Interfaces<'static>,
    spawner: Spawner,
}

impl Wifi {
    /// Configures the [`Wifi`] controller with the given parameters.
    ///
    /// # Errors
    ///
    /// Failed to initialize the `Wi-Fi` controller and retrieve the
    /// network interfaces.
    pub fn configure(peripherals_wifi: WIFI<'static>, spawner: Spawner) -> Result<Self> {
        let esp_radio_controller = &*mk_static!(Controller<'static>, esp_radio::init()?);
        let (controller, interfaces) =
            esp_radio::wifi::new(esp_radio_controller, peripherals_wifi, Config::default())?;

        Ok(Self {
            _esp_radio_controller: esp_radio_controller,
            controller,
            interfaces,
            spawner,
        })
    }

    /// Connects a device to a `Wi-Fi` access point.
    ///
    /// # Errors
    ///
    /// - Missing `Wi-Fi` SSID
    /// - Missing `Wi-Fi` password
    /// - Failed to configure the `Wi-Fi` settings
    /// - Failed to spawn the task for connecting the device to the access
    ///   point via `Wi-Fi`.
    pub async fn connect(mut self, ssid: &str, password: &str) -> Result<Interfaces<'static>> {
        if ssid.is_empty() {
            return Err(Error::new(ErrorKind::WiFi, "Missing Wi-Fi SSID"));
        }

        if password.is_empty() {
            return Err(Error::new(ErrorKind::WiFi, "Missing Wi-Fi password"));
        }

        let client_config = ModeConfig::Client(
            ClientConfig::default()
                .with_ssid(ssid.into())
                .with_password(password.into()),
        );

        self.controller.set_config(&client_config)?;

        self.spawner.spawn(connect(self.controller))?;

        // Wait until Wi-Fi is connected.
        while sta_state() != WifiStaState::Connected {
            embassy_time::Timer::after_millis(100).await;
        }

        Ok(self.interfaces)
    }
}

#[embassy_executor::task]
async fn connect(mut wifi_controller: WifiController<'static>) {
    info!("Wi-Fi connection task started");
    loop {
        if sta_state() == WifiStaState::Connected {
            wifi_controller
                .wait_for_event(WifiEvent::StaDisconnected)
                .await;
            embassy_time::Timer::after_secs(WIFI_RECONNECT_DELAY).await;
        }

        if !matches!(wifi_controller.is_started(), Ok(true)) {
            info!("Starting Wi-Fi...");
            wifi_controller
                .start_async()
                .await
                .map_err(Error::from)
                .expect("Impossible to start Wi-Fi");
            info!("Wi-Fi started");
        }

        info!("Attempting to connect...");
        if let Err(e) = wifi_controller.connect_async().await {
            error!("Wi-Fi connect failed: {e:?}");
            embassy_time::Timer::after_secs(WIFI_RECONNECT_DELAY).await;
        } else {
            info!("Wi-Fi connected!");
        }
    }
}
