pub(crate) mod bool;
pub(crate) mod f32;
pub(crate) mod f64;
pub(crate) mod i32;
pub(crate) mod u8;

use core::marker::PhantomData;

use embassy_time::Timer;

use crate::events::{WAIT_FOR_MILLISECONDS, WRITE_ON_NETWORK};

#[inline]
async fn notify_network_task() {
    // Wait for a bit after the writing operation.
    Timer::after_millis(WAIT_FOR_MILLISECONDS).await;
    // Write over the network.
    WRITE_ON_NETWORK.signal(1);
    // Wait for a bit after sending the signal.
    Timer::after_millis(WAIT_FOR_MILLISECONDS).await;
}

/// A notifier to signal an [`Event`].
pub struct Notifier<T: Clone + Copy> {
    index: usize,
    phantom: PhantomData<T>,
}
