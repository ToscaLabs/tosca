#[derive(Clone)]
pub(crate) struct LightMockup {
    pub(crate) brightness: u64,
    pub(crate) save_energy: bool,
    state: bool,
}

impl Default for LightMockup {
    fn default() -> Self {
        Self::init(4, true)
    }
}

impl LightMockup {
    pub(crate) const fn init(brightness: u64, save_energy: bool) -> Self {
        Self {
            brightness,
            save_energy,
            state: false,
        }
    }

    pub(crate) fn turn_light_on(&mut self, brightness: u64, save_energy: bool) {
        self.brightness = brightness;
        self.save_energy = save_energy;
        println!("Run turn light on with brightness={brightness} and save energy={save_energy}");
    }

    pub(crate) fn turn_light_off(&mut self) {
        self.state = false;
        println!("Run turn light off");
    }

    pub(crate) fn toggle(&mut self) {
        self.state = !self.state;
        println!("Run light toggle");
    }
}
