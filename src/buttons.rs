pub enum GbKeyEvent {
    Button(Button),
    Dpad(DpadDirection),
}

pub enum Button {
    A,
    B,
    Start,
    Select,
}

pub enum DpadDirection {
    Right,
    Left,
    Up,
    Down,
}

pub struct Btns {
    row: u8,
    btn_nib: u8,
    dpad_nib: u8,
}

impl Btns {
    pub fn new() -> Self {
        Btns {
            row: 0xF0,
            btn_nib: 0x0F,
            dpad_nib: 0x0F,
        }
    }

    pub fn press(&mut self, key: GbKeyEvent) {
        match key {
            GbKeyEvent::Button(btn) => self.btn_nib &= !(1 << btn as u8),
            GbKeyEvent::Dpad(direction) => self.dpad_nib &= !(1 << direction as u8),
        };
    }

    pub fn release(&mut self, key: GbKeyEvent) {
        match key {
            GbKeyEvent::Button(btn) => self.btn_nib |= 1 << btn as u8,
            GbKeyEvent::Dpad(direction) => self.dpad_nib |= 1 << direction as u8,
        };
    }

    pub fn pick_row(&mut self, val: u8) {
        self.row = (self.row & 0xCF) | (val & 0x30);
    }

    pub fn data(&self) -> u8 {
        let mut data = self.row;
        if self.row & 0x10 == 0 {
            data |= self.dpad_nib;
        } else if self.row & 0x20 == 0 {
            data |= self.btn_nib;
        } else {
            data |= 0x0F;
        }
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input() {
        use Button::*;
        use DpadDirection::*;

        let mut btns = Btns::new();

        // All released
        btns.pick_row(0x30);
        assert_eq!(btns.data() & 0xF, 0xF);

        // A + read Btns
        btns.pick_row(!0x20);
        btns.press(GbKeyEvent::Button(A));
        assert_eq!(btns.data(), 0b11011110);

        // Up + read DPad
        btns.pick_row(!0x10);
        btns.press(GbKeyEvent::Dpad(Up));
        assert_eq!(btns.data(), 0b11101011);
    }
}
