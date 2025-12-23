#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

extern crate alloc;

use core::sync::atomic::{AtomicBool, Ordering};

use tosca::parameters::Parameters;
use tosca::route::{LightOffRoute, LightOnRoute, Route};

use esp_hal::Config;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{AnyPin, Input, InputConfig, Level, Output, OutputConfig, Pin, Pull};
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::rng::Rng;
use esp_hal::timer::timg::TimerGroup;

use log::info;

use embassy_executor::Spawner;
use embassy_net::Ipv4Address;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::Timer;

use tosca_esp32c3::{
    devices::light::Light,
    events::{EventsConfig, EventsManager, broker::BrokerData, interrupt::Notifier},
    mdns::Mdns,
    net::NetworkStack,
    parameters::ParametersPayloads,
    response::{ErrorResponse, OkResponse, SerialResponse},
    server::Server,
    wifi::Wifi,
};

const MAX_HEAP_SIZE: usize = 64 * 1024;
const MILLISECONDS_TO_WAIT: u64 = 100;

// Socket buffer size.
const TX_SIZE: usize = 2048;
// Server buffer size.
const RX_SIZE: usize = 4096;
// Maximum number of allowed headers in a request.
const MAXIMUM_HEADERS_COUNT: usize = 32;
// Timeout.
const TIMEOUT: u32 = 15 * 1000;

// Signal that indicates a change in the LED's state.
static NOTIFY_LED: Signal<CriticalSectionRawMutex, LedInput> = Signal::new();
// Atomic signal to enable and disable the toggle task.
static TOGGLE_CONTROLLER: AtomicBool = AtomicBool::new(false);

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[toml_cfg::toml_config]
struct DeviceConfig {
    #[default("")]
    ssid: &'static str,
    #[default("")]
    password: &'static str,
    #[default("")]
    broker_address: &'static str,
    #[default(0)]
    broker_port: u16,
}

#[derive(Clone, Copy)]
enum LedInput {
    On,
    Off,
    Toggle,
    Button,
}

#[embassy_executor::task]
async fn press_button(mut button: Input<'static>) {
    loop {
        // Wait until the button is pressed.
        button.wait_for_rising_edge().await;
        info!("Button Pressed!");

        // Disable the toggle task.
        TOGGLE_CONTROLLER.store(false, Ordering::Relaxed);

        // Wait for a specified amount of time before notifying the LED.
        Timer::after_millis(MILLISECONDS_TO_WAIT).await;

        // Notify led to change its current state.
        NOTIFY_LED.signal(LedInput::Button);

        // Wait for some time before starting the loop again.
        Timer::after_millis(MILLISECONDS_TO_WAIT).await;
    }
}

// Turn the led on.
#[inline]
fn led_on(led: &mut Output<'static>) {
    led.set_low();
    info!("Led is on!");
}

// Turn the led off.
#[inline]
fn led_off(led: &mut Output<'static>) {
    led.set_high();
    info!("Led is off!");
}

// Toggle the led.
#[inline]
fn toggle_led(led: &mut Output<'static>) {
    // Toggle the LED on or off based on its current state.
    //
    // Since the LED uses a pull-up configuration, a high signal indicates that
    // the LED is turned off.
    if led.is_set_high() {
        led_on(led);
    } else {
        led_off(led);
    }
}

async fn change_led(led: AnyPin<'static>, notifier: Notifier<bool>) {
    // Output led.
    let mut led = Output::new(led, Level::High, OutputConfig::default());

    // Notify the external process of the LED status when the task is initiated
    notifier.update_event(led.is_set_low()).await;

    loop {
        // Wait until a signal is received before proceeding.
        let led_input = NOTIFY_LED.wait().await;

        match led_input {
            LedInput::On => {
                led_on(&mut led);
            }
            LedInput::Off => {
                led_off(&mut led);
            }
            LedInput::Button => {
                toggle_led(&mut led);
            }
            LedInput::Toggle => {
                while TOGGLE_CONTROLLER.load(Ordering::Relaxed) {
                    // Since the toggle operates in seconds, the entire process
                    // should complete in less than 1 second.
                    toggle_led(&mut led);
                    notifier.update_event(led.is_set_low()).await;
                    Timer::after_secs(1).await;
                }
            }
        }
        // Notify at the end of each input operation.
        notifier.update_event(led.is_set_low()).await;

        // Wait for a specified duration before restarting the loop.
        Timer::after_millis(MILLISECONDS_TO_WAIT).await;
    }
}

