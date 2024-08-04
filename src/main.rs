use common::{Elevator, Floor};
use elevator_control::ElevatorControl;

mod common;
mod elevator_control;
mod gpio;
mod i2c;
mod uart;

fn main() {
    let mut elevator = ElevatorControl::new();

    elevator.init();

    elevator.move_to(Elevator::One, Floor::Second);
    elevator.move_to(Elevator::Two, Floor::Third);
    elevator.move_to(Elevator::One, Floor::Ground);
    elevator.move_to(Elevator::Two, Floor::First);
}
