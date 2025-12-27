use blinksy_esp::ClocklessRmtError;
use core::{convert::TryFrom, time::Duration};
use embassy_time::{Duration as EmbassyDuration, Timer};
use log::info;

use crate::led_strip::RmtControl;
use crate::motion_detection::MotionDetector;

pub const LIGHT_DURATION: Duration = Duration::from_secs(15);
pub const GRACE_PERIOD: Duration = Duration::from_secs(4);
pub const STEP: Duration = Duration::from_millis(100);
pub const TARGET_BRIGHTNESS: f32 = 0.09;
pub const MIN_BRIGHTNESS: f32 = 0.04;
const CHECK_INTERVAL: Duration = Duration::from_millis(25);

pub trait LightControl {
    fn set_brightness(&mut self, brightness: f32) -> Result<(), ClocklessRmtError>;

    fn light_on(&mut self) -> Result<(), ClocklessRmtError> {
        self.set_brightness(TARGET_BRIGHTNESS)
    }

    fn light_off(&mut self) -> Result<(), ClocklessRmtError> {
        self.set_brightness(0.0)
    }
}

pub struct LedStrip<'a> {
    control: RmtControl<'a>,
}

impl<'a> LedStrip<'a> {
    pub fn new(control: RmtControl<'a>) -> Self {
        Self { control }
    }

    fn render(&mut self) -> Result<(), ClocklessRmtError> {
        let elapsed_in_ms = blinksy_esp::time::elapsed().as_millis();
        self.control.tick(elapsed_in_ms)
    }
}

impl<'a> LightControl for LedStrip<'a> {
    fn set_brightness(&mut self, brightness: f32) -> Result<(), ClocklessRmtError> {
        self.control.set_brightness(brightness);
        self.render()
    }
}

#[allow(async_fn_in_trait)]
pub trait Sleeper {
    async fn delay(&mut self, duration: Duration);
}

#[derive(Default)]
pub struct EmbassySleeper;

impl Sleeper for EmbassySleeper {
    async fn delay(&mut self, duration: Duration) {
        let duration = EmbassyDuration::try_from(duration).unwrap_or(EmbassyDuration::MAX);
        Timer::after(duration).await;
    }
}

pub struct Driver<L, S> {
    lights: L,
    sleeper: S,
    current_brightness: f32,
}

impl<L, S> Driver<L, S> {
    pub fn new(lights: L, sleeper: S) -> Self {
        Self {
            lights,
            sleeper,
            current_brightness: 0.0,
        }
    }

    pub fn into_parts(self) -> (L, S) {
        (self.lights, self.sleeper)
    }
}

