use crate::common::{Direction, Elevator};
use rppal::gpio::{Gpio, OutputPin};

pub const LOWEST_POINT: i32 = 0;
pub const HIGHEST_POINT: i32 = 25500;

pub struct EngineControl {
    dir1_pin: OutputPin,
    dir2_pin: OutputPin,
    potm_pin: OutputPin,
}

impl EngineControl {
    pub fn new(elevator: Elevator) -> Self {
        let gpio = Gpio::new().unwrap();

        let instance = match elevator {
            Elevator::One => EngineControl {
                dir1_pin: gpio.get(20).unwrap().into_output_high(),
                dir2_pin: gpio.get(21).unwrap().into_output_high(),
                potm_pin: gpio.get(12).unwrap().into_output_low(),
            },
            Elevator::Two => EngineControl {
                dir1_pin: gpio.get(19).unwrap().into_output_high(),
                dir2_pin: gpio.get(26).unwrap().into_output_high(),
                potm_pin: gpio.get(13).unwrap().into_output_low(),
            },
        };

        instance
    }

    pub fn set_potency(&mut self, duty_cycle: f64) {
        self.potm_pin.set_pwm_frequency(1000.0, duty_cycle).unwrap();
    }

    pub fn set_direction(&mut self, direction: Direction) {
        match direction {
            Direction::Up => {
                self.dir1_pin.set_high();
                self.dir2_pin.set_low();
            }
            Direction::Down => {
                self.dir1_pin.set_low();
                self.dir2_pin.set_high();
            }
            Direction::Stop => {
                self.dir1_pin.set_high();
                self.dir2_pin.set_high();
            }
        }
    }
}
