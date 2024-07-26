use crate::common::Elevator;
use crate::uart::modbus::{
    create_modbus, read_modbus, READ_ENCODER, READ_REGISTERS, SEND_PWM, SEND_TEMP, WRITE_REGISTERS,
};
use rppal::uart::{Parity, Queue, Uart};
use std::{collections::HashMap, thread::sleep, time::Duration};

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Encoder {
    One = 0x00,
    Two = 0x01,
}

const BUTTON_COUNT: u8 = 22;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Button {
    GroundFloorUp1 = 0x00,
    FirstFloorUp1 = 0x01,
    FirstFloorDown1 = 0x02,
    SecondFloorUp1 = 0x03,
    SecondFloorDown1 = 0x04,
    ThirdFloorDown1 = 0x05,
    Emergency1 = 0x06,
    GroundFloorCall1 = 0x07,
    FirstFloorCall1 = 0x08,
    SecondFloorCall1 = 0x09,
    ThirdFloorCall1 = 0x0A,

    GroundFloorUp2 = 0xA0,
    FirstFloorUp2 = 0xA1,
    FirstFloorDown2 = 0xA2,
    SecondFloorUp2 = 0xA3,
    SecondFloorDown2 = 0xA4,
    ThirdFloorDown2 = 0xA5,
    Emergency2 = 0xA6,
    GroundFloorCall2 = 0xA7,
    FirstFloorCall2 = 0xA8,
    SecondFloorCall2 = 0xA9,
    ThirdFloorCall2 = 0xAA,
}

impl From<u8> for Button {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Button::GroundFloorUp1,
            0x01 => Button::FirstFloorUp1,
            0x02 => Button::FirstFloorDown1,
            0x03 => Button::SecondFloorUp1,
            0x04 => Button::SecondFloorDown1,
            0x05 => Button::ThirdFloorDown1,
            0x06 => Button::Emergency1,
            0x07 => Button::GroundFloorCall1,
            0x08 => Button::FirstFloorCall1,
            0x09 => Button::SecondFloorCall1,
            0x0A => Button::ThirdFloorCall1,

            0xA0 => Button::GroundFloorUp2,
            0xA1 => Button::FirstFloorUp2,
            0xA2 => Button::FirstFloorDown2,
            0xA3 => Button::SecondFloorUp2,
            0xA4 => Button::SecondFloorDown2,
            0xA5 => Button::ThirdFloorDown2,
            0xA6 => Button::Emergency2,
            0xA7 => Button::GroundFloorCall2,
            0xA8 => Button::FirstFloorCall2,
            0xA9 => Button::SecondFloorCall2,
            0xAA => Button::ThirdFloorCall2,
            _ => panic!("Invalid button value: {}", value),
        }
    }
}

pub struct Esp32 {
    uart: Uart,
}

impl Esp32 {
    pub fn new() -> Self {
        let uart = Uart::new(115200, Parity::None, 8, 1).unwrap();

        println!("ESP32 initialized in UART mode");

        Esp32 { uart }
    }

    pub fn get_encoder_value(&mut self, encoder: Encoder) -> i32 {
        let mut success = false;
        let mut encoder_value = -1;

        let data = vec![encoder as u8];
        let buffer = create_modbus(READ_ENCODER, &data);

        for _ in 0..3 {
            self.uart.write(&buffer).unwrap();

            sleep(Duration::from_millis(100));

            let mut response = [0; 9];
            self.uart.read(&mut response).unwrap();

            let response = read_modbus(READ_ENCODER, &response);

            match response {
                Ok(value) => {
                    if value.len() != 4 {
                        eprintln!("Invalid encoder value length: {}", value.len());
                        self.uart.flush(Queue::Both).unwrap();
                        continue;
                    }

                    success = true;
                    encoder_value = i32::from_le_bytes([value[0], value[1], value[2], value[3]]);
                    break;
                }
                Err(msg) => {
                    eprintln!("Error reading encoder value: {}", msg);
                    self.uart.flush(Queue::Both).unwrap();
                }
            };
        }

        if !success {
            panic!("Failed to read encoder value after 3 attempts");
        }

        encoder_value
    }

    pub fn send_pwm(&mut self, encoder: Encoder, pwm: i32) {
        let mut success = false;

        let mut data = Vec::with_capacity(5);
        data.push(encoder as u8);
        data.extend(&pwm.to_le_bytes());
        let buffer = create_modbus(SEND_PWM, &data);

        for _ in 0..3 {
            self.uart.write(&buffer).unwrap();

            sleep(Duration::from_millis(100));

            let mut response = [0; 5];
            self.uart.read(&mut response).unwrap();

            let response = read_modbus(SEND_PWM, &response);

            match response {
                Ok(_) => {
                    success = true;
                    break;
                }
                Err(msg) => {
                    eprintln!("Error sending PWM: {}", msg);
                    self.uart.flush(Queue::Both).unwrap();
                }
            };
        }

        if !success {
            panic!("Failed to send PWM after 3 attempts");
        }
    }

    pub fn send_temp(&mut self, elevator: Elevator, temp: f32) {
        let mut success = false;

        let mut data = Vec::with_capacity(5);
        data.push(elevator as u8);
        data.extend(&temp.to_le_bytes());
        let buffer = create_modbus(SEND_TEMP, &data);

        for _ in 0..3 {
            self.uart.write(&buffer).unwrap();

            sleep(Duration::from_millis(100));

            let mut response = [0; 5];
            self.uart.read(&mut response).unwrap();

            let response = read_modbus(SEND_TEMP, &response);

            match response {
                Ok(_) => {
                    success = true;
                    break;
                }
                Err(msg) => {
                    eprintln!("Error sending temperature: {}", msg);
                    self.uart.flush(Queue::Both).unwrap();
                }
            };
        }

        if !success {
            panic!("Failed to send temperature after 3 attempts");
        }
    }