async fn notify_led(
    led_input: LedInput,
    message: &str,
    text_message: &'static str,
) -> Result<SerialResponse, ErrorResponse> {
    // Disable the toggle task.
    TOGGLE_CONTROLLER.store(false, Ordering::Relaxed);

    // Wait for a specified amount of time before notifying the LED.
    Timer::after_millis(MILLISECONDS_TO_WAIT).await;

    // Notify led to change its current state.
    NOTIFY_LED.signal(led_input);

    log::info!("{message}");

    // Returns a serial response from a text.
    Ok(SerialResponse::text(text_message))
}

async fn turn_light_on(_parameters: ParametersPayloads) -> Result<SerialResponse, ErrorResponse> {
    notify_led(LedInput::On, "Led turned on through PUT route!", "Light on").await
}

async fn turn_light_off(
    mut parameters: ParametersPayloads,
) -> Result<SerialResponse, ErrorResponse> {
    let test_value = parameters.u8("test-value")?.value;

    info!("Test value: {test_value}");

    notify_led(
        LedInput::Off,
        "Led turned off through PUT route!",
        "Light off",
    )
    .await
}

async fn toggle(_parameters: ParametersPayloads) -> Result<OkResponse, ErrorResponse> {
    // Enable the toggle task.
    TOGGLE_CONTROLLER.store(true, Ordering::Relaxed);

    // Notify led.
    NOTIFY_LED.signal(LedInput::Toggle);

    info!("Led toggled through GET route!");

    Ok(OkResponse::new())
}

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();

    let config = Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: MAX_HEAP_SIZE);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_int = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_int.software_interrupt0);

    info!("ESP RTOS started!");

    let rng = Rng::new();

    // Retrieve device configuration.
    let device_config = DEVICE_CONFIG;

    let interfaces = Wifi::configure(peripherals.WIFI, spawner)
        .expect("Failed to configure Wi-Fi")
        .connect(device_config.ssid, device_config.password)
        .await
        .expect("Failed to connect to Wi-Fi");

    // The number of tasks in the stack must be increased depending on the
    // needs. If the number of task is less than the actual number of tasks,
    // there may be malfunctions.
    //
    // In this case, the value is 7 because we have:
    // - 1 server tasks
    // - 1 wifi task
    // - 1 mdns task
    // - 1 stack task
    // - 1 task to check if a button is pressed
    // - 1 task to check if a led state is changed
    // - 1 task to send data to an external broker
    let stack = NetworkStack::build::<7>(rng, interfaces.sta, spawner)
        .await
        .expect("Failed to create network stack.");

    // Input button.
    let button = Input::new(
        peripherals.GPIO9,
        InputConfig::default().with_pull(Pull::Up),
    );

    spawner
        .spawn(press_button(button))
        .expect("Impossible to spawn the task to press the button task");

    let device = Light::new(&interfaces.ap)
        .turn_light_on_stateless_serial(
            LightOnRoute::put("On").description("Turn light on."),
            turn_light_on,
        )
        .turn_light_off_stateless_serial(
            LightOffRoute::put("Off")
                .description("Turn light off.")
                .with_parameters(Parameters::new().u8("test-value", 42)),
            turn_light_off,
        )
        .stateless_ok_route(
            Route::get("Toggle", "/toggle").description("Toggle the light on and off."),
            toggle,
        )
        .build();

    let events_config = EventsConfig::new(
        spawner,
        stack,
        BrokerData::ip(
            device_config
                .broker_address
                .parse()
                .unwrap_or(Ipv4Address::LOCALHOST.into()),
            device_config.broker_port,
        ),
        device,
    );

    let device = EventsManager::config(events_config)
        .bool_event(
            "led",
            "Change the built-in led status",
            change_led,
            peripherals.GPIO8.degrade(),
        )
        .run_network_task()
        .await
        .expect("Failed to run the events manager");

    #[allow(clippy::large_futures)]
    Server::<TX_SIZE, RX_SIZE, MAXIMUM_HEADERS_COUNT, _>::new(device, Mdns::new(rng))
        .keepalive_timeout(TIMEOUT)
        .run(stack, spawner)
        .await
        .expect("Failed to run a server");
}
