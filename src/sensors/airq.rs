use super::adc::{AdcCh, AdcErr};
use std::error::Error;

pub struct AirQuality<'a, T> where T : AdcCh {
    adc : &'a T,
}

impl<T> AirQuality<'_, T> where T : AdcCh {
    pub fn new(adc : &T) -> AirQuality<T> {
        AirQuality {adc}
    }

    pub fn get_ratio_rs_r(&self) -> Result<f32, AdcErr> {
        let v_out = self.adc.get_voltage()?;
        Ok(((self.adc.get_ref_voltage()/v_out) - (v_out/self.adc.get_ref_voltage()))/10.0)
    }
}