    pub fn read_buttons_in_range(
        &mut self,
        elevator: Elevator,
        start: Button,
        end: Button,
    ) -> HashMap<Button, bool> {
        let mut success = false;
        let mut buttons = HashMap::with_capacity(BUTTON_COUNT as usize / 2);

        let start_idx = start as u8;
        let end_idx = end as u8;

        let (min, max) = if elevator == Elevator::One {
            (0x00, 0x0A)
        } else {
            (0xA0, 0xAA)
        };

        if end_idx < start_idx {
            panic!("Invalid button range: {:X} - {:X}", start_idx, end_idx);
        }

        if start_idx < min || start_idx > max {
            panic!("Invalid start button: {:X}", start_idx);
        }

        if end_idx < min || end_idx > max {
            panic!("Invalid end button: {:X}", end_idx);
        }

        let data_len = end_idx - start_idx + 1;

        let data = vec![data_len];
        let operation = READ_REGISTERS(start_idx, data_len);
        let buffer = create_modbus(operation, &data);

        for _ in 0..3 {
            self.uart.write(&buffer).unwrap();

            sleep(Duration::from_millis(100));

            let mut response = vec![0; 4 + data_len as usize];
            self.uart.read(&mut response).unwrap();

            let response = read_modbus(operation, &response);

            match response {
                Ok(value) => {
                    if value.len() != data_len as usize {
                        eprintln!("Invalid buttons value length: {}", value.len());
                        self.uart.flush(Queue::Both).unwrap();
                        continue;
                    }

                    success = true;

                    for i in start_idx..=end_idx {
                        buttons.insert(Button::from(i), value[(i - start_idx) as usize] != 0);
                    }

                    break;
                }
                Err(msg) => {
                    eprintln!("Error reading buttons: {}", msg);
                    self.uart.flush(Queue::Both).unwrap();
                }
            };
        }

        if !success {
            panic!("Failed to read buttons after 3 attempts");
        }

        buttons
    }

    pub fn read_all_buttons(&mut self, elevator: Elevator) -> HashMap<Button, bool> {
        match elevator {
            Elevator::One => self.read_buttons_in_range(
                elevator,
                Button::GroundFloorUp1,
                Button::ThirdFloorCall1,
            ),
            Elevator::Two => self.read_buttons_in_range(
                elevator,
                Button::GroundFloorUp2,
                Button::ThirdFloorCall2,
            ),
        }
    }

    pub fn read_button(&mut self, elevator: Elevator, button: Button) -> bool {
        *self
            .read_buttons_in_range(elevator, button, button)
            .get(&button)
            .unwrap()
    }

    pub fn write_button_in_range(
        &mut self,
        elevator: Elevator,
        start: Button,
        end: Button,
        state: &[bool],
    ) {
        let mut success = false;

        let (min, max) = if elevator == Elevator::One {
            (0x00, 0x0A)
        } else {
            (0xA0, 0xAA)
        };

        let start_idx = start as u8;
        let end_idx = end as u8;

        if end_idx < start_idx {
            panic!("Invalid button range: {:X} - {:X}", start_idx, end_idx);
        }

        if start_idx < min || start_idx > max {
            panic!("Invalid start button: {:X}", start_idx);
        }

        if end_idx < min || end_idx > max {
            panic!("Invalid end button: {:X}", end_idx);
        }

        let data_len = end_idx - start_idx + 1;

        if state.len() != data_len as usize {
            panic!("Invalid state length: {}", state.len());
        }

        let mut data = Vec::with_capacity(1 + data_len as usize);

        data.push(data_len);

        for s in state {
            data.push(*s as u8);
        }

        let operation = WRITE_REGISTERS(start_idx, data_len);
        let buffer = create_modbus(operation, &data);

        for _ in 0..3 {
            self.uart.write(&buffer).unwrap();

            sleep(Duration::from_millis(100));

            let mut response = vec![0; 4 + data_len as usize];
            self.uart.read(&mut response).unwrap();

            let response = read_modbus(operation, &response);

            match response {
                Ok(value) => {
                    if value.len() != data_len as usize {
                        eprintln!("Invalid buttons value length: {}", value.len());
                        self.uart.flush(Queue::Both).unwrap();
                        continue;
                    }

                    success = true;
                    break;
                }
                Err(msg) => {
                    eprintln!("Error reading buttons: {}", msg);
                    self.uart.flush(Queue::Both).unwrap();
                }
            };
        }

        if !success {
            panic!("Failed to write buttons after 3 attempts");
        }
    }

    pub fn write_button(&mut self, elevator: Elevator, button: Button, state: bool) {
        self.write_button_in_range(elevator, button, button, &[state]);
    }

    pub fn write_all_buttons(&mut self, elevator: Elevator, state: &[bool]) {
        match elevator {
            Elevator::One => self.write_button_in_range(
                elevator,
                Button::GroundFloorUp1,
                Button::ThirdFloorCall1,
                state,
            ),
            Elevator::Two => self.write_button_in_range(
                elevator,
                Button::GroundFloorUp2,
                Button::ThirdFloorCall2,
                state,
            ),
        }
    }
}
