use core::sync::atomic::{AtomicBool, Ordering};

pub trait MotionDetector {
    fn motion_detected(&mut self) -> bool;
}

pub struct MotionState {
    detected: AtomicBool,
}

impl MotionState {
    pub const fn new() -> Self {
        Self {
            detected: AtomicBool::new(false),
        }
    }

    pub fn update(&self, detected: bool) {
        self.detected.store(detected, Ordering::Relaxed);
    }

    pub fn is_detected(&self) -> bool {
        self.detected.load(Ordering::Relaxed)
    }
}

impl Default for MotionState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SharedMotionDetector<'a> {
    state: &'a MotionState,
}

impl<'a> SharedMotionDetector<'a> {
    pub fn new(state: &'a MotionState) -> Self {
        Self { state }
    }
}

impl MotionDetector for SharedMotionDetector<'_> {
    fn motion_detected(&mut self) -> bool {
        self.state.is_detected()
    }
}
