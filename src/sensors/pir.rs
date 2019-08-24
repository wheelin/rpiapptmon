use rppal::gpio;
use std::thread;
use std::time::Duration;

pub struct PirSensor{
    pin : gpio::InputPin,
}

impl PirSensor {
    pub fn new(rpi_pin_nb : u8) -> Result<PirSensor, gpio::Error> {
        let pin = gpio::Gpio::new()?.get(rpi_pin_nb)?.into_input_pulldown();
        
        Ok(PirSensor {
            pin,
        })
    }

    pub fn on_detection<F>(self, cb : F) where F : Fn() + Send + 'static {
        thread::spawn(move || {
            loop {
                if self.pin.is_high() {
                    cb();
                }
                thread::sleep(Duration::from_secs(5));
            }
        });
    }
}