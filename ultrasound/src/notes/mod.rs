use esp_hal::{
    delay::Delay,
    gpio::{DriveMode, Level},
    ledc::{
        HighSpeed, Ledc,
        channel::{self, ChannelIFace},
        timer::{self, TimerIFace},
    },
    peripherals::Peripherals,
    rmt::{ContinuousTxTransaction, LoopMode, PulseCode, Rmt, TxChannelConfig, TxChannelCreator},
    time::Rate,
};

use crate::MANDATORY_RMT_MHZ_FREQUENCY_FOR_ESP32;

pub mod pink_panther;

#[derive(Copy, Clone, Eq, PartialEq)]
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

    Silence,
}

impl Note {
    pub fn play<'ch>(&self, tx: ContinuousTxTransaction<'ch>) -> ContinuousTxTransaction<'ch> {
        tx.stop()
            .unwrap()
            .transmit_continuously(&[self], LoopMode::Infinite)
            .unwrap()
    }
}

impl From<Note> for Rate {
    fn from(note: Note) -> Self {
        Rate::from_hz(match note {
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

            Note::Silence => 1,
        })
    }
}

impl From<Note> for PulseCode {
    fn from(note: Note) -> Self {
        if note == Note::Silence {
            return PulseCode::new(Level::Low, 1, Level::Low, 1);
        }

        let phase_ticks = Rate::from(note).as_hz() as u16;
        PulseCode::new(Level::High, phase_ticks, Level::Low, phase_ticks)
    }
}

impl From<&Note> for Rate {
    fn from(note: &Note) -> Self {
        Self::from(*note)
    }
}

impl From<&Note> for PulseCode {
    fn from(note: &Note) -> Self {
        Self::from(*note)
    }
}

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

pub fn play_song_with_rmt(peripherals: Peripherals, tempo: u16, melody: &[(Note, i16)]) {
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
    let mut buzzer_tx = buzzer_channel
        .transmit_continuously(&[Note::Silence], LoopMode::Infinite)
        .unwrap();

    let delay = Delay::new();
    let song = Song::new(tempo);
    for (note, duration_type) in melody {
        let note_duration = song.calc_note_duration(*duration_type);
        if *note == Note::Silence {
            buzzer_tx = Note::Silence.play(buzzer_tx);
            delay.delay_millis(note_duration);
            continue;
        }

        buzzer_tx = note.play(buzzer_tx);
        let pause_duration = note_duration / 10; // 10% of note_duration
        delay.delay_millis(note_duration - pause_duration); // play 90%
        buzzer_tx = Note::Silence.play(buzzer_tx);
        delay.delay_millis(pause_duration); // Pause for 10%
    }
}

pub fn _play_song_with_ledc(peripherals: Peripherals, tempo: u16, melody: &[(Note, i16)]) {
    let ledc = Ledc::new(peripherals.LEDC);
    let mut hstimer0 = ledc.timer::<HighSpeed>(timer::Number::Timer0);

    let delay = Delay::new();
    let song = Song::new(tempo);
    for (note, duration_type) in melody {
        let note_duration = song.calc_note_duration(*duration_type);
        if *note == Note::Silence {
            delay.delay_millis(note_duration);
            continue;
        }

        let frequency = Rate::from(*note);
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
