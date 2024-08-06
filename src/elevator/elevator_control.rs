use crate::common::{
    Direction::{self, Stop},
    Elevator, Floor,
};
use crate::elevator::{calibration_control, floor_control, panel_control, temperature_control};
use crate::gpio::{engine_control::EngineControl, pid::PidController};
use crate::i2c::ssd1306::SSD1306;
use crate::uart::esp32::{Encoder, Esp32};
use rppal::gpio::{Gpio, InputPin};
use std::{
    collections::VecDeque,
    sync::{atomic::AtomicBool, Arc, Mutex, RwLock},
};
use stoppable_thread::StoppableHandle;

#[derive(Default)]
pub struct FloorsPosition {
    pub ground: i32,
    pub first: i32,
    pub second: i32,
    pub third: i32,
}

pub struct SensorPins {
    pub ground_sensor_pin: InputPin,
    pub first_sensor_pin: InputPin,
    pub second_sensor_pin: InputPin,
    pub third_sensor_pin: InputPin,
}

pub struct ElevatorState {
    pub elevator: Elevator,
    pub encoder: Encoder,

    pub engine_control: EngineControl,
    pub pid: PidController,
    pub sensors: SensorPins,

    pub queue: Arc<RwLock<VecDeque<Floor>>>,
    pub emergency: Arc<AtomicBool>,

    pub current_floor: Floor,
    pub current_direction: Direction,
}

pub struct ElevatorControl {
    esp32: Arc<Mutex<Esp32>>,
    ssd1306: Arc<Mutex<SSD1306>>,

    elevator_1: Arc<Mutex<ElevatorState>>,
    elevator_2: Arc<Mutex<ElevatorState>>,

    floors_range: Arc<RwLock<FloorsPosition>>,

    temperature_thread: Option<StoppableHandle<()>>,
    panel_thread: Option<StoppableHandle<()>>,
    elevator_1_thread: Option<StoppableHandle<()>>,
    elevator_2_thread: Option<StoppableHandle<()>>,

    ready: bool,
}

impl Drop for ElevatorControl {
    fn drop(&mut self) {
        if self.temperature_thread.is_some() {
            println!("WARNING: stop() was not called before dropping ElevatorControl. This may hinder the cleanup process.");
            println!("To avoid this warning, call stop() before dropping ElevatorControl.");
            self.stop();
        }
    }
}

impl ElevatorControl {
    pub fn new() -> Self {
        // Init
        let gpio = Gpio::new().unwrap();
        let esp32 = Esp32::new();
        let mut ssd1306 = SSD1306::new();

        ssd1306.update_floor(Elevator::One, Floor::Undefined);
        ssd1306.update_floor(Elevator::Two, Floor::Undefined);
        ssd1306.update_direction(Elevator::One, Stop);
        ssd1306.update_direction(Elevator::Two, Stop);

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
            current_floor: Floor::Undefined,
            current_direction: Stop,
            queue: Arc::new(RwLock::new(VecDeque::new())),
            emergency: Arc::new(AtomicBool::new(false)),
        };

