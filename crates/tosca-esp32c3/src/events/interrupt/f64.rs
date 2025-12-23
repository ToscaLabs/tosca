use core::marker::PhantomData;
use core::pin::Pin;

use alloc::boxed::Box;

use esp_hal::gpio::AnyPin;

use tosca::events::Event;

use crate::events::EVENTS;

use super::{Notifier, notify_network_task};

pub(crate) type F64Fn = Box<
    dyn Fn(
            AnyPin<'static>,
            Notifier<f64>,
        ) -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>
        + Send
        + Sync
        + 'static,
>;

#[embassy_executor::task]
pub(crate) async fn monitor_f64_event(
    event_f64: Event<f64>,
    pin: AnyPin<'static>,
    f64_notifier: Notifier<f64>,
    func: F64Fn,
) {
    f64_notifier.init_event(event_f64).await;

    // We leak the function since this task will live until the end of the
    // process. We also free the memory.
    let leak = Box::leak(func);

    // Run the function.
    leak(pin, f64_notifier).await;
}

pub(crate) type F64FnPinless = Box<
    dyn Fn(Notifier<f64>) -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>
        + Send
        + Sync
        + 'static,
>;

#[embassy_executor::task]
pub(crate) async fn monitor_f64_event_pinless(
    event_f64: Event<f64>,
    f64_notifier: Notifier<f64>,
    func: F64FnPinless,
) {
    f64_notifier.init_event(event_f64).await;

    // We leak the function since this task will live until the end of the
    // process. We also free the memory.
    let leak = Box::leak(func);

    // Run the function.
    leak(f64_notifier).await;
}

impl Notifier<f64> {
    /// Updates the [`Event<f64>`].
    #[inline]
    pub async fn update_event(&self, value: f64) {
        // Update the f64 event.
        {
            EVENTS.lock().await.update_f64_value(self.index, value);
        }
        notify_network_task().await;
    }

    pub(crate) const fn f64(index: usize) -> Self {
        Self {
            index,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub(crate) async fn init_event(&self, event_f64: Event<f64>) {
        {
            EVENTS.lock().await.add_f64_event(event_f64);
        }
    }
}
