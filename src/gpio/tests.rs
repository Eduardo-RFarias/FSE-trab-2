use crate::common::Elevator;
use crate::gpio::elevator_control::{Direction, ElevatorControl};
use std::thread;
use std::time::Duration;

#[test]
fn move_elevator() {
    // Arrange
    let mut gpio = ElevatorControl::new();

    // Act
    gpio.set_direction(Elevator::One, Direction::Up);
    gpio.set_potency(Elevator::One, 1.0);

    gpio.set_direction(Elevator::Two, Direction::Up);
    gpio.set_potency(Elevator::Two, 0.7);

    // Assert
    // No assertion needed, just testing that the function does not panic

    // Wait for the elevator to move
    thread::sleep(Duration::from_secs(1));

    // Cleanup
    gpio.set_direction(Elevator::One, Direction::Idle);
    gpio.set_potency(Elevator::One, 0.0);

    gpio.set_direction(Elevator::Two, Direction::Idle);
    gpio.set_potency(Elevator::Two, 0.0);
}