        elevator_1.engine_control.set_direction(Stop);

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
            current_floor: Floor::Undefined,
            current_direction: Stop,
            queue: Arc::new(RwLock::new(VecDeque::new())),
            emergency: Arc::new(AtomicBool::new(false)),
        };

        elevator_2.engine_control.set_direction(Stop);

        let floors_range = FloorsPosition::default();

        let esp32 = Arc::new(Mutex::new(esp32));
        let ssd1306 = Arc::new(Mutex::new(ssd1306));

        // Return
        Self {
            esp32,
            ssd1306,
            elevator_1: Arc::new(Mutex::new(elevator_1)),
            elevator_2: Arc::new(Mutex::new(elevator_2)),
            temperature_thread: None,
            panel_thread: None,
            elevator_1_thread: None,
            elevator_2_thread: None,
            floors_range: Arc::new(RwLock::new(floors_range)),
            ready: false,
        }
    }

    pub fn init(&mut self) {
        // Calibration
        match calibration_control::read_calibration() {
            Ok(floors_range) => {
                *self.floors_range.write().unwrap() = floors_range;
            }
            Err(_) => {
                println!("Calibration file not found.");

                calibration_control::start(
                    self.esp32.clone(),
                    self.elevator_1.clone(),
                    self.floors_range.clone(),
                );

                calibration_control::write_calibration(&self.floors_range.read().unwrap());
            }
        }

        // Temperature thread
        self.temperature_thread = Some(temperature_control::start(
            self.esp32.clone(),
            self.ssd1306.clone(),
        ));

        // Get current floor for elevator 1
        {
            let current_position = self.esp32.lock().unwrap().get_encoder_value(Encoder::One);
            let floors_range = self.floors_range.read().unwrap();
            let mut elevator = self.elevator_1.lock().unwrap();

            elevator.current_floor = if current_position < floors_range.first - 100 {
                Floor::Ground
            } else if current_position < floors_range.second - 100 {
                Floor::First
            } else if current_position < floors_range.third - 100 {
                Floor::Second
            } else {
                Floor::Third
            };

            self.ssd1306
                .lock()
                .unwrap()
                .update_floor(Elevator::One, elevator.current_floor);
        }

        // Get current floor for elevator 2
        {
            let current_position = self.esp32.lock().unwrap().get_encoder_value(Encoder::Two);
            let floors_range = self.floors_range.read().unwrap();
            let mut elevator = self.elevator_2.lock().unwrap();

            elevator.current_floor = if current_position < floors_range.first - 100 {
                Floor::Ground
            } else if current_position < floors_range.second - 100 {
                Floor::First
            } else if current_position < floors_range.third - 100 {
                Floor::Second
            } else {
                Floor::Third
            };

            self.ssd1306
                .lock()
                .unwrap()
                .update_floor(Elevator::Two, elevator.current_floor);
        }

        // Panel thread
        {
            let elevator_1 = self.elevator_1.lock().unwrap();
            let elevator_2 = self.elevator_2.lock().unwrap();

            self.panel_thread = Some(panel_control::start(
                self.esp32.clone(),
                (elevator_1.queue.clone(), elevator_2.queue.clone()),
                (elevator_1.emergency.clone(), elevator_2.emergency.clone()),
            ));
        }

        // Floors thread
        self.elevator_1_thread = Some(floor_control::start(
            self.esp32.clone(),
            self.ssd1306.clone(),
            self.elevator_1.clone(),
            self.floors_range.clone(),
        ));

        self.elevator_2_thread = Some(floor_control::start(
            self.esp32.clone(),
            self.ssd1306.clone(),
            self.elevator_2.clone(),
            self.floors_range.clone(),
        ));

        self.ready = true;
    }

    pub fn stop(&mut self) {
        if let Some(handle) = self.temperature_thread.take() {
            handle.stop().join().unwrap();
        }

        if let Some(handle) = self.panel_thread.take() {
            handle.stop().join().unwrap();
        }

        if let Some(handle) = self.elevator_1_thread.take() {
            handle.stop().join().unwrap();
        }

        if let Some(handle) = self.elevator_2_thread.take() {
            handle.stop().join().unwrap();
        }

        {
            let mut elevator_1 = self.elevator_1.lock().unwrap();
            elevator_1.engine_control.set_direction(Stop);
            elevator_1.engine_control.set_potency(0.0);

            let mut elevator_2 = self.elevator_2.lock().unwrap();
            elevator_2.engine_control.set_direction(Stop);
            elevator_2.engine_control.set_potency(0.0);
        }

        let mut ssd1306 = self.ssd1306.lock().unwrap();
        let mut esp32 = self.esp32.lock().unwrap();

        ssd1306.update_direction(Elevator::One, Stop);
        ssd1306.update_floor(Elevator::One, Floor::Ground);
        ssd1306.update_temperature(Elevator::One, 0.0);
        esp32.write_all_buttons(Elevator::One, &[false; 11]);

        ssd1306.update_direction(Elevator::Two, Stop);
        ssd1306.update_floor(Elevator::Two, Floor::Ground);
        ssd1306.update_temperature(Elevator::Two, 0.0);
        esp32.write_all_buttons(Elevator::Two, &[false; 11]);

        self.ready = false;
    }
}
