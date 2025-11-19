use blinksy_esp::ClocklessRmtError;
use esp_hal::delay::Delay;
use esp_hal::time::Duration;
use log::info;

use crate::led_strip::RmtControl;
use crate::motion_detection::MotionDetector;

pub const LIGHT_DURATION: Duration = Duration::from_secs(6);
pub const GRACE_PERIOD: Duration = Duration::from_secs(3);
pub const STEP: Duration = Duration::from_millis(100);

pub trait LightControl {
    fn light_on(&mut self) -> Result<(), ClocklessRmtError>;
    fn light_off(&mut self) -> Result<(), ClocklessRmtError>;
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
    fn light_on(&mut self) -> Result<(), ClocklessRmtError> {
        self.control.set_brightness(0.05);
        self.render()
    }

    fn light_off(&mut self) -> Result<(), ClocklessRmtError> {
        self.control.set_brightness(0.0);
        self.render()
    }
}

pub trait Sleeper {
    fn delay(&mut self, duration: Duration);
}

impl Sleeper for Delay {
    fn delay(&mut self, duration: Duration) {
        Delay::delay(self, duration);
    }
}

pub struct Driver<L, S> {
    lights: L,
    sleeper: S,
}

impl<L, S> Driver<L, S> {
    pub fn new(lights: L, sleeper: S) -> Self {
        Self { lights, sleeper }
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
        self.lights.light_on()
    }

    pub fn light_off(&mut self) -> Result<(), ClocklessRmtError> {
        self.lights.light_off()
    }

    pub fn delay_for(&mut self, duration: Duration) {
        self.sleeper.delay(duration);
    }

    pub fn keep_on_until_silence(
        &mut self,
        motion_sensor: &mut impl MotionDetector,
    ) -> Result<(), ClocklessRmtError> {
        self.keep_on_until_silence_with_profile(motion_sensor, LightingProfile::default())
    }

    pub fn keep_on_until_silence_with_profile(
        &mut self,
        motion_sensor: &mut impl MotionDetector,
        profile: LightingProfile,
    ) -> Result<(), ClocklessRmtError> {
        self.light_on()?;

        self.delay_for(profile.grace_period);
        let mut time_remaining = profile
            .light_duration
            .checked_sub(profile.grace_period)
            .unwrap_or(Duration::ZERO);
        loop {
            if motion_sensor.motion_detected() {
                info!("Motion reset!");
                time_remaining = profile.light_duration;
            }

            if time_remaining == Duration::ZERO {
                info!("Clear");
                self.light_off()?;
                return Ok(());
            }

            self.delay_for(profile.step);
            time_remaining = time_remaining
                .checked_sub(profile.step)
                .unwrap_or(Duration::ZERO);
        }
    }
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
