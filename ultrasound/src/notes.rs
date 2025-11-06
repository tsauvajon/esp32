use esp_hal::{
    delay::Delay,
    gpio::{DriveMode, interconnect::PeripheralOutput},
    ledc::{
        HighSpeed, Ledc,
        channel::{self, Channel, ChannelIFace},
        timer::{self, Timer, TimerIFace},
    },
    peripherals::Peripherals,
    time::Rate,
};

// Note frequencies in Hertz as f64
pub const NOTE_B0: f64 = 31.0;
pub const NOTE_C1: f64 = 33.0;
pub const NOTE_CS1: f64 = 35.0;
pub const NOTE_D1: f64 = 37.0;
pub const NOTE_DS1: f64 = 39.0;
pub const NOTE_E1: f64 = 41.0;
pub const NOTE_F1: f64 = 44.0;
pub const NOTE_FS1: f64 = 46.0;
pub const NOTE_G1: f64 = 49.0;
pub const NOTE_GS1: f64 = 52.0;
pub const NOTE_A1: f64 = 55.0;
pub const NOTE_AS1: f64 = 58.0;
pub const NOTE_B1: f64 = 62.0;
pub const NOTE_C2: f64 = 65.0;
pub const NOTE_CS2: f64 = 69.0;
pub const NOTE_D2: f64 = 73.0;
pub const NOTE_DS2: f64 = 78.0;
pub const NOTE_E2: f64 = 82.0;
pub const NOTE_F2: f64 = 87.0;
pub const NOTE_FS2: f64 = 93.0;
pub const NOTE_G2: f64 = 98.0;
pub const NOTE_GS2: f64 = 104.0;
pub const NOTE_A2: f64 = 110.0;
pub const NOTE_AS2: f64 = 117.0;
pub const NOTE_B2: f64 = 123.0;
pub const NOTE_C3: f64 = 131.0;
pub const NOTE_CS3: f64 = 139.0;
pub const NOTE_D3: f64 = 147.0;
pub const NOTE_DS3: f64 = 156.0;
pub const NOTE_E3: f64 = 165.0;
pub const NOTE_F3: f64 = 175.0;
pub const NOTE_FS3: f64 = 185.0;
pub const NOTE_G3: f64 = 196.0;
pub const NOTE_GS3: f64 = 208.0;
pub const NOTE_A3: f64 = 220.0;
pub const NOTE_AS3: f64 = 233.0;
pub const NOTE_B3: f64 = 247.0;
pub const NOTE_C4: f64 = 262.0;
pub const NOTE_CS4: f64 = 277.0;
pub const NOTE_D4: f64 = 294.0;
pub const NOTE_DS4: f64 = 311.0;
pub const NOTE_E4: f64 = 330.0;
pub const NOTE_F4: f64 = 349.0;
pub const NOTE_FS4: f64 = 370.0;
pub const NOTE_G4: f64 = 392.0;
pub const NOTE_GS4: f64 = 415.0;
pub const NOTE_A4: f64 = 440.0;
pub const NOTE_AS4: f64 = 466.0;
pub const NOTE_B4: f64 = 494.0;
pub const NOTE_C5: f64 = 523.0;
pub const NOTE_CS5: f64 = 554.0;
pub const NOTE_D5: f64 = 587.0;
pub const NOTE_DS5: f64 = 622.0;
pub const NOTE_E5: f64 = 659.0;
pub const NOTE_F5: f64 = 698.0;
pub const NOTE_FS5: f64 = 740.0;
pub const NOTE_G5: f64 = 784.0;
pub const NOTE_GS5: f64 = 831.0;
pub const NOTE_A5: f64 = 880.0;
pub const NOTE_AS5: f64 = 932.0;
pub const NOTE_B5: f64 = 988.0;
pub const NOTE_C6: f64 = 1047.0;
pub const NOTE_CS6: f64 = 1109.0;
pub const NOTE_D6: f64 = 1175.0;
pub const NOTE_DS6: f64 = 1245.0;
pub const NOTE_E6: f64 = 1319.0;
pub const NOTE_F6: f64 = 1397.0;
pub const NOTE_FS6: f64 = 1480.0;
pub const NOTE_G6: f64 = 1568.0;
pub const NOTE_GS6: f64 = 1661.0;
pub const NOTE_A6: f64 = 1760.0;
pub const NOTE_AS6: f64 = 1865.0;
pub const NOTE_B6: f64 = 1976.0;
pub const NOTE_C7: f64 = 2093.0;
pub const NOTE_CS7: f64 = 2217.0;
pub const NOTE_D7: f64 = 2349.0;
pub const NOTE_DS7: f64 = 2489.0;
pub const NOTE_E7: f64 = 2637.0;
pub const NOTE_F7: f64 = 2794.0;
pub const NOTE_FS7: f64 = 2960.0;
pub const NOTE_G7: f64 = 3136.0;
pub const NOTE_GS7: f64 = 3322.0;
pub const NOTE_A7: f64 = 3520.0;
pub const NOTE_AS7: f64 = 3729.0;
pub const NOTE_B7: f64 = 3951.0;
pub const NOTE_C8: f64 = 4186.0;
pub const NOTE_CS8: f64 = 4435.0;
pub const NOTE_D8: f64 = 4699.0;
pub const NOTE_DS8: f64 = 4978.0;
pub const REST: f64 = 0.0; // No sound, for pauses

