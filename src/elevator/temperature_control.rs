use crate::common::Elevator;
use crate::i2c::{bme280::BME280, ssd1306::SSD1306};
use crate::uart::esp32::Esp32;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use stoppable_thread::StoppableHandle;

pub fn start(esp32: Arc<Mutex<Esp32>>, ssd1306: Arc<Mutex<SSD1306>>) -> StoppableHandle<()> {
    stoppable_thread::spawn(move |stopped| {
        let mut bme280 = BME280::new();

        let mut temperature_1 = 0.0;
        let mut temperature_2 = 0.0;

        while !stopped.get() {
            let current_temperature_1 = bme280.measure_temperature(Elevator::One);
            let current_temperature_2 = bme280.measure_temperature(Elevator::Two);

            if current_temperature_1 != temperature_1 {
                temperature_1 = current_temperature_1;

                esp32
                    .lock()
                    .unwrap()
                    .send_temp(Elevator::One, temperature_1);

                ssd1306
                    .lock()
                    .unwrap()
                    .update_temperature(Elevator::One, temperature_1);
            }

            if current_temperature_2 != temperature_2 {
                temperature_2 = current_temperature_2;

                esp32
                    .lock()
                    .unwrap()
                    .send_temp(Elevator::Two, temperature_2);

                ssd1306
                    .lock()
                    .unwrap()
                    .update_temperature(Elevator::Two, temperature_2);
            }

            thread::sleep(Duration::from_secs(1));
        }
    })
}
