use crate::common::{Direction, Elevator, Floor};
use crate::i2c::{bme280::BME280, ssd1306::SSD1306};

#[test]
fn measure() {
    // Arrange
    let mut bme280 = BME280::new();

    // Act
    let temperature_1 = bme280.measure_temperature(Elevator::One);
    let temperature_2 = bme280.measure_temperature(Elevator::Two);

    // Assert
    assert!(temperature_1 > 0.0 && temperature_2 < 50.0);
    assert!(temperature_2 > 0.0 && temperature_2 < 50.0);
}

#[test]
fn screen_update() {
    // Arrange
    let mut ssd1306 = SSD1306::new();

    // Act
    ssd1306.update_temperature(Elevator::One, 25.0);
    ssd1306.update_temperature(Elevator::Two, 30.0);

    ssd1306.update_floor(Elevator::One, Floor::First);
    ssd1306.update_floor(Elevator::Two, Floor::Ground);

    ssd1306.update_direction(Elevator::One, Direction::Up);
    ssd1306.update_direction(Elevator::Two, Direction::Down);

    ssd1306.update_floor(Elevator::Two, Floor::Third);
    ssd1306.update_direction(Elevator::Two, Direction::Stop);

    ssd1306.update_floor(Elevator::One, Floor::Ground);
    ssd1306.update_direction(Elevator::One, Direction::Stop);

    ssd1306.update_temperature(Elevator::One, 20.0);
    ssd1306.update_temperature(Elevator::Two, 25.0);

    // Assert
    // No panic
}
