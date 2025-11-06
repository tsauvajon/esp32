#![no_std]

pub mod notes;

use esp_hal::{
    delay::Delay,
    gpio::{DriveMode, Input, InputConfig, Level, Output, OutputConfig, Pull},
    ledc::{
        LSGlobalClkSource, Ledc, LowSpeed,
        channel::{self, ChannelIFace, config::Config as ChannelConfig},
        timer::{
            self, LSClockSource, TimerIFace,
            config::{Config as TimerConfig, Duty},
        },
    },
    peripherals::Peripherals,
    rmt::{LoopMode, PulseCode, Rmt, TxChannelConfig, TxChannelCreator},
    rtc_cntl::Rtc,
    time::Rate,
};

use crate::notes::Note;

const MAX_USEFUL_DISTANCE_CM: f64 = 60.0;
const SOUND_CM_PER_MICROSECOND: f64 = 0.0343;
const ROUND_TRIP_SEGMENTS: f64 = 2.0;

pub fn run(peripherals: Peripherals) -> ! {
    // LED
    let led = Output::new(peripherals.GPIO32, Level::Low, OutputConfig::default());
    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);
    let mut lstimer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    lstimer0
        .configure(TimerConfig {
            duty: Duty::Duty5Bit,
            clock_source: LSClockSource::APBClk,
            frequency: Rate::from_khz(24),
        })
        .unwrap();
    let mut led_channel = ledc.channel(channel::Number::Channel0, led);
    led_channel
        .configure(ChannelConfig {
            timer: &lstimer0,
            duty_pct: 10,
            drive_mode: DriveMode::PushPull,
        })
        .unwrap();

    // RMT Buzzer
    let rmt = Rmt::new(peripherals.RMT, Rate::from_mhz(80)).unwrap();
    let buzzer_channel = TxChannelCreator::configure_tx(
        rmt.channel2,
        peripherals.GPIO27,
        TxChannelConfig::default().with_clk_divider(80), // 80 MhZ / 80 = 1 MhZ clock
    )
    .unwrap();
    let mut buzzer_tx = buzzer_channel
        .transmit_continuously(&[PulseCode::from(Note::Silence); 1], LoopMode::Infinite)
        .unwrap();

    // Emit ultrasound waves
    let mut trigger = Output::new(peripherals.GPIO5, Level::Low, OutputConfig::default());

    // Receive ultrasound waves
    let echo = Input::new(
        peripherals.GPIO18,
        InputConfig::default().with_pull(Pull::Down),
    );

    let real_time_clock = Rtc::new(peripherals.LPWR);

    loop {
        send_wave(&mut trigger);
        let pulse = measure_echo(&echo, &real_time_clock);
        let distance = pulse.distance_cm();

        let brightness_pct = brightness_percentage(distance);
        // info!("Calculated a distance of {distance:.0} cm. Led brightness of {brightness_pct}%");
        led_channel.set_duty(brightness_pct).unwrap();

        let note = match brightness_pct {
            1..=25 => Note::C6,
            26..=50 => Note::A4,
            51..=75 => Note::E4,
            76..=100 => Note::C4,
            0 | 101.. => Note::Silence,
        };
        // Mutates the variable to always keep transmitting a note.
        // On each loop iteration, stop transmitting and transmit a new note.
        buzzer_tx = buzzer_tx
            .stop()
            .unwrap()
            .transmit_continuously(&[PulseCode::from(note); 1], LoopMode::Infinite)
            .unwrap();

        Delay::new().delay_millis(10);
    }
}

fn brightness_percentage(distance: f64) -> u8 {
    if distance > MAX_USEFUL_DISTANCE_CM {
        return 0;
    }

    let ratio = (MAX_USEFUL_DISTANCE_CM - distance) / MAX_USEFUL_DISTANCE_CM;
    let brightness = (ratio * 100.0) as u8;
    brightness.min(100)
}

fn send_wave(trigger: &mut Output) {
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
fn measure_echo(echo: &Input, real_time_clock: &Rtc) -> UltrasoundPulse {
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
