use embassy_executor::Spawner;
use embassy_time::Timer;

use esp_hal::peripherals::WIFI;

use esp_radio::wifi::{
    AuthenticationMethodConfig, Config, ControllerConfig, Interface, WifiController,
    sta::StationConfig,
};

use log::{error, info};

use crate::error::{Error, ErrorKind, Result};

pub(crate) const WIFI_RECONNECT_DELAY: u64 = 2;

/// The `Wi-Fi` controller.
///
/// Configures and establishes a connection to a `Wi-Fi` access point.
pub struct Wifi {
    controller: WifiController<'static>,
    interface: Interface,
    spawner: Spawner,
}

impl Wifi {
    /// Configures the [`Wifi`] controller with the given parameters.
    ///
    /// # Errors
    ///
    /// - Failed to initialize the `Wi-Fi` controller.
    /// - The station interface has already been acquired.
    pub fn configure(peripherals_wifi: WIFI<'static>, spawner: Spawner) -> Result<Self> {
        let controller = WifiController::new(peripherals_wifi, ControllerConfig::default())?;

        let interface = Interface::try_station().ok_or_else(|| {
            Error::new(
                ErrorKind::WiFi,
                "Wi-Fi station interface has already been acquired",
            )
        })?;

        Ok(Self {
            controller,
            interface,
            spawner,
        })
    }

    /// Connects a device to a `Wi-Fi` access point.
    ///
    /// # Errors
    ///
    /// - Missing `Wi-Fi` SSID.
    /// - Missing `Wi-Fi` password.
    /// - Invalid `Wi-Fi` SSID or password.
    /// - Failed to configure the `Wi-Fi` settings.
    /// - Failed to spawn the task for reconnecting the device via `Wi-Fi`.
    pub async fn connect(mut self, ssid: &str, password: &str) -> Result<Interface> {
        if ssid.is_empty() {
            return Err(Error::new(ErrorKind::WiFi, "Missing Wi-Fi SSID"));
        }

        if password.is_empty() {
            return Err(Error::new(ErrorKind::WiFi, "Missing Wi-Fi password"));
        }

        // Configure the device as a Wi-Fi station, meaning it acts as a client
        // and connects to a password-protected access point using WPA2 Personal.
        let station_config = Config::Station(
            StationConfig::default()
                .with_ssid(ssid.try_into()?)
                .with_authentication(AuthenticationMethodConfig::Wpa2Personal(
                    password.try_into()?,
                )),
        );

        self.controller.set_config(&station_config)?;

        // Wait until Wi-Fi is connected.
        connect_with_retry(&mut self.controller).await;

        self.spawner.spawn(connect(self.controller)?);

        Ok(self.interface)
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
