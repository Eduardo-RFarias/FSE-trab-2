use bme280::{i2c::BME280 as Device, Error, Measurements};
use linux_embedded_hal::{Delay, I2CError, I2cdev};

pub struct Bme280 {
    delay: Delay,
    device: Device<I2cdev>,
}

impl Bme280 {
    pub fn new() -> Self {
        let i2c = I2cdev::new("/dev/i2c-1").unwrap();
        let mut device = Device::new_primary(i2c);
        let mut delay = Delay;
        device.init(&mut delay).unwrap();

        Bme280 { delay, device }
    }

    pub fn measure(&mut self) -> Result<Measurements<I2CError>, Error<I2CError>> {
        self.device.measure(&mut self.delay)
    }
}
