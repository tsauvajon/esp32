use esp_hal::{
    delay::Delay,
    gpio::DriveMode,
    ledc::{
        HighSpeed, Ledc,
        channel::{self, ChannelIFace},
        timer::{self, TimerIFace},
    },
    peripherals::Peripherals,
    time::Rate,
};

pub enum Note {
    B0,
    C1,
    CS1,
    D1,
    DS1,
    E1,
    F1,
    FS1,
    G1,
    GS1,
    A1,
    AS1,
    B1,
    C2,
    CS2,
    D2,
    DS2,
    E2,
    F2,
    FS2,
    G2,
    GS2,
    A2,
    AS2,
    B2,
    C3,
    CS3,
    D3,
    DS3,
    E3,
    F3,
    FS3,
    G3,
    GS3,
    A3,
    AS3,
    B3,
    C4,
    CS4,
    D4,
    DS4,
    E4,
    F4,
    FS4,
    G4,
    GS4,
    A4,
    AS4,
    B4,
    C5,
    CS5,
    D5,
    DS5,
    E5,
    F5,
    FS5,
    G5,
    GS5,
    A5,
    AS5,
    B5,
    C6,
    CS6,
    D6,
    DS6,
    E6,
    F6,
    FS6,
    G6,
    GS6,
    A6,
    AS6,
    B6,
    C7,
    CS7,
    D7,
    DS7,
    E7,
    F7,
    FS7,
    G7,
    GS7,
    A7,
    AS7,
    B7,
    C8,
    CS8,
    D8,
    DS8,
}

impl Note {
    pub fn rate(&self) -> Rate {
        Rate::from_hz(match self {
            Note::B0 => 31,
            Note::C1 => 33,
            Note::CS1 => 35,
            Note::D1 => 37,
            Note::DS1 => 39,
            Note::E1 => 41,
            Note::F1 => 44,
            Note::FS1 => 46,
            Note::G1 => 49,
            Note::GS1 => 52,
            Note::A1 => 55,
            Note::AS1 => 58,
            Note::B1 => 62,
            Note::C2 => 65,
            Note::CS2 => 69,
            Note::D2 => 73,
            Note::DS2 => 78,
            Note::E2 => 82,
            Note::F2 => 87,
            Note::FS2 => 93,
            Note::G2 => 98,
            Note::GS2 => 104,
            Note::A2 => 110,
            Note::AS2 => 117,
            Note::B2 => 123,
            Note::C3 => 131,
            Note::CS3 => 139,
            Note::D3 => 147,
            Note::DS3 => 156,
            Note::E3 => 165,
            Note::F3 => 175,
            Note::FS3 => 185,
            Note::G3 => 196,
            Note::GS3 => 208,
            Note::A3 => 220,
            Note::AS3 => 233,
            Note::B3 => 247,
            Note::C4 => 262,
            Note::CS4 => 277,
            Note::D4 => 294,
            Note::DS4 => 311,
            Note::E4 => 330,
            Note::F4 => 349,
            Note::FS4 => 370,
            Note::G4 => 392,
            Note::GS4 => 415,
            Note::A4 => 440,
            Note::AS4 => 466,
            Note::B4 => 494,
            Note::C5 => 523,
            Note::CS5 => 554,
            Note::D5 => 587,
            Note::DS5 => 622,
            Note::E5 => 659,
            Note::F5 => 698,
            Note::FS5 => 740,
            Note::G5 => 784,
            Note::GS5 => 831,
            Note::A5 => 880,
            Note::AS5 => 932,
            Note::B5 => 988,
            Note::C6 => 1047,
            Note::CS6 => 1109,
            Note::D6 => 1175,
            Note::DS6 => 1245,
            Note::E6 => 1319,
            Note::F6 => 1397,
            Note::FS6 => 1480,
            Note::G6 => 1568,
            Note::GS6 => 1661,
            Note::A6 => 1760,
            Note::AS6 => 1865,
            Note::B6 => 1976,
            Note::C7 => 2093,
            Note::CS7 => 2217,
            Note::D7 => 2349,
            Note::DS7 => 2489,
            Note::E7 => 2637,
            Note::F7 => 2794,
            Note::FS7 => 2960,
            Note::G7 => 3136,
            Note::GS7 => 3322,
            Note::A7 => 3520,
            Note::AS7 => 3729,
            Note::B7 => 3951,
            Note::C8 => 4186,
            Note::CS8 => 4435,
            Note::D8 => 4699,
            Note::DS8 => 4978,
        })
    }
}

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
            peripherals.GPIO27.clone_unchecked()
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
