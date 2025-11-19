#![no_std]
#![no_main]

use blinksy_esp::ClocklessRmtError;
use esp_hal::main;
use esp_hal::time::Duration;
use heapless::Vec;
use log::info;
use pir_motion_sensor::lighting::{Driver, LightControl, LightingProfile, Sleeper};
use pir_motion_sensor::motion_detection::MotionDetector;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();
    info!("Starting lighting tests");
    run_tests();
}

fn run_tests() -> ! {
    const TESTS: &[TestCase] = &[
        TestCase {
            name: "turns_off_after_duration_without_motion",
            test: turns_off_after_duration_without_motion,
        },
        TestCase {
            name: "resets_countdown_when_motion_detected_during_window",
            test: resets_countdown_when_motion_detected_during_window,
        },
        TestCase {
            name: "immediately_turns_off_when_grace_exceeds_light_duration",
            test: immediately_turns_off_when_grace_exceeds_light_duration,
        },
    ];

    for case in TESTS {
        info!("Running {}", case.name);
        (case.test)();
        info!("Passed {}", case.name);
    }

    info!("All lighting tests passed");
    loop {
        core::hint::spin_loop();
    }
}

struct TestCase {
    name: &'static str,
    test: fn(),
}

fn turns_off_after_duration_without_motion() {
    let profile = LightingProfile::new(
        Duration::from_millis(500),
        Duration::from_millis(200),
        Duration::from_millis(100),
    );
    let lights = MockLights::default();
    let sleeper = MockSleeper::default();
    let mut driver = Driver::new(lights, sleeper);
    let steps_needed = steps_to_clear(profile);
    let script = repeat_value(false, steps_needed.saturating_add(2));
    let mut detector = MockMotionDetector::with_script(script);

    driver
        .keep_on_until_silence_with_profile(&mut detector, profile)
        .unwrap();

    let (lights, sleeper) = driver.into_parts();
    assert_eq!(lights.events.as_slice(), &[LightEvent::On, LightEvent::Off]);
    assert_eq!(sleeper.delays.first(), Some(&profile.grace_period));
    assert_eq!(
        sleeper
            .delays
            .iter()
            .filter(|&&duration| duration == profile.step)
            .count(),
        steps_needed
    );
}

fn resets_countdown_when_motion_detected_during_window() {
    let profile = LightingProfile::new(
        Duration::from_millis(500),
        Duration::from_millis(200),
        Duration::from_millis(100),
    );
    let lights = MockLights::default();
    let sleeper = MockSleeper::default();
    let mut driver = Driver::new(lights, sleeper);
    let reset_steps = steps_for_duration(profile.light_duration, profile.step);
    let mut sequence = Script::new();
    sequence.extend_from_slice(&[false, false, true]).unwrap();
    let extra = usize::try_from(reset_steps).unwrap_or(0).saturating_add(2);
    for _ in 0..extra {
        sequence.push(false).unwrap();
    }
    let mut detector = MockMotionDetector::with_script(sequence);

    driver
        .keep_on_until_silence_with_profile(&mut detector, profile)
        .unwrap();

    let (lights, sleeper) = driver.into_parts();
    assert_eq!(lights.events.as_slice(), &[LightEvent::On, LightEvent::Off]);
    assert_eq!(
        sleeper
            .delays
            .iter()
            .filter(|&&duration| duration == profile.step)
            .count(),
        reset_steps as usize + 2
    );
}

fn immediately_turns_off_when_grace_exceeds_light_duration() {
    let profile = LightingProfile::new(
        Duration::from_millis(100),
        Duration::from_millis(200),
        Duration::from_millis(50),
    );
    let lights = MockLights::default();
    let sleeper = MockSleeper::default();
    let mut driver = Driver::new(lights, sleeper);
    let script = repeat_value(false, 4);
    let mut detector = MockMotionDetector::with_script(script);

    driver
        .keep_on_until_silence_with_profile(&mut detector, profile)
        .unwrap();

    let (lights, sleeper) = driver.into_parts();
    assert_eq!(lights.events.as_slice(), &[LightEvent::On, LightEvent::Off]);
    assert_eq!(sleeper.delays.as_slice(), &[profile.grace_period]);
}

#[derive(Default, Debug)]
struct MockLights {
    events: EventLog,
}

impl LightControl for MockLights {
    fn light_on(&mut self) -> Result<(), ClocklessRmtError> {
        self.events.push(LightEvent::On).unwrap();
        Ok(())
    }

    fn light_off(&mut self) -> Result<(), ClocklessRmtError> {
        self.events.push(LightEvent::Off).unwrap();
        Ok(())
    }
}

#[derive(Default, Debug)]
struct MockSleeper {
    delays: DelayLog,
}

impl Sleeper for MockSleeper {
    fn delay(&mut self, duration: Duration) {
        self.delays.push(duration).unwrap();
    }
}

#[derive(Debug, PartialEq, Eq)]
enum LightEvent {
    On,
    Off,
}

struct MockMotionDetector {
    readings: Script,
    next: usize,
}

impl MockMotionDetector {
    fn with_script(readings: Script) -> Self {
        Self { readings, next: 0 }
    }
}

impl MotionDetector for MockMotionDetector {
    fn motion_detected(&mut self) -> bool {
        let reading = self.readings.get(self.next).copied().unwrap_or(false);
        self.next += 1;
        reading
    }
}

type EventLog = Vec<LightEvent, 8>;
type DelayLog = Vec<Duration, 32>;
type Script = Vec<bool, 64>;

fn steps_to_clear(profile: LightingProfile) -> usize {
    profile
        .light_duration
        .checked_sub(profile.grace_period)
        .unwrap_or(Duration::ZERO)
        .as_micros()
        .checked_div(profile.step.as_micros())
        .unwrap_or(0) as usize
}

fn steps_for_duration(duration: Duration, step: Duration) -> u32 {
    duration
        .as_micros()
        .checked_div(step.as_micros())
        .unwrap_or(0) as u32
}

fn repeat_value(value: bool, count: usize) -> Script {
    let mut script = Script::new();
    for _ in 0..count {
        script.push(value).unwrap();
    }
    script
}
