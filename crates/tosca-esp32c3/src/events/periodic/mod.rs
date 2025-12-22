pub(crate) mod bool;
pub(crate) mod f32;
pub(crate) mod f64;
pub(crate) mod i32;
pub(crate) mod u8;

use core::marker::PhantomData;
use core::time::Duration;

use embassy_time::Timer;

use crate::events::{WAIT_FOR_MILLISECONDS, WRITE_ON_NETWORK};

/// A notifier to signal a [`PeriodicEvent`].
pub struct PeriodicNotifier<T: Clone + Copy> {
    index: usize,
    time_interval: Duration,
    phantom: PhantomData<T>,
}

#[inline]
async fn notify_network_task(secs: u64) {
    // Wait for a bit after the writing operation.
    Timer::after_millis(WAIT_FOR_MILLISECONDS).await;
    // Write over the network.
    WRITE_ON_NETWORK.signal(1);
    // Wait for a bit after sending the signal.
    Timer::after_secs(secs).await;
}
