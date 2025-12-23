use core::marker::PhantomData;
use core::pin::Pin;

use alloc::boxed::Box;

use esp_hal::gpio::AnyPin;

use tosca::events::Event;

use crate::events::EVENTS;

use super::{Notifier, notify_network_task};

pub(crate) type I32Fn = Box<
    dyn Fn(
            AnyPin<'static>,
            Notifier<i32>,
        ) -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>
        + Send
        + Sync
        + 'static,
>;

#[embassy_executor::task]
pub(crate) async fn monitor_i32_event(
    event_i32: Event<i32>,
    pin: AnyPin<'static>,
    i32_notifier: Notifier<i32>,
    func: I32Fn,
) {
    i32_notifier.init_event(event_i32).await;

    // We leak the function since this task will live until the end of the
    // process. We also free the memory.
    let leak = Box::leak(func);

    // Run the function.
    leak(pin, i32_notifier).await;
}

pub(crate) type I32FnPinless = Box<
    dyn Fn(Notifier<i32>) -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>
        + Send
        + Sync
        + 'static,
>;

#[embassy_executor::task]
pub(crate) async fn monitor_i32_event_pinless(
    event_i32: Event<i32>,
    i32_notifier: Notifier<i32>,
    func: I32FnPinless,
) {
    i32_notifier.init_event(event_i32).await;

    // We leak the function since this task will live until the end of the
    // process. We also free the memory.
    let leak = Box::leak(func);

    // Run the function.
    leak(i32_notifier).await;
}

impl Notifier<i32> {
    /// Updates the [`Event<i32>`].
    #[inline]
    pub async fn update_event(&self, value: i32) {
        // Update the i32 event.
        {
            EVENTS.lock().await.update_i32_value(self.index, value);
        }
        notify_network_task().await;
    }

    pub(crate) const fn i32(index: usize) -> Self {
        Self {
            index,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub(crate) async fn init_event(&self, event_i32: Event<i32>) {
        {
            EVENTS.lock().await.add_i32_event(event_i32);
        }
    }
}
