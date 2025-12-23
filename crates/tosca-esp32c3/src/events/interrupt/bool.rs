use core::marker::PhantomData;
use core::pin::Pin;

use alloc::boxed::Box;

use esp_hal::gpio::AnyPin;

use tosca::events::Event;

use crate::events::EVENTS;

use super::{Notifier, notify_network_task};

pub(crate) type BoolFn = Box<
    dyn Fn(
            AnyPin<'static>,
            Notifier<bool>,
        ) -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>
        + Send
        + Sync
        + 'static,
>;

#[embassy_executor::task]
pub(crate) async fn monitor_bool_event(
    event_bool: Event<bool>,
    pin: AnyPin<'static>,
    bool_notifier: Notifier<bool>,
    func: BoolFn,
) {
    bool_notifier.init_event(event_bool).await;

    // We leak the function since this task will live until the end of the
    // process. We also free the memory.
    let leak = Box::leak(func);

    // Run the function.
    leak(pin, bool_notifier).await;
}

pub(crate) type BoolFnPinless = Box<
    dyn Fn(Notifier<bool>) -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>
        + Send
        + Sync
        + 'static,
>;

#[embassy_executor::task]
pub(crate) async fn monitor_bool_event_pinless(
    event_bool: Event<bool>,
    bool_notifier: Notifier<bool>,
    func: BoolFnPinless,
) {
    bool_notifier.init_event(event_bool).await;

    // We leak the function since this task will live until the end of the
    // process. We also free the memory.
    let leak = Box::leak(func);

    // Run the function.
    leak(bool_notifier).await;
}

impl Notifier<bool> {
    /// Updates the [`Event<bool>`].
    #[inline]
    pub async fn update_event(&self, value: bool) {
        // Update the bool event.
        {
            EVENTS.lock().await.update_bool_value(self.index, value);
        }
        // Notify network task.
        notify_network_task().await;
    }

    pub(crate) const fn bool(index: usize) -> Self {
        Self {
            index,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub(crate) async fn init_event(&self, event_bool: Event<bool>) {
        {
            EVENTS.lock().await.add_bool_event(event_bool);
        }
    }
}
