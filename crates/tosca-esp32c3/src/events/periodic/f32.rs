use core::marker::PhantomData;
use core::pin::Pin;
use core::time::Duration;

use alloc::boxed::Box;

use esp_hal::gpio::AnyPin;

use tosca::events::PeriodicEvent;

use crate::events::EVENTS;

use super::{PeriodicNotifier, notify_network_task};

pub(crate) type PeriodicF32Fn = Box<
    dyn Fn(
            AnyPin<'static>,
            PeriodicNotifier<f32>,
        ) -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>
        + Send
        + Sync
        + 'static,
>;

#[embassy_executor::task]
pub(crate) async fn monitor_periodic_f32_event(
    periodic_event_f32: PeriodicEvent<f32>,
    pin: AnyPin<'static>,
    periodic_f32_notifier: PeriodicNotifier<f32>,
    func: PeriodicF32Fn,
) {
    periodic_f32_notifier.init_event(periodic_event_f32).await;

    // We leak the function since this task will live until the end of the
    // process. We also free the memory.
    let leak = Box::leak(func);

    // Run the function.
    leak(pin, periodic_f32_notifier).await;
}

impl PeriodicNotifier<f32> {
    /// Updates the [`PeriodicEvent<f32>`] and then waits for a determined
    /// time interval before checking again the event.
    #[inline]
    pub async fn update_event(&self, value: f32) {
        // Update the f32 value in the shared structure.
        {
            EVENTS
                .lock()
                .await
                .update_periodic_f32_value(self.index, value);
        }
        // Notify the network task and wait for the chosen amount of seconds.
        notify_network_task(self.time_interval.as_secs()).await;
    }

    pub(crate) const fn f32(index: usize, time_interval: Duration) -> Self {
        Self {
            index,
            time_interval,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub(crate) async fn init_event(&self, periodic_event_f32: PeriodicEvent<f32>) {
        {
            EVENTS
                .lock()
                .await
                .add_periodic_f32_event(periodic_event_f32);
        }
    }
}
