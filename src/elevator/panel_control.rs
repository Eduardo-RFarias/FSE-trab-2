use crate::common::{Elevator, Floor};
use crate::uart::esp32::{Button, Esp32};
use std::collections::VecDeque;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;
use stoppable_thread::StoppableHandle;

pub fn start(
    esp32: Arc<Mutex<Esp32>>,
    queues: (Arc<RwLock<VecDeque<Floor>>>, Arc<RwLock<VecDeque<Floor>>>),
    emergency: (Arc<AtomicBool>, Arc<AtomicBool>),
) -> StoppableHandle<()> {
    stoppable_thread::spawn(move |stopped| {
        while !stopped.get() {
            if !emergency.0.load(Relaxed) {
                let registers = esp32.lock().unwrap().read_all_buttons(Elevator::One);

                for (button, value) in registers {
                    if !value {
                        continue;
                    }

                    if button == Button::Emergency1 {
                        emergency.0.store(true, Relaxed);

                        queues.0.write().unwrap().clear();

                        let mut state = [false; 11];
                        state[6] = true;

                        esp32
                            .lock()
                            .unwrap()
                            .write_all_buttons(Elevator::One, &state);
                    } else {
                        if let Some(floor) = button.into_floor(Elevator::One) {
                            let mut queue = queues.0.write().unwrap();

                            if !queue.contains(&floor) {
                                queue.push_back(floor);
                            }
                        }
                    }
                }
            }

            if !emergency.1.load(Relaxed) {
                let registers = esp32.lock().unwrap().read_all_buttons(Elevator::Two);

                for (button, value) in registers {
                    if !value {
                        continue;
                    }

                    if button == Button::Emergency2 {
                        emergency.1.store(true, Relaxed);

                        queues.1.write().unwrap().clear();

                        let mut state = [false; 11];
                        state[6] = true;

                        esp32
                            .lock()
                            .unwrap()
                            .write_all_buttons(Elevator::Two, &state);
                    } else {
                        if let Some(floor) = button.into_floor(Elevator::Two) {
                            let mut queue = queues.1.write().unwrap();

                            if !queue.contains(&floor) {
                                queue.push_back(floor);
                            }
                        }
                    }
                }
            }

            thread::sleep(Duration::from_millis(500));
        }
    })
}
