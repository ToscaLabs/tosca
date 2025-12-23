use core::marker::PhantomData;
use core::pin::Pin;
use core::time::Duration;

use alloc::boxed::Box;

use esp_hal::gpio::AnyPin;

use tosca::events::PeriodicEvent;

use crate::events::EVENTS;

use super::{PeriodicNotifier, notify_network_task};

pub(crate) type PeriodicBoolFn = Box<
    dyn Fn(
            AnyPin<'static>,
            PeriodicNotifier<bool>,
        ) -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>
        + Send
        + Sync
        + 'static,
>;

#[embassy_executor::task]
pub(crate) async fn monitor_periodic_bool_event(
    periodic_event_bool: PeriodicEvent<bool>,
    pin: AnyPin<'static>,
    periodic_bool_notifier: PeriodicNotifier<bool>,
    func: PeriodicBoolFn,
) {
    periodic_bool_notifier.init_event(periodic_event_bool).await;

    // We leak the function since this task will live until the end of the
    // process. We also free the memory.
    let leak = Box::leak(func);

    // Run the function.
    leak(pin, periodic_bool_notifier).await;
}

pub(crate) type PeriodicBoolFnPinless = Box<
    dyn Fn(PeriodicNotifier<bool>) -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>
        + Send
        + Sync
        + 'static,
>;

#[embassy_executor::task]
pub(crate) async fn monitor_periodic_bool_event_pinless(
    periodic_event_bool: PeriodicEvent<bool>,
    periodic_bool_notifier: PeriodicNotifier<bool>,
    func: PeriodicBoolFnPinless,
) {
    periodic_bool_notifier.init_event(periodic_event_bool).await;

    // We leak the function since this task will live until the end of the
    // process. We also free the memory.
    let leak = Box::leak(func);

    // Run the function.
    leak(periodic_bool_notifier).await;
}

impl PeriodicNotifier<bool> {
    /// Updates the [`PeriodicEvent<bool>`] and then waits for a determined
    /// time interval before checking again the event.
    #[inline]
    pub async fn update_event(&self, value: bool) {
        // Update the periodic bool event.
        {
            EVENTS
                .lock()
                .await
                .update_periodic_bool_value(self.index, value);
        }
        // Notify the network task and wait for the chosen amount of seconds.
        notify_network_task(self.time_interval.as_secs()).await;
    }

    pub(crate) const fn bool(index: usize, time_interval: Duration) -> Self {
        Self {
            index,
            time_interval,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub(crate) async fn init_event(&self, periodic_event_bool: PeriodicEvent<bool>) {
        {
            EVENTS
                .lock()
                .await
                .add_periodic_bool_event(periodic_event_bool);
        }
    }
}
