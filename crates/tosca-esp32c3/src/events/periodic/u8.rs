use core::marker::PhantomData;
use core::pin::Pin;
use core::time::Duration;

use alloc::boxed::Box;

use esp_hal::gpio::AnyPin;

use tosca::events::PeriodicEvent;

use crate::events::EVENTS;

use super::{PeriodicNotifier, notify_network_task};

pub(crate) type PeriodicU8Fn = Box<
    dyn Fn(
            AnyPin<'static>,
            PeriodicNotifier<u8>,
        ) -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>
        + Send
        + Sync
        + 'static,
>;

#[embassy_executor::task]
pub(crate) async fn monitor_periodic_u8_event(
    periodic_event_u8: PeriodicEvent<u8>,
    pin: AnyPin<'static>,
    periodic_u8_notifier: PeriodicNotifier<u8>,
    func: PeriodicU8Fn,
) {
    periodic_u8_notifier.init_event(periodic_event_u8).await;

    // We leak the function since this task will live until the end of the
    // process. We also free the memory.
    let leak = Box::leak(func);

    // Run the function.
    leak(pin, periodic_u8_notifier).await;
}

pub(crate) type PeriodicU8FnPinless = Box<
    dyn Fn(PeriodicNotifier<u8>) -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>
        + Send
        + Sync
        + 'static,
>;

#[embassy_executor::task]
pub(crate) async fn monitor_periodic_u8_event_pinless(
    periodic_event_u8: PeriodicEvent<u8>,
    periodic_u8_notifier: PeriodicNotifier<u8>,
    func: PeriodicU8FnPinless,
) {
    periodic_u8_notifier.init_event(periodic_event_u8).await;

    // We leak the function since this task will live until the end of the
    // process. We also free the memory.
    let leak = Box::leak(func);

    // Run the function.
    leak(periodic_u8_notifier).await;
}

impl PeriodicNotifier<u8> {
    /// Updates the [`PeriodicEvent<u8>`] and then waits for a determined
    /// time interval before checking again the event.
    #[inline]
    pub async fn update_event(&self, value: u8) {
        // Update the u8 value in the shared structure.
        {
            EVENTS
                .lock()
                .await
                .update_periodic_u8_value(self.index, value);
        }
        // Notify the network task and wait for the chosen amount of seconds.
        notify_network_task(self.time_interval.as_secs()).await;
    }

    pub(crate) const fn u8(index: usize, time_interval: Duration) -> Self {
        Self {
            index,
            time_interval,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub(crate) async fn init_event(&self, periodic_event_u8: PeriodicEvent<u8>) {
        {
            EVENTS.lock().await.add_periodic_u8_event(periodic_event_u8);
        }
    }
}
