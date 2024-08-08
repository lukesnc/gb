pub struct Apu {
    pub master_control: u8,
}

impl Apu {
    pub fn new() -> Self {
        Apu { master_control: 0 }
    }
}
