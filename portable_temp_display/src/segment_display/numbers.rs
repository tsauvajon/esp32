impl<'p> super::SegmentDisplay<'p> {
    pub(super) fn display_digit(&mut self, number: u8) {
        match number {
            1 => self.one(),
            2 => self.two(),
            3 => self.three(),
            4 => self.four(),
            5 => self.five(),
            6 => self.six(),
            7 => self.seven(),
            8 => self.eight(),
            9 => self.nine(),
            _ => self.zero(),
        }
    }

    fn zero(&mut self) {
        self.seg_a.set_low();
        self.seg_b.set_low();
        self.seg_c.set_low();
        self.seg_d.set_low();
        self.seg_e.set_low();
        self.seg_f.set_low();
        self.seg_g.set_high();
    }

    fn one(&mut self) {
        self.seg_a.set_high();
        self.seg_b.set_low();
        self.seg_c.set_low();
        self.seg_d.set_high();
        self.seg_e.set_high();
        self.seg_f.set_high();
        self.seg_g.set_high();
    }

    fn two(&mut self) {
        self.seg_a.set_low();
        self.seg_b.set_low();
        self.seg_c.set_high();
        self.seg_d.set_low();
        self.seg_e.set_low();
        self.seg_f.set_high();
        self.seg_g.set_low();
    }

    fn three(&mut self) {
        self.seg_a.set_low();
        self.seg_b.set_low();
        self.seg_c.set_low();
        self.seg_d.set_low();
        self.seg_e.set_high();
        self.seg_f.set_high();
        self.seg_g.set_low();
    }

    fn four(&mut self) {
        self.seg_a.set_high();
        self.seg_b.set_low();
        self.seg_c.set_low();
        self.seg_d.set_high();
        self.seg_e.set_high();
        self.seg_f.set_low();
        self.seg_g.set_low();
    }

    fn five(&mut self) {
        self.seg_a.set_low();
        self.seg_b.set_high();
        self.seg_c.set_low();
        self.seg_d.set_low();
        self.seg_e.set_high();
        self.seg_f.set_low();
        self.seg_g.set_low();
    }

    fn six(&mut self) {
        self.seg_a.set_low();
        self.seg_b.set_high();
        self.seg_c.set_low();
        self.seg_d.set_low();
        self.seg_e.set_low();
        self.seg_f.set_low();
        self.seg_g.set_low();
    }

    fn seven(&mut self) {
        self.seg_a.set_low();
        self.seg_b.set_low();
        self.seg_c.set_low();
        self.seg_d.set_high();
        self.seg_e.set_high();
        self.seg_f.set_high();
        self.seg_g.set_high();
    }

    fn eight(&mut self) {
        self.seg_a.set_low();
        self.seg_b.set_low();
        self.seg_c.set_low();
        self.seg_d.set_low();
        self.seg_e.set_low();
        self.seg_f.set_low();
        self.seg_g.set_low();
    }

    fn nine(&mut self) {
        self.seg_a.set_low();
        self.seg_b.set_low();
        self.seg_c.set_low();
        self.seg_d.set_low();
        self.seg_e.set_high();
        self.seg_f.set_low();
        self.seg_g.set_low();
    }
}
