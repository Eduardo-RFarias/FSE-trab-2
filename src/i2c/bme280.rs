use crate::common::Elevator;
use bme280::i2c::BME280 as Device;
use rppal::{hal::Delay, i2c::I2c};
use std::fs::{metadata, read_to_string};

const TEMPERATURE_FILE_1: &str = "/sys/bus/i2c/devices/i2c-1/1-0076/iio:device0/in_temp_input";
const TEMPERATURE_FILE_2: &str = "/sys/bus/i2c/devices/i2c-1/1-0077/iio:device1/in_temp_input";

fn file_exists(file: &str) -> bool {
    metadata(file).is_ok()
}

struct I2cSensors {
    sensor1: Device<I2c>,
    sensor2: Device<I2c>,
    delay: Delay,
}

pub struct BME280 {
    sensors: Option<I2cSensors>,
}

impl BME280 {
    pub fn new() -> Self {
        // If TEMPERATURE_FILE_1 and TEMPERATURE_FILE_2 exist, then the BME280 is connected as kernel module
        if file_exists(TEMPERATURE_FILE_1) && file_exists(TEMPERATURE_FILE_2) {
            Self { sensors: None }
        }
        // If not, then the BME280 is connected as I2C device
        else {
            let i2c = I2c::new().unwrap();
            let mut bme280_1 = Device::new_primary(i2c);

            let i2c = I2c::new().unwrap();
            let mut bme280_2 = Device::new_secondary(i2c);

            let mut delay = Delay::new();

            bme280_1.init(&mut delay).unwrap();
            bme280_2.init(&mut delay).unwrap();

            Self {
                sensors: Some(I2cSensors {
                    sensor1: bme280_1,
                    sensor2: bme280_2,
                    delay,
                }),
            }
        }
    }

    pub fn measure_temperature(&mut self, elevator: Elevator) -> f32 {
        match &mut self.sensors {
            Some(sensors) => match elevator {
                Elevator::One => &mut sensors.sensor1,
                Elevator::Two => &mut sensors.sensor2,
            }
            .measure(&mut sensors.delay)
            .unwrap()
            .temperature
            .round(),
            None => read_to_string(match elevator {
                Elevator::One => TEMPERATURE_FILE_1,
                Elevator::Two => TEMPERATURE_FILE_2,
            })
            .unwrap()
            .trim()
            .parse()
            .map(|temp: f32| (temp + 5.0) / 1000.0)
            .unwrap()
            .round(),
        }
    }
}
