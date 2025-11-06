#![no_std]

pub mod notes;

use esp_hal::{
    delay::Delay,
    gpio::{
        DriveMode, Input, InputConfig, Level, Output, OutputConfig, Pull,
        interconnect::PeripheralOutput,
    },
    ledc::{
        HighSpeed, LSGlobalClkSource, Ledc, LowSpeed,
        channel::{self, Channel, ChannelIFace, config::Config as ChannelConfig},
        timer::{
            self, LSClockSource, Timer, TimerIFace,
            config::{Config as TimerConfig, Duty},
        },
    },
    peripherals::Peripherals,
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

    // BUZZER
    // let mut hstimer1 = ledc.timer::<HighSpeed>(timer::Number::Timer1);
    // hstimer1
    //     .configure(timer::config::Config {
    //         duty: timer::config::Duty::Duty10Bit,
    //         clock_source: timer::HSClockSource::APBClk,
    //         frequency: Note::A1.rate(), // Will be replaced
    //     })
    //     .unwrap();
    // let buzzer = Output::new(peripherals.GPIO27, Level::Low, OutputConfig::default());
    // let mut rmt = Rmt::new(peripherals.RMT, Note::A1.rate()).unwrap();
    // let buzzer_channel = rmt
    //     .channel0
    //     .configure_tx(peripherals.GPIO27, TxChannelConfig::default())
    //     .unwrap();
    // buzzer_channel
    //     .transmit(&[PulseCode::new(Level::High, 200, Level::Low, 50); 20])
    //     .unwrap();
    // let buzzer_channel = note_channel(&ledc, &hstimer1, buzzer);

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
            1..=32 => Note::DS4,
            33..=65 => Note::GS7,
            66..=100 => Note::C8,
            0 | 101.. => Note::A1,
        };
        let mut hstimer1 = ledc.timer::<HighSpeed>(timer::Number::Timer1);
        hstimer1
            .configure(timer::config::Config {
                duty: timer::config::Duty::Duty10Bit,
                clock_source: timer::HSClockSource::APBClk,
                frequency: note.rate(),
            })
            .unwrap();
        let buzzer_channel = note_channel(&ledc, &hstimer1, unsafe {
            peripherals.GPIO27.clone_unchecked()
        });
        buzzer_channel
            .set_duty(if brightness_pct == 0 { 0 } else { 50 })
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
fn measure_echo(echo: &Input, real_time_clock: &Rtc) -> Pulse {
    while echo.is_low() {}
    let start = real_time_clock.current_time_us();
    while echo.is_high() {}
    let end = real_time_clock.current_time_us();

    Pulse {
        width_microseconds: (end - start),
    }
}

struct Pulse {
    width_microseconds: u64,
}

impl Pulse {
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

pub fn note_channel<'a>(
    ledc: &Ledc<'a>,
    timer: &'a Timer<'a, HighSpeed>,
    output_pin: impl PeripheralOutput<'a>,
) -> Channel<'a, HighSpeed> {
    let mut channel1 = ledc.channel(channel::Number::Channel1, output_pin);
    channel1
        .configure(channel::config::Config {
            timer,
            duty_pct: 0,
            drive_mode: DriveMode::PushPull,
        })
        .unwrap();
    channel1
}
