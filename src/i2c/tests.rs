use crate::{i2c::bme280, uart::esp32::Elevator};

#[test]
fn measure() {
    // Arrange (no need to mock the file system)

    // Act
    let temperature_1 = bme280::measure_temperature(Elevator::One).unwrap();
    let temperature_2 = bme280::measure_temperature(Elevator::Two).unwrap();

    // Assert
    assert!(temperature_1 > 0.0 && temperature_2 < 50.0);
    assert!(temperature_2 > 0.0 && temperature_2 < 50.0);
}
