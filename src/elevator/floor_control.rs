use super::elevator_control::{ElevatorState, FloorsPosition};
use crate::common::{
    Direction::{Down, Stop, Up},
    Floor,
};
use crate::i2c::ssd1306::SSD1306;
use crate::uart::esp32::{Button, Esp32};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering::Relaxed},
        Arc, Mutex, MutexGuard, RwLock,
    },
    thread,
    time::Duration,
};
use stoppable_thread::StoppableHandle;

pub fn start(
    esp32: Arc<Mutex<Esp32>>,
    ssd1306: Arc<Mutex<SSD1306>>,
    elevator: Arc<Mutex<ElevatorState>>,
    floors_range: Arc<RwLock<FloorsPosition>>,
) -> StoppableHandle<()> {
    stoppable_thread::spawn(move |stopped| {
        let mut elevator = elevator.lock().unwrap();

        while !stopped.get() {
            if !elevator.emergency.load(Relaxed) {
                let floor = elevator.queue.write().unwrap().pop_front();

                if let Some(floor) = floor {
                    let emergency = elevator.emergency.clone();

                    move_to(
                        &esp32,
                        &ssd1306,
                        &mut elevator,
                        &floors_range,
                        floor,
                        emergency,
                    );

                    thread::sleep(Duration::from_secs(2));
                }
            }

            thread::sleep(Duration::from_millis(100));
        }
    })
}

fn move_to(
    esp32: &Arc<Mutex<Esp32>>,
    ssd1306: &Arc<Mutex<SSD1306>>,
    elevator: &mut MutexGuard<ElevatorState>,
    floors_range: &Arc<RwLock<FloorsPosition>>,
    floor: Floor,
    emergency: Arc<AtomicBool>,
) {
    if floor != elevator.current_floor {
        {
            let mut ssd1306 = ssd1306.lock().unwrap();

            if floor > elevator.current_floor {
                elevator.current_direction = Up;
                ssd1306.update_direction(elevator.elevator, Up);
            } else {
                elevator.current_direction = Down;
                ssd1306.update_direction(elevator.elevator, Down);
            }
        }

        let floors_range = floors_range.read().unwrap();

        let target = match floor {
            Floor::Ground => floors_range.ground,
            Floor::First => floors_range.first,
            Floor::Second => floors_range.second,
            Floor::Third => floors_range.third,
            Floor::Undefined => unreachable!(),
        };

        while match floor {
            Floor::Ground => &elevator.sensors.ground_sensor_pin,
            Floor::First => &elevator.sensors.first_sensor_pin,
            Floor::Second => &elevator.sensors.second_sensor_pin,
            Floor::Third => &elevator.sensors.third_sensor_pin,
            Floor::Undefined => unreachable!(),
        }
        .is_low()
            && !emergency.load(Relaxed)
        {
            let current_position = esp32.lock().unwrap().get_encoder_value(elevator.encoder);

            let (pid, direction) = elevator.pid.get_control_signal(current_position, target);

            elevator.engine_control.set_direction(direction);
            elevator.engine_control.set_potency(pid.max(0.05));

            if let Ok(mut esp32) = esp32.try_lock() {
                esp32.send_control_signal(elevator.encoder, (pid * 100.0) as i32);
            }

            let current_floor = if current_position < floors_range.first - 100 {
                Floor::Ground
            } else if current_position < floors_range.second - 100 {
                Floor::First
            } else if current_position < floors_range.third - 100 {
                Floor::Second
            } else {
                Floor::Third
            };

            if current_floor != elevator.current_floor {
                elevator.current_floor = current_floor;

                ssd1306
                    .lock()
                    .unwrap()
                    .update_floor(elevator.elevator, current_floor);
            }

            thread::sleep(Duration::from_millis(100));
        }

        elevator.engine_control.set_direction(Stop);
        elevator.engine_control.set_potency(0.0);

        ssd1306
            .lock()
            .unwrap()
            .update_direction(elevator.elevator, Stop);
    }

    let buttons_to_deactivate = Button::get_buttons(elevator.elevator, floor);

    for button in buttons_to_deactivate {
        esp32
            .lock()
            .unwrap()
            .write_button(elevator.elevator, button, false);
    }
}