impl<L, S> Driver<L, S>
where
    L: LightControl,
    S: Sleeper,
{
    pub fn light_on(&mut self) -> Result<(), ClocklessRmtError> {
        self.set_brightness(TARGET_BRIGHTNESS)
    }

    pub fn light_off(&mut self) -> Result<(), ClocklessRmtError> {
        self.set_brightness(0.0)
    }

    pub async fn delay_for(&mut self, duration: Duration) {
        self.sleeper.delay(duration).await;
    }

    fn set_brightness(&mut self, brightness: f32) -> Result<(), ClocklessRmtError> {
        let clamped = if brightness <= 0.0 {
            0.0
        } else {
            brightness.clamp(MIN_BRIGHTNESS, 1.0)
        };
        self.current_brightness = clamped;
        self.lights.set_brightness(clamped)
    }

    pub async fn keep_on_until_silence(
        &mut self,
        motion_sensor: &mut impl MotionDetector,
    ) -> Result<(), ClocklessRmtError> {
        self.keep_on_until_silence_with_profile(
            motion_sensor,
            LightingProfile {
                light_duration: Duration::from_secs(15),
                ..LightingProfile::default()
            },
        )
        .await
    }

    pub async fn keep_on_until_silence_with_profile(
        &mut self,
        motion_sensor: &mut impl MotionDetector,
        profile: LightingProfile,
    ) -> Result<(), ClocklessRmtError> {
        let mut monitor = MotionMonitor::new(motion_sensor, profile.step);

        loop {
            self.set_brightness(TARGET_BRIGHTNESS)?;
            match self.hold_for(profile.light_duration, &mut monitor).await? {
                TransitionOutcome::MotionDetected => {
                    info!("Motion reset!");
                    continue;
                }
                TransitionOutcome::Completed => {
                    info!("Clear");
                    self.light_off()?;
                    return Ok(());
                }
            }
        }
    }

    async fn hold_for<M: MotionDetector>(
        &mut self,
        duration: Duration,
        monitor: &mut MotionMonitor<'_, M>,
    ) -> Result<TransitionOutcome, ClocklessRmtError> {
        if duration == Duration::ZERO {
            return Ok(TransitionOutcome::Completed);
        }

        let mut elapsed = Duration::ZERO;
        while elapsed < duration {
            if monitor.should_check() && monitor.check() {
                return Ok(TransitionOutcome::MotionDetected);
            }

            let remaining = duration.checked_sub(elapsed).unwrap_or(Duration::ZERO);
            if remaining == Duration::ZERO {
                break;
            }

            let chunk = next_chunk_duration(remaining, monitor);
            if chunk == Duration::ZERO {
                continue;
            }

            self.delay_for(chunk).await;
            elapsed = elapsed.checked_add(chunk).unwrap_or(duration);
            monitor.advance(chunk);
        }

        Ok(TransitionOutcome::Completed)
    }
}

fn next_chunk_duration<M: MotionDetector>(
    remaining: Duration,
    monitor: &MotionMonitor<'_, M>,
) -> Duration {
    let mut chunk = remaining.min(CHECK_INTERVAL);
    let limit = monitor.remaining_until_check();
    if limit != Duration::ZERO && limit < chunk {
        chunk = limit;
    }
    chunk
}

struct MotionMonitor<'a, M: MotionDetector> {
    detector: &'a mut M,
    step: Duration,
    time_until_next_check: Duration,
}

impl<'a, M: MotionDetector> MotionMonitor<'a, M> {
    fn new(detector: &'a mut M, step: Duration) -> Self {
        Self {
            detector,
            step,
            time_until_next_check: Duration::ZERO,
        }
    }

    fn should_check(&self) -> bool {
        self.time_until_next_check == Duration::ZERO
    }

    fn check(&mut self) -> bool {
        let detected = self.detector.motion_detected();
        self.time_until_next_check = self.interval();
        detected
    }

    fn advance(&mut self, duration: Duration) {
        if duration >= self.time_until_next_check {
            self.time_until_next_check = Duration::ZERO;
        } else {
            self.time_until_next_check = self
                .time_until_next_check
                .checked_sub(duration)
                .unwrap_or(Duration::ZERO);
        }
    }

    fn remaining_until_check(&self) -> Duration {
        if self.step == Duration::ZERO {
            CHECK_INTERVAL
        } else {
            self.time_until_next_check
        }
    }

    fn interval(&self) -> Duration {
        if self.step == Duration::ZERO {
            CHECK_INTERVAL
        } else {
            self.step
        }
    }
}

enum TransitionOutcome {
    Completed,
    MotionDetected,
}

#[derive(Clone, Copy)]
pub struct LightingProfile {
    pub light_duration: Duration,
    pub grace_period: Duration,
    pub step: Duration,
}

impl LightingProfile {
    pub const fn new(light_duration: Duration, grace_period: Duration, step: Duration) -> Self {
        Self {
            light_duration,
            grace_period,
            step,
        }
    }
}

impl Default for LightingProfile {
    fn default() -> Self {
        Self::new(LIGHT_DURATION, GRACE_PERIOD, STEP)
    }
}
