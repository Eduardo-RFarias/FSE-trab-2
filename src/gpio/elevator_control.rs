use crate::common::Elevator;
use rppal::gpio::{Gpio, InputPin, OutputPin};

struct ElevatorPins {
    dir1_pin: OutputPin,
    dir2_pin: OutputPin,
    potm_pin: OutputPin,
    ground_sensor_pin: InputPin,
    first_sensor_pin: InputPin,
    second_sensor_pin: InputPin,
    third_sensor_pin: InputPin,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Idle,
    Up,
    Down,
    Stop,
}

pub struct ElevatorControl {
    gpio: Gpio,
    elevator1: ElevatorPins,
    elevator2: ElevatorPins,
}

impl ElevatorControl {
    pub fn new() -> Self {
        let gpio = Gpio::new().unwrap();

        let elevator1 = ElevatorPins {
            dir1_pin: gpio.get(20).unwrap().into_output_low(),
            dir2_pin: gpio.get(21).unwrap().into_output_low(),
            potm_pin: gpio.get(12).unwrap().into_output_low(),
            ground_sensor_pin: gpio.get(28).unwrap().into_input_pulldown(),
            first_sensor_pin: gpio.get(23).unwrap().into_input_pulldown(),
            second_sensor_pin: gpio.get(24).unwrap().into_input_pulldown(),
            third_sensor_pin: gpio.get(25).unwrap().into_input_pulldown(),
        };

        let elevator2 = ElevatorPins {
            dir1_pin: gpio.get(19).unwrap().into_output_low(),
            dir2_pin: gpio.get(26).unwrap().into_output_low(),
            potm_pin: gpio.get(13).unwrap().into_output_low(),
            ground_sensor_pin: gpio.get(17).unwrap().into_input_pulldown(),
            first_sensor_pin: gpio.get(27).unwrap().into_input_pulldown(),
            second_sensor_pin: gpio.get(22).unwrap().into_input_pulldown(),
            third_sensor_pin: gpio.get(6).unwrap().into_input_pulldown(),
        };

        println!("Elevator control initialized");

        Self {
            gpio,
            elevator1,
            elevator2,
        }
    }

    pub fn set_potency(&mut self, elevator: Elevator, duty_cycle: f64) {
        let elevator_pins = match elevator {
            Elevator::One => &mut self.elevator1,
            Elevator::Two => &mut self.elevator2,
        };

        elevator_pins
            .potm_pin
            .set_pwm_frequency(1000.0, duty_cycle)
            .unwrap();
    }

    pub fn set_direction(&mut self, elevator: Elevator, direction: Direction) {
        let elevator_pins = match elevator {
            Elevator::One => &mut self.elevator1,
            Elevator::Two => &mut self.elevator2,
        };

        match direction {
            Direction::Idle => {
                elevator_pins.dir1_pin.set_low();
                elevator_pins.dir2_pin.set_low();
            }
            Direction::Up => {
                elevator_pins.dir1_pin.set_high();
                elevator_pins.dir2_pin.set_low();
            }
            Direction::Down => {
                elevator_pins.dir1_pin.set_low();
                elevator_pins.dir2_pin.set_high();
            }
            Direction::Stop => {
                elevator_pins.dir1_pin.set_high();
                elevator_pins.dir2_pin.set_high();
            }
        }
    }
}
