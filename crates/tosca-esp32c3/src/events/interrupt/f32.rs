use core::marker::PhantomData;
use core::pin::Pin;

use alloc::boxed::Box;

use esp_hal::gpio::AnyPin;

use tosca::events::Event;

use crate::events::EVENTS;

use super::{Notifier, notify_network_task};

pub(crate) type F32Fn = Box<
    dyn Fn(
            AnyPin<'static>,
            Notifier<f32>,
        ) -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>
        + Send
        + Sync
        + 'static,
>;

#[embassy_executor::task]
pub(crate) async fn monitor_f32_event(
    event_f32: Event<f32>,
    pin: AnyPin<'static>,
    f32_notifier: Notifier<f32>,
    func: F32Fn,
) {
    f32_notifier.init_event(event_f32).await;

    // We leak the function since this task will live until the end of the
    // process. We also free the memory.
    let leak = Box::leak(func);

    // Run the function.
    leak(pin, f32_notifier).await;
}

impl Notifier<f32> {
    /// Updates the [`Event<f32>`].
    #[inline]
    pub async fn update_event(&self, value: f32) {
        // Update the f32 event.
        {
            EVENTS.lock().await.update_f32_value(self.index, value);
        }
        notify_network_task().await;
    }

    pub(crate) const fn f32(index: usize) -> Self {
        Self {
            index,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub(crate) async fn init_event(&self, event_f32: Event<f32>) {
        {
            EVENTS.lock().await.add_f32_event(event_f32);
        }
    }
}
