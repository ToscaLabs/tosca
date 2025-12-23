use core::marker::PhantomData;
use core::pin::Pin;

use alloc::boxed::Box;

use esp_hal::gpio::AnyPin;

use tosca::events::Event;

use crate::events::EVENTS;

use super::{Notifier, notify_network_task};

pub(crate) type U8Fn = Box<
    dyn Fn(
            AnyPin<'static>,
            Notifier<u8>,
        ) -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>
        + Send
        + Sync
        + 'static,
>;

#[embassy_executor::task]
pub(crate) async fn monitor_u8_event(
    event_u8: Event<u8>,
    pin: AnyPin<'static>,
    u8_notifier: Notifier<u8>,
    func: U8Fn,
) {
    u8_notifier.init_event(event_u8).await;

    // We leak the function since this task will live until the end of the
    // process. We also free the memory.
    let leak = Box::leak(func);

    // Run the function.
    leak(pin, u8_notifier).await;
}

pub(crate) type U8FnPinless = Box<
    dyn Fn(Notifier<u8>) -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>
        + Send
        + Sync
        + 'static,
>;

#[embassy_executor::task]
pub(crate) async fn monitor_u8_event_pinless(
    event_u8: Event<u8>,
    u8_notifier: Notifier<u8>,
    func: U8FnPinless,
) {
    u8_notifier.init_event(event_u8).await;

    // We leak the function since this task will live until the end of the
    // process. We also free the memory.
    let leak = Box::leak(func);

    // Run the function.
    leak(u8_notifier).await;
}

impl Notifier<u8> {
    /// Updates the [`Event<u8>`].
    #[inline]
    pub async fn update_event(&self, value: u8) {
        // Update the u8 event.
        {
            EVENTS.lock().await.update_u8_value(self.index, value);
        }
        notify_network_task().await;
    }

    pub(crate) const fn u8(index: usize) -> Self {
        Self {
            index,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub(crate) async fn init_event(&self, event_u8: Event<u8>) {
        {
            EVENTS.lock().await.add_u8_event(event_u8);
        }
    }
}
