use crate::common::Floor;
use crate::gpio::engine_control::{HIGHEST_POINT, LOWEST_POINT};
use crate::gpio::{engine_control::EngineControl, pid::PidController};
use crate::i2c::bme280::BME280;
use crate::i2c::ssd1306::SSD1306;
use crate::{
    common::{Direction, Elevator},
    uart::esp32::{Encoder, Esp32},
};
use rppal::gpio::{Gpio, InputPin, Level, Trigger};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};

struct FloorsPosition {
    ground: Arc<AtomicI32>,
    first: Arc<AtomicI32>,
    second: Arc<AtomicI32>,
    third: Arc<AtomicI32>,
}

struct SensorPins {
    ground_sensor_pin: InputPin,
    first_sensor_pin: InputPin,
    second_sensor_pin: InputPin,
    third_sensor_pin: InputPin,
}

struct ElevatorState {
    elevator: Elevator,
    encoder: Encoder,
    engine_control: EngineControl,
    pid: PidController,
    sensors: SensorPins,
    current_position: Arc<AtomicI32>,
    current_floor: Floor,
    current_direction: Direction,
}

pub struct ElevatorControl {
    esp32: Arc<Mutex<Esp32>>,
    ssd1306: Arc<Mutex<SSD1306>>,
    bme280: Arc<Mutex<BME280>>,
    elevator_1: ElevatorState,
    elevator_2: ElevatorState,
    floors_range: FloorsPosition,
    calibrated: bool,
    running_temperature_thread: Arc<AtomicBool>,
}

