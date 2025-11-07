#![no_std]

pub mod notes;

use esp_hal::{
    delay::Delay,
    gpio::{DriveMode, Input, InputConfig, Level, Output, OutputConfig, OutputPin, Pull},
    ledc::{
        Ledc, LowSpeed,
        channel::{self, ChannelIFace, config::Config as ChannelConfig},
        timer::{
            self, LSClockSource, Timer, TimerIFace,
            config::{Config as TimerConfig, Duty},
        },
    },
    peripherals::Peripherals,
    rmt::{PulseCode, Rmt, TxChannelConfig, TxChannelCreator},
    rtc_cntl::Rtc,
    time::Rate,
};

use crate::notes::{Jukebox, Note};

const MAX_USEFUL_DISTANCE_CM: f64 = 60.0;
const SOUND_CM_PER_MICROSECOND: f64 = 0.0343;
const ROUND_TRIP_SEGMENTS: f64 = 2.0;

// Note: on ESP32 and ESP32-S2 you cannot specify a base frequency other than 80 MHz
const MANDATORY_RMT_MHZ_FREQUENCY_FOR_ESP32: u32 = 80;

pub fn run_rear_parking_sensor(peripherals: Peripherals) -> ! {
    // Ultrasound
    let mut trigger = Output::new(peripherals.GPIO5, Level::Low, OutputConfig::default());
    let echo = Input::new(
        peripherals.GPIO18,
        InputConfig::default().with_pull(Pull::Down),
    );
    let real_time_clock = Rtc::new(peripherals.LPWR);

    // LED
    let ledc = Ledc::new(peripherals.LEDC);
    let mut lstimer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    lstimer0
        .configure(TimerConfig {
            duty: Duty::Duty5Bit,
            clock_source: LSClockSource::APBClk,
            frequency: Rate::from_khz(24),
        })
        .unwrap();
    let led_channel = ledc_channel(&ledc, peripherals.GPIO32, &lstimer0);

    // RMT Buzzer
    let rmt = Rmt::new(
        peripherals.RMT,
        Rate::from_mhz(MANDATORY_RMT_MHZ_FREQUENCY_FOR_ESP32),
    )
    .unwrap();
    let buzzer_channel = TxChannelCreator::configure_tx(
        rmt.channel2,
        peripherals.GPIO27,
        TxChannelConfig::default().with_clk_divider(MANDATORY_RMT_MHZ_FREQUENCY_FOR_ESP32 as u8), // 80 MHz / 80 = 1 MhZ clock
    )
    .unwrap();
    let mut jukebox = Jukebox::new(buzzer_channel);

    loop {
        send_ultrasound_wave(&mut trigger);
        let pulse = measure_echo(&echo, &real_time_clock);
        let distance = pulse.distance_cm();

        let brightness_pct = brightness_percentage(distance);
        // info!("Calculated a distance of {distance:.0} cm. Led brightness of {brightness_pct}%");
        led_channel.set_duty(brightness_pct).unwrap();

        let note = match brightness_pct {
            1..=25 => Note::C4,
            26..=50 => Note::E4,
            51..=75 => Note::A4,
            76..=100 => Note::C6,
            0 | 101.. => Note::Silence,
        };
        jukebox = jukebox.play_note(note).unwrap();

        Delay::new().delay_millis(10);
    }
}

fn ledc_channel<'a>(
    ledc: &'a Ledc,
    led: impl OutputPin + 'a,
    lstimer: &'a Timer<'a, LowSpeed>,
) -> channel::Channel<'a, LowSpeed> {
    let led = Output::new(led, Level::Low, OutputConfig::default());
    let mut led_channel = ledc.channel(channel::Number::Channel0, led);
    led_channel
        .configure(ChannelConfig {
            timer: lstimer,
            duty_pct: 0,
            drive_mode: DriveMode::PushPull,
        })
        .unwrap();
    led_channel
}

fn brightness_percentage(distance: f64) -> u8 {
    if distance > MAX_USEFUL_DISTANCE_CM {
        return 0;
    }

    let ratio = (MAX_USEFUL_DISTANCE_CM - distance) / MAX_USEFUL_DISTANCE_CM;
    let brightness = (ratio * 100.0) as u8;
    brightness.min(100)
}

fn send_ultrasound_wave(trigger: &mut Output<'_>) {
    let delay = Delay::new(); // TODO: take an `impl DelayNs` instead

    // Ensure the Trigger pin is low before starting
    trigger.set_low();
    delay.delay_micros(2);

    // Send a 10-microseconds high pulse
    trigger.set_high();
    delay.delay_micros(10);
    trigger.set_low();
}

/// Reads the pulse width in microseconds, which is equal to the delay between waves
fn measure_echo(echo: &Input<'_>, real_time_clock: &Rtc<'_>) -> UltrasoundPulse {
    while echo.is_low() {}
    let start = real_time_clock.current_time_us();
    while echo.is_high() {}
    let end = real_time_clock.current_time_us();

    UltrasoundPulse {
        width_microseconds: (end - start),
    }
}

struct UltrasoundPulse {
    width_microseconds: u64,
}

impl UltrasoundPulse {
    /// To calculate the distance, we need to use the pulse width.
    /// The pulse width tells us how long it took for the ultrasonic waves to
    /// travel to an obstacle and return.
    /// Since the pulse represents the round-trip time, we divide it by 2 to
    /// account for the journey to the obstacle and back.
    ///
    /// The speed of sound in air is approximately 0.0343 cm per microsecond.
    /// By multiplying the time (in microseconds) by this value and dividing
    /// by 2, we obtain the distance to the obstacle in centimeters.
    fn distance_cm(&self) -> f64 {
        (self.width_microseconds as f64 * SOUND_CM_PER_MICROSECOND) / ROUND_TRIP_SEGMENTS
    }
}

/// Build the "note equivalent" of the brightness.
///
/// Assumption: brightness goes from 0 to 100.
///
/// The sound it produces is quite bad in practice though.
#[deprecated]
fn _brightness_to_note_pulse(brightess_percent: u8) -> PulseCode {
    if brightess_percent == 0 {
        return PulseCode::new(Level::Low, 1, Level::Low, 1);
    }

    let darkest_note = Rate::from(Note::B0).as_hz() as f32;
    let brightest_note = Rate::from(Note::DS8).as_hz() as f32;
    let frequency_ratio = brightest_note / darkest_note;

    let brightness_ratio = (brightess_percent as f32 - 1.0) / (100.0 - 1.0);
    // Notes don't have a linear distribution, so we need to weigh it down
    let weighted_frequency = darkest_note * libm::powf(frequency_ratio, brightness_ratio);
    let weighted_frequency = libm::roundf(weighted_frequency) as u16;

    PulseCode::new(
        Level::High,
        weighted_frequency,
        Level::Low,
        weighted_frequency,
    )
}
