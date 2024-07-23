use crate::i2c::bme280::Bme280;

#[test]
fn measure() {
    // Arrange
    let mut bme280 = Bme280::new();

    // Act
    let measurements = bme280.measure().unwrap();

    // Assert
    assert!(measurements.temperature > 0.0 && measurements.temperature < 50.0);
    assert!(measurements.pressure > 70000.0 && measurements.pressure < 100000.0);
}