impl ElevatorControl {
    pub fn new() -> Self {
        let gpio = Gpio::new().unwrap();
        let esp32 = Arc::new(Mutex::new(Esp32::new()));

        let mut elevator_1 = ElevatorState {
            elevator: Elevator::One,
            encoder: Encoder::One,
            engine_control: EngineControl::new(Elevator::One),
            pid: PidController::new(),
            sensors: SensorPins {
                ground_sensor_pin: gpio.get(18).unwrap().into_input_pulldown(),
                first_sensor_pin: gpio.get(23).unwrap().into_input_pulldown(),
                second_sensor_pin: gpio.get(24).unwrap().into_input_pulldown(),
                third_sensor_pin: gpio.get(25).unwrap().into_input_pulldown(),
            },
            current_position: Arc::new(AtomicI32::new(0)),
            current_floor: Floor::Ground,
            current_direction: Direction::Stop,
        };

        let mut elevator_2 = ElevatorState {
            elevator: Elevator::Two,
            encoder: Encoder::Two,
            engine_control: EngineControl::new(Elevator::Two),
            pid: PidController::new(),
            sensors: SensorPins {
                ground_sensor_pin: gpio.get(17).unwrap().into_input_pulldown(),
                first_sensor_pin: gpio.get(27).unwrap().into_input_pulldown(),
                second_sensor_pin: gpio.get(22).unwrap().into_input_pulldown(),
                third_sensor_pin: gpio.get(6).unwrap().into_input_pulldown(),
            },
            current_position: Arc::new(AtomicI32::new(0)),
            current_floor: Floor::Ground,
            current_direction: Direction::Stop,
        };

        elevator_1.engine_control.set_direction(Direction::Stop);
        elevator_2.engine_control.set_direction(Direction::Stop);

        let ssd1306 = Arc::new(Mutex::new(SSD1306::new()));
        let bme280 = Arc::new(Mutex::new(BME280::new()));

        Self {
            esp32,
            ssd1306,
            bme280,
            elevator_1,
            elevator_2,
            floors_range: FloorsPosition {
                ground: Arc::new(AtomicI32::new(0)),
                first: Arc::new(AtomicI32::new(0)),
                second: Arc::new(AtomicI32::new(0)),
                third: Arc::new(AtomicI32::new(0)),
            },
            calibrated: false,
            running_temperature_thread: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn init(&mut self) {
        if !self.calibrated {
            self.calibrate();
        }

        self.start_temperature_control();
    }

    pub fn move_to(&mut self, elevator: Elevator, floor: Floor) {
        let elevator = match elevator {
            Elevator::One => &mut self.elevator_1,
            Elevator::Two => &mut self.elevator_2,
        };

        let target = match floor {
            Floor::Ground => &self.floors_range.ground,
            Floor::First => &self.floors_range.first,
            Floor::Second => &self.floors_range.second,
            Floor::Third => &self.floors_range.third,
        }
        .load(Ordering::Relaxed);

        let sensor = match floor {
            Floor::Ground => &elevator.sensors.ground_sensor_pin,
            Floor::First => &elevator.sensors.first_sensor_pin,
            Floor::Second => &elevator.sensors.second_sensor_pin,
            Floor::Third => &elevator.sensors.third_sensor_pin,
        };

        while sensor.is_low() {
            let current_position = self
                .esp32
                .lock()
                .unwrap()
                .get_encoder_value(elevator.encoder);

            let (pid, direction) = elevator.pid.get_control_signal(current_position, target);

            elevator.engine_control.set_direction(direction);
            elevator.engine_control.set_potency(pid);

            // Send the control signal to the ESP32 asynchronously
            let esp32 = self.esp32.clone();
            let encoder = elevator.encoder;
            thread::spawn(move || {
                if let Ok(mut esp32) = esp32.try_lock() {
                    esp32.send_control_signal(encoder, (pid * 100.0) as i32);
                }
            });

            thread::sleep(Duration::from_millis(100));
        }

        elevator.engine_control.set_direction(Direction::Stop);
        elevator.engine_control.set_potency(0.0);
    }

    pub fn set_direction(&mut self, elevator: Elevator, direction: Direction) {
        let elevator = match elevator {
            Elevator::One => &mut self.elevator_1,
            Elevator::Two => &mut self.elevator_2,
        };

        elevator.current_direction = direction;

        self.ssd1306
            .lock()
            .unwrap()
            .update_direction(elevator.elevator, direction);
    }

    pub fn set_floor(&mut self, elevator: Elevator, floor: Floor) {
        let elevator = match elevator {
            Elevator::One => &mut self.elevator_1,
            Elevator::Two => &mut self.elevator_2,
        };

        elevator.current_floor = floor;

        self.ssd1306
            .lock()
            .unwrap()
            .update_floor(elevator.elevator, floor);
    }

    pub fn start_temperature_control(&self) {
        self.running_temperature_thread
            .store(true, Ordering::Relaxed);

        let esp32 = self.esp32.clone();
        let bme280 = self.bme280.clone();
        let ssd1306 = self.ssd1306.clone();
        let running = self.running_temperature_thread.clone();
        thread::spawn(move || {
            while running.load(Ordering::Relaxed) {
                let temperature_1;
                let temperature_2;

                {
                    let mut bme280 = bme280.lock().unwrap();

                    temperature_1 = bme280.measure_temperature(Elevator::One);
                    temperature_2 = bme280.measure_temperature(Elevator::Two);
                }

                {
                    let mut esp32 = esp32.lock().unwrap();

                    esp32.send_temp(Elevator::One, temperature_1);
                    esp32.send_temp(Elevator::Two, temperature_2);
                }

                {
                    let mut ssd1306 = ssd1306.lock().unwrap();

                    ssd1306.update_temperature(Elevator::One, temperature_1);
                    ssd1306.update_temperature(Elevator::Two, temperature_2);
                }

                thread::sleep(Duration::from_secs(5));
            }
        });
    }

    pub fn stop_temperature_control(&self) {
        self.running_temperature_thread
            .store(false, Ordering::Relaxed);
    }

    fn calibrate(&mut self) {
        println!("Starting elevator calibration");

        // First, we need to move the elevator to the lowest point
        self.send_to_bottom();

        // Then, set the interrupts for the sensors
        self.set_interrupts();

        // Move the elevator to the top
        self.send_to_top();

        // Clear the interrupts
        self.clear_interrupts();

        self.calibrated = true;

        println!("Elevator calibration finished.");
    }

    fn send_to_bottom(&mut self) {
        self.elevator_1
            .engine_control
            .set_direction(Direction::Down);

        self.elevator_1.engine_control.set_potency(1.0);

        loop {
            let current_position = self
                .esp32
                .lock()
                .unwrap()
                .get_encoder_value(self.elevator_1.encoder);

            if current_position <= LOWEST_POINT {
                break;
            }

            thread::sleep(Duration::from_millis(100));
        }

        self.elevator_1
            .engine_control
            .set_direction(Direction::Stop);

        self.elevator_1.engine_control.set_potency(0.0);
    }

    fn send_to_top(&mut self) {
        self.elevator_1.engine_control.set_direction(Direction::Up);

        self.elevator_1.engine_control.set_potency(0.10);

        loop {
            let current_position = self
                .esp32
                .lock()
                .unwrap()
                .get_encoder_value(self.elevator_1.encoder);

            self.elevator_1
                .current_position
                .store(current_position, Ordering::Relaxed);

            if current_position >= HIGHEST_POINT {
                break;
            }

            thread::sleep(Duration::from_millis(100));
        }

        self.elevator_1
            .engine_control
            .set_direction(Direction::Stop);

        self.elevator_1.engine_control.set_potency(0.0);
    }

    fn set_interrupts(&mut self) {
        let current_position = self.elevator_1.current_position.clone();
        let floor_range = self.floors_range.ground.clone();
        self.elevator_1
            .sensors
            .ground_sensor_pin
            .set_async_interrupt(Trigger::Both, move |level| match level {
                Level::High => {
                    let current_position = current_position.load(Ordering::Relaxed);
                    floor_range.store(current_position, Ordering::Relaxed);
                }
                Level::Low => {
                    let current_position = current_position.load(Ordering::Relaxed);
                    let old_position = floor_range.load(Ordering::Relaxed);
                    floor_range.store((old_position + current_position) / 2, Ordering::Relaxed);
                }
            })
            .unwrap();

        let current_position = self.elevator_1.current_position.clone();
        let floor_range = self.floors_range.first.clone();
        self.elevator_1
            .sensors
            .first_sensor_pin
            .set_async_interrupt(Trigger::Both, move |level| match level {
                Level::High => {
                    let current_position = current_position.load(Ordering::Relaxed);
                    floor_range.store(current_position, Ordering::Relaxed);
                }
                Level::Low => {
                    let current_position = current_position.load(Ordering::Relaxed);
                    let old_position = floor_range.load(Ordering::Relaxed);
                    floor_range.store((old_position + current_position) / 2, Ordering::Relaxed);
                }
            })
            .unwrap();

        let current_position = self.elevator_1.current_position.clone();
        let floor_range = self.floors_range.second.clone();
        self.elevator_1
            .sensors
            .second_sensor_pin
            .set_async_interrupt(Trigger::Both, move |level| match level {
                Level::High => {
                    let current_position = current_position.load(Ordering::Relaxed);
                    floor_range.store(current_position, Ordering::Relaxed);
                }
                Level::Low => {
                    let current_position = current_position.load(Ordering::Relaxed);
                    let old_position = floor_range.load(Ordering::Relaxed);
                    floor_range.store((old_position + current_position) / 2, Ordering::Relaxed);
                }
            })
            .unwrap();

        let current_position = self.elevator_1.current_position.clone();
        let floor_range = self.floors_range.third.clone();
        self.elevator_1
            .sensors
            .third_sensor_pin
            .set_async_interrupt(Trigger::Both, move |level| match level {
                Level::High => {
                    let current_position = current_position.load(Ordering::Relaxed);
                    floor_range.store(current_position, Ordering::Relaxed);
                }
                Level::Low => {
                    let current_position = current_position.load(Ordering::Relaxed);
                    let old_position = floor_range.load(Ordering::Relaxed);
                    floor_range.store((old_position + current_position) / 2, Ordering::Relaxed);
                }
            })
            .unwrap();
    }

    fn clear_interrupts(&mut self) {
        self.elevator_1
            .sensors
            .ground_sensor_pin
            .clear_async_interrupt()
            .unwrap();
        self.elevator_1
            .sensors
            .first_sensor_pin
            .clear_async_interrupt()
            .unwrap();
        self.elevator_1
            .sensors
            .second_sensor_pin
            .clear_async_interrupt()
            .unwrap();
        self.elevator_1
            .sensors
            .third_sensor_pin
            .clear_async_interrupt()
            .unwrap();
    }
}
