use esp_hal::gpio::Input;

pub trait MotionDetector {
    fn motion_detected(&mut self) -> bool;
}

pub struct PirMotionSensor<'p>(Input<'p>);

impl<'p> PirMotionSensor<'p> {
    pub fn new(pin: Input<'p>) -> Self {
        Self(pin)
    }
}

impl<'p> MotionDetector for PirMotionSensor<'p> {
    fn motion_detected(&mut self) -> bool {
        self.0.is_high()
    }
}
