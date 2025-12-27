use core::sync::atomic::{AtomicBool, Ordering};
use core::{convert::TryFrom, time::Duration};

use embassy_executor::task;
use embassy_time::{Duration as EmbassyDuration, Timer};
use esp_hal::gpio::Input;

pub trait MotionDetector {
    fn motion_detected(&mut self) -> bool;
}

pub struct PirMotionSensor<'p>(Input<'p>);

impl<'p> PirMotionSensor<'p> {
    pub fn new(pin: Input<'p>) -> Self {
        Self(pin)
    }
}

impl<'p> MotionDetector for PirMotionSensor<'p> {
    fn motion_detected(&mut self) -> bool {
        self.0.is_high()
    }
}

pub struct MotionState {
    detected: AtomicBool,
}

impl MotionState {
    pub const fn new() -> Self {
        Self {
            detected: AtomicBool::new(false),
        }
    }

    pub fn update(&self, detected: bool) {
        self.detected.store(detected, Ordering::Relaxed);
    }

    pub fn is_detected(&self) -> bool {
        self.detected.load(Ordering::Relaxed)
    }
}

impl Default for MotionState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SharedMotionDetector<'a> {
    state: &'a MotionState,
}

impl<'a> SharedMotionDetector<'a> {
    pub fn new(state: &'a MotionState) -> Self {
        Self { state }
    }
}

impl MotionDetector for SharedMotionDetector<'_> {
    fn motion_detected(&mut self) -> bool {
        self.state.is_detected()
    }
}

#[task]
pub async fn monitor_motion(
    sensor: &'static mut PirMotionSensor<'static>,
    state: &'static MotionState,
    interval: Duration,
) -> ! {
    let interval = EmbassyDuration::try_from(interval).unwrap_or(EmbassyDuration::MAX);
    loop {
        let detected = sensor.motion_detected();
        state.update(detected);
        Timer::after(interval).await;
    }
}
