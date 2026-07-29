use embassy_executor::Spawner;
use embassy_time::Timer;

use esp_hal::peripherals::WIFI;

use esp_radio::wifi::{Config, ControllerConfig, Interfaces, WifiController, sta::StationConfig};

use log::{error, info};

use crate::error::{Error, ErrorKind, Result};

pub(crate) const WIFI_RECONNECT_DELAY: u64 = 2;

/// The `Wi-Fi` controller.
///
/// Configures and establishes a connection to a `Wi-Fi` access point.
pub struct Wifi {
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
        let (controller, interfaces) =
            esp_radio::wifi::new(peripherals_wifi, ControllerConfig::default())?;

        Ok(Self {
            controller,
            interfaces,
            spawner,
        })
    }

    /// Connects a device to a `Wi-Fi` access point.
    ///
    /// # Errors
    ///
    /// - Missing `Wi-Fi` SSID.
    /// - Missing `Wi-Fi` password.
    /// - Failed to configure the `Wi-Fi` settings.
    /// - Failed to spawn the task for connecting the device to the access.
    ///   point via `Wi-Fi`.
    pub async fn connect(mut self, ssid: &str, password: &str) -> Result<Interfaces<'static>> {
        if ssid.is_empty() {
            return Err(Error::new(ErrorKind::WiFi, "Missing Wi-Fi SSID"));
        }

        if password.is_empty() {
            return Err(Error::new(ErrorKind::WiFi, "Missing Wi-Fi password"));
        }

        // Configure the device as a Wi-Fi station, meaning it acts as a client
        // and connects to an existing access point using the given credentials.
        let station_config = Config::Station(
            StationConfig::default()
                .with_ssid(ssid)
                .with_password(password.into()),
        );

        self.controller.set_config(&station_config)?;

        // Wait until Wi-Fi is connected.
        connect_with_retry(&mut self.controller).await;

        self.spawner.spawn(connect(self.controller)?);

        Ok(self.interfaces)
    }
}

async fn connect_with_retry(wifi_controller: &mut WifiController<'_>) {
    loop {
        info!("Attempting to connect...");

        if let Err(e) = wifi_controller.connect_async().await {
            error!("Wi-Fi connect failed: {e:?}");
            Timer::after_secs(WIFI_RECONNECT_DELAY).await;
        } else {
            info!("Wi-Fi connected!");
            break;
        }
    }
}

#[embassy_executor::task]
async fn connect(mut wifi_controller: WifiController<'static>) {
    info!("Wi-Fi connection task started");
    loop {
        if let Err(e) = wifi_controller.wait_for_disconnect_async().await {
            error!("Failed while waiting for Wi-Fi disconnection: {e:?}");
        }
        Timer::after_secs(WIFI_RECONNECT_DELAY).await;
        connect_with_retry(&mut wifi_controller).await;
    }
}
