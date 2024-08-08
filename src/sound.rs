pub struct Apu {
    pub master_control: u8,
    pub panning: u8,
    pub master_volume_vin_panning: u8,

    // Ch1 -- Pulse with period sweep
    pub ch1_sweep: u8,
    pub ch1_len_timer_duty: u8,
    pub ch1_volume_envelope: u8,
    pub ch1_period_low: u8,
    pub ch1_period_hi_control: u8,
    // Ch2 -- Pulse
    // Ch3 -- Wave output
    // Ch4 -- Noise
}

impl Apu {
    pub fn new() -> Self {
        Apu {
            master_control: 0,
            panning: 0,
            master_volume_vin_panning: 0,
            ch1_sweep: 0,
            ch1_len_timer_duty: 0,
            ch1_volume_envelope: 0,
            ch1_period_low: 0,
            ch1_period_hi_control: 0,
        }
    }
}
