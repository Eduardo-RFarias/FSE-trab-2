use crate::common::Direction::{Down, Stop, Up};
use crate::common::Floor;
use crate::elevator::elevator_control::{ElevatorState, FloorsPosition};
use crate::uart::esp32::Esp32;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex, MutexGuard, RwLock};
use std::thread;
use std::time::Duration;

const CALIBRATION_FILE: &str = "calibration.bin";

pub fn read_calibration() -> Result<FloorsPosition, String> {
    let mut file = File::open(CALIBRATION_FILE).map_err(|_| "Calibration file not found")?;

    let mut floors_position = FloorsPosition::default();

    for i in 0..4 {
        let mut buffer = [0; 4];
        file.read_exact(&mut buffer)
            .map_err(|e| format!("Invalid calibration file: {:?}", e))?;

        let number = i32::from_le_bytes(buffer);

        match i {
            0 => floors_position.ground = number,
            1 => floors_position.first = number,
            2 => floors_position.second = number,
            3 => floors_position.third = number,
            _ => unreachable!(),
        }
    }

    Ok(floors_position)
}

pub fn write_calibration(floors_position: &FloorsPosition) {
    let mut file = File::create(CALIBRATION_FILE).unwrap();

    for i in 0..4 {
        let number = match i {
            0 => floors_position.ground,
            1 => floors_position.first,
            2 => floors_position.second,
            3 => floors_position.third,
            _ => unreachable!(),
        };

        file.write_all(&number.to_le_bytes()).unwrap();
    }
}

pub fn start(
    esp32: Arc<Mutex<Esp32>>,
    elevator: Arc<Mutex<ElevatorState>>,
    floors_range: Arc<RwLock<FloorsPosition>>,
) {
    println!("Starting elevator calibration, do not close the program.");

    let mut esp32 = esp32.lock().unwrap();
    let mut elevator = elevator.lock().unwrap();
    let mut floors_range = floors_range.write().unwrap();

    // First, we need to move the elevator to the lowest point
    elevator.engine_control.set_direction(Down);
    elevator.engine_control.set_potency(1.0);

    loop {
        let current_position = esp32.get_encoder_value(elevator.encoder);

        if current_position <= 0 {
            break;
        }

        thread::sleep(Duration::from_millis(100));
    }

    elevator.engine_control.set_direction(Stop);
    elevator.engine_control.set_potency(0.0);

    wait_for_floor_calibration(
        &mut esp32,
        &mut elevator,
        Floor::Ground,
        &mut floors_range.ground,
    );

    wait_for_floor_calibration(
        &mut esp32,
        &mut elevator,
        Floor::First,
        &mut floors_range.first,
    );

    wait_for_floor_calibration(
        &mut esp32,
        &mut elevator,
        Floor::Second,
        &mut floors_range.second,
    );

    wait_for_floor_calibration(
        &mut esp32,
        &mut elevator,
        Floor::Third,
        &mut floors_range.third,
    );

    println!("Elevator calibration finished.");
}

fn wait_for_floor_calibration(
    esp32: &mut MutexGuard<Esp32>,
    elevator: &mut MutexGuard<ElevatorState>,
    floor: Floor,
    floor_range: &mut i32,
) {
    // Then, we can start rising the elevator until the sensor is triggered
    elevator.engine_control.set_direction(Up);
    elevator.engine_control.set_potency(0.10);

    while match floor {
        Floor::Ground => &elevator.sensors.ground_sensor_pin,
        Floor::First => &elevator.sensors.first_sensor_pin,
        Floor::Second => &elevator.sensors.second_sensor_pin,
        Floor::Third => &elevator.sensors.third_sensor_pin,
        Floor::Undefined => unreachable!(),
    }
    .is_low()
    {}

    elevator.engine_control.set_direction(Stop);
    elevator.engine_control.set_potency(0.0);

    // At the sensor rising, we can set the floor position
    *floor_range = esp32.get_encoder_value(elevator.encoder);

    // Then, we can start rising the elevator until the falling edge of the sensor
    elevator.engine_control.set_direction(Up);
    elevator.engine_control.set_potency(0.10);

    while match floor {
        Floor::Ground => &elevator.sensors.ground_sensor_pin,
        Floor::First => &elevator.sensors.first_sensor_pin,
        Floor::Second => &elevator.sensors.second_sensor_pin,
        Floor::Third => &elevator.sensors.third_sensor_pin,
        Floor::Undefined => unreachable!(),
    }
    .is_high()
    {}

    elevator.engine_control.set_direction(Stop);
    elevator.engine_control.set_potency(0.0);

    // At the sensor falling edge, we can set the floor position with a mean value
    *floor_range = (*floor_range + esp32.get_encoder_value(elevator.encoder)) / 2;
}
