enum KeyEvent {
    Button(Button),
    Dpad(DpadDirection),
}

enum Button {
    A,
    B,
    Start,
    Select,
}

enum DpadDirection {
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

    pub fn press(&mut self, key: KeyEvent) {
        match key {
            KeyEvent::Button(btn) => self.btn_nib &= !(1 << btn as u8),
            KeyEvent::Dpad(direction) => self.dpad_nib &= !(1 << direction as u8),
        };
    }

    pub fn release(&mut self, key: KeyEvent) {
        match key {
            KeyEvent::Button(btn) => self.btn_nib |= 1 << btn as u8,
            KeyEvent::Dpad(direction) => self.dpad_nib |= 1 << direction as u8,
        };
    }

    pub fn pick_row(&mut self, val: u8) {
        self.row = (self.row & 0xCF) | (val & 0x30);
    }

    pub fn data(&self) -> u8 {
        let mut data = self.row;
        if self.row & 0x10 == 0 {
            data |= self.dpad_nib;
        }
        if self.row & 0x20 == 0 {
            data |= self.btn_nib;
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
        btns.pick_row(!0x20);
        btns.press(KeyEvent::Button(A));
        assert_eq!(btns.data(), 0b11011110);

        btns.pick_row(!0x10);
        btns.press(KeyEvent::Dpad(Up));
        assert_eq!(btns.data(), 0b11101011);
    }
}
