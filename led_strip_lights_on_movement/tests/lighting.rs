#![no_std]
#![no_main]

use blinksy_esp::ClocklessRmtError;
use core::fmt::Debug;
use esp_hal::main;
use esp_hal::time::Duration;
use heapless::Vec;
use pir_motion_sensor::lighting::{Driver, LightControl, LightingProfile, Sleeper};
use pir_motion_sensor::motion_detection::MotionDetector;

esp_bootloader_esp_idf::esp_app_desc!();

const TESTS: &[TestCase] = &[
    TestCase {
        name: "lighting::turns_off_after_duration_without_motion",
        test: turns_off_after_duration_without_motion,
    },
    TestCase {
        name: "lighting::resets_countdown_when_motion_detected_during_window",
        test: resets_countdown_when_motion_detected_during_window,
    },
    TestCase {
        name: "lighting::immediately_turns_off_when_grace_exceeds_light_duration",
        test: immediately_turns_off_when_grace_exceeds_light_duration,
    },
];

const MAX_TESTS: usize = 8;
type FailureLog = Vec<Failure, MAX_TESTS>;

#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();
    run_tests();
}

fn run_tests() -> ! {
    esp_println::println!("running {} tests", TESTS.len());

    let mut passed = 0usize;
    let mut failures: FailureLog = Vec::new();

    for case in TESTS {
        esp_println::print!("test {} ... ", case.name);
        match (case.test)() {
            Ok(()) => {
                passed += 1;
                esp_println::println!("ok");
            }
            Err(err) => {
                let _ = failures.push(Failure {
                    name: case.name,
                    message: err.message,
                });
                esp_println::println!("FAILED");
            }
        }
    }

    let failed = failures.len();
    let ignored = TESTS.len().saturating_sub(passed + failed);
    let status = if failed == 0 { "ok" } else { "FAILED" };

    if failed > 0 {
        esp_println::println!();
        esp_println::println!("failures:");
        for failure in failures.iter() {
            esp_println::println!("---- {} ----", failure.name);
            esp_println::println!("{}", failure.message);
        }
    }

    esp_println::println!();
    esp_println::println!(
        "test result: {}. {} passed; {} failed; {} ignored; 0 measured; 0 filtered out",
        status,
        passed,
        failed,
        ignored
    );

    loop {
        core::hint::spin_loop();
    }
}

struct TestCase {
    name: &'static str,
    test: fn() -> TestResult,
}

#[derive(Clone, Copy)]
struct Failure {
    name: &'static str,
    message: &'static str,
}

type TestResult = Result<(), TestError>;

#[derive(Clone, Copy)]
struct TestError {
    message: &'static str,
}

impl TestError {
    const fn new(message: &'static str) -> Self {
        Self { message }
    }
}

fn turns_off_after_duration_without_motion() -> TestResult {
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
        .map_err(|_| TestError::new("driver failed to keep lights on"))?;

    let (lights, sleeper) = driver.into_parts();
    ensure_eq(
        lights.events.as_slice(),
        &[LightEvent::On, LightEvent::Off],
        "lights should toggle on/off exactly once",
    )?;
    let first_delay = sleeper.delays.first();
    ensure_eq(
        &first_delay,
        &Some(&profile.grace_period),
        "driver must wait for grace period",
    )?;
    let step_delays = sleeper
        .delays
        .iter()
        .filter(|&&duration| duration == profile.step)
        .count();
    ensure_eq(
        &step_delays,
        &steps_needed,
        "driver should step countdown until timeout",
    )?;

    Ok(())
}

fn resets_countdown_when_motion_detected_during_window() -> TestResult {
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
        .map_err(|_| TestError::new("driver failed to keep lights on"))?;

    let (lights, sleeper) = driver.into_parts();
    ensure_eq(
        lights.events.as_slice(),
        &[LightEvent::On, LightEvent::Off],
        "lights should toggle on/off exactly once",
    )?;
    let step_delays = sleeper
        .delays
        .iter()
        .filter(|&&duration| duration == profile.step)
        .count();
    let expected = reset_steps as usize + 2;
    ensure_eq(
        &step_delays,
        &expected,
        "driver should restart countdown after motion",
    )?;

    Ok(())
}

fn immediately_turns_off_when_grace_exceeds_light_duration() -> TestResult {
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
        .map_err(|_| TestError::new("driver failed to keep lights on"))?;

    let (lights, sleeper) = driver.into_parts();
    ensure_eq(
        lights.events.as_slice(),
        &[LightEvent::On, LightEvent::Off],
        "lights should toggle on/off exactly once",
    )?;
    ensure_eq(
        sleeper.delays.as_slice(),
        &[profile.grace_period],
        "driver should exit immediately when grace exceeds duration",
    )?;

    Ok(())
}

/// Libtest-style assertions must not panic or the runner cannot print a summary,
/// so this helper logs mismatches and returns a `TestResult` instead of using `assert_eq!`.
fn ensure_eq<T>(left: &T, right: &T, message: &'static str) -> TestResult
where
    T: PartialEq + Debug + ?Sized,
{
    if left != right {
        log::error!("{}: left={:?}, right={:?}", message, left, right);
        Err(TestError::new(message))
    } else {
        Ok(())
    }
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

/// No std::vec, so this replaces it
fn repeat_value(value: bool, count: usize) -> Script {
    let mut script = Script::new();
    for _ in 0..count {
        script.push(value).unwrap();
    }
    script
}
