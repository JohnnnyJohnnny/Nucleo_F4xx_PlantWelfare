use stm32f4xx_hal::gpio::{gpioa::PA5, Output, PushPull};

//use embedded_hal::digital::v2::{OutputPin, ToggleableOutputPin};

/// Onboard led
pub struct Led {
    pa5: PA5<Output<PushPull>>,
}

impl Led {
    pub fn new(pin: PA5<Output<PushPull>>) -> Self {
        //let pa5 = pin.into_push_pull_output();
        let pa5 = pin;
        Self { pa5 }
    }

    pub fn set(&mut self, enable: bool) {
        if enable {
            self.pa5.set_high();
        } else {
            self.pa5.set_low();
        }
    }

    pub fn toggle(&mut self) {
        self.pa5.toggle();
    }
}