pub const PINK_PANTHER_TEMPO: u16 = 120;
pub const PINK_PANTHER_MELODY: [(f64, i16); 88] = [
    (REST, 2),
    (REST, 4),
    (REST, 8),
    (NOTE_DS4, 8),
    (NOTE_E4, -4),
    (REST, 8),
    (NOTE_FS4, 8),
    (NOTE_G4, -4),
    (REST, 8),
    (NOTE_DS4, 8),
    (NOTE_E4, -8),
    (NOTE_FS4, 8),
    (NOTE_G4, -8),
    (NOTE_C5, 8),
    (NOTE_B4, -8),
    (NOTE_E4, 8),
    (NOTE_G4, -8),
    (NOTE_B4, 8),
    (NOTE_AS4, 2),
    (NOTE_A4, -16),
    (NOTE_G4, -16),
    (NOTE_E4, -16),
    (NOTE_D4, -16),
    (NOTE_E4, 2),
    (REST, 4),
    (REST, 8),
    (NOTE_DS4, 4),
    (NOTE_E4, -4),
    (REST, 8),
    (NOTE_FS4, 8),
    (NOTE_G4, -4),
    (REST, 8),
    (NOTE_DS4, 8),
    (NOTE_E4, -8),
    (NOTE_FS4, 8),
    (NOTE_G4, -8),
    (NOTE_C5, 8),
    (NOTE_B4, -8),
    (NOTE_G4, 8),
    (NOTE_B4, -8),
    (NOTE_E5, 8),
    (NOTE_DS5, 1),
    (NOTE_D5, 2),
    (REST, 4),
    (REST, 8),
    (NOTE_DS4, 8),
    (NOTE_E4, -4),
    (REST, 8),
    (NOTE_FS4, 8),
    (NOTE_G4, -4),
    (REST, 8),
    (NOTE_DS4, 8),
    (NOTE_E4, -8),
    (NOTE_FS4, 8),
    (NOTE_G4, -8),
    (NOTE_C5, 8),
    (NOTE_B4, -8),
    (NOTE_E4, 8),
    (NOTE_G4, -8),
    (NOTE_B4, 8),
    (NOTE_AS4, 2),
    (NOTE_A4, -16),
    (NOTE_G4, -16),
    (NOTE_E4, -16),
    (NOTE_D4, -16),
    (NOTE_E4, -4),
    (REST, 4),
    (REST, 4),
    (NOTE_E5, -8),
    (NOTE_D5, 8),
    (NOTE_B4, -8),
    (NOTE_A4, 8),
    (NOTE_G4, -8),
    (NOTE_E4, -8),
    (NOTE_AS4, 16),
    (NOTE_A4, -8),
    (NOTE_AS4, 16),
    (NOTE_A4, -8),
    (NOTE_AS4, 16),
    (NOTE_A4, -8),
    (NOTE_AS4, 16),
    (NOTE_A4, -8),
    (NOTE_G4, -16),
    (NOTE_E4, -16),
    (NOTE_D4, -16),
    (NOTE_E4, 16),
    (NOTE_E4, 16),
    (NOTE_E4, 2),
];

pub struct Song {
    whole_note: u32,
}

impl Song {
    pub fn new(tempo: u16) -> Self {
        let whole_note = (60_000 * 4) / tempo as u32;
        Self { whole_note }
    }

    pub fn calc_note_duration(&self, divider: i16) -> u32 {
        if divider > 0 {
            self.whole_note / divider as u32
        } else {
            let duration = self.whole_note / divider.unsigned_abs() as u32;
            (duration as f64 * 1.5) as u32
        }
    }
}

pub struct Note {
    frequency: Rate,
}

impl Note {
    pub fn from_frequency(freq: u32) -> Self {
        Self {
            frequency: Rate::from_hz(freq),
        }
    }

    pub fn build_timer<'a>(&self, ledc: &Ledc<'a>) -> Timer<'a, HighSpeed> {
        let mut hstimer0 = ledc.timer::<HighSpeed>(timer::Number::Timer0);
        hstimer0
            .configure(timer::config::Config {
                duty: timer::config::Duty::Duty10Bit,
                clock_source: timer::HSClockSource::APBClk,
                frequency: self.frequency,
            })
            .unwrap();
        hstimer0
    }
}

pub fn play_note<'a>(
    ledc: &Ledc<'a>,
    timer: &'a Timer<'a, HighSpeed>,
    output_pin: impl PeripheralOutput<'a>,
) -> Channel<'a, HighSpeed> {
    let mut channel0 = ledc.channel(channel::Number::Channel0, output_pin);
    channel0
        .configure(channel::config::Config {
            timer,
            duty_pct: 50,
            drive_mode: DriveMode::PushPull,
        })
        .unwrap();
    channel0
}

pub fn play_pink_panther_theme(peripherals: Peripherals) {
    let ledc = Ledc::new(peripherals.LEDC);
    let mut hstimer0 = ledc.timer::<HighSpeed>(timer::Number::Timer0);

    let delay = Delay::new();
    let song = Song::new(PINK_PANTHER_TEMPO);
    for (note, duration_type) in PINK_PANTHER_MELODY {
        let note_duration = song.calc_note_duration(duration_type);
        if note == REST {
            delay.delay_millis(note_duration);
            continue;
        }

        let frequency = Rate::from_hz(note as u32);
        hstimer0
            .configure(timer::config::Config {
                duty: timer::config::Duty::Duty10Bit,
                clock_source: timer::HSClockSource::APBClk,
                frequency,
            })
            .unwrap();

        let mut channel0 = ledc.channel(channel::Number::Channel0, unsafe {
            Peripherals::steal().GPIO27
        });
        channel0
            .configure(channel::config::Config {
                timer: &hstimer0,
                duty_pct: 50,
                drive_mode: DriveMode::PushPull,
            })
            .unwrap();

        let pause_duration = note_duration / 10; // 10% of note_duration
        delay.delay_millis(note_duration - pause_duration); // play 90%
        channel0.set_duty(0).unwrap();
        delay.delay_millis(pause_duration); // Pause for 10%
    }
}
