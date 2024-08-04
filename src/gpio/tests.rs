use crate::common::{Direction, Elevator};
use crate::gpio::{engine_control::EngineControl, pid::PidController};
use std::thread;
use std::time::Duration;

#[test]
fn move_elevator() {
    // Arrange
    let mut elevator1 = EngineControl::new(Elevator::One);
    let mut elevator2 = EngineControl::new(Elevator::Two);

    // Act
    elevator1.set_direction(Direction::Up);
    elevator1.set_potency(1.0);

    elevator2.set_direction(Direction::Up);
    elevator2.set_potency(1.0);

    thread::sleep(Duration::from_secs(1));

    elevator1.set_direction(Direction::Down);
    elevator1.set_potency(1.0);

    elevator2.set_direction(Direction::Down);
    elevator2.set_potency(1.0);

    thread::sleep(Duration::from_secs(1));

    // Cleanup
    elevator1.set_direction(Direction::Stop);
    elevator1.set_potency(0.0);

    elevator2.set_direction(Direction::Stop);
    elevator2.set_potency(0.0);
}

#[test]
fn pid() {
    // Arrange
    let mut pid = PidController::new();

    // Act
    let (potency_1, direction_1) = pid.get_control_signal(0, 25000);
    let (potency_2, direction_2) = pid.get_control_signal(25000, 0);

    // Assert
    assert_eq!(potency_1, 1.0);
    assert_eq!(direction_1, Direction::Up);

    assert_eq!(potency_2, 1.0);
    assert_eq!(direction_2, Direction::Down);
}
