use super::adc::*;

pub struct PhotoResistor<'a, T> where T : AdcCh {
    adc_channel : &'a T,
    r2_val      : u32,
}

impl<T> PhotoResistor<'_, T> where T : AdcCh {
    pub fn new(adc_channel : &T, r2 : u32) -> PhotoResistor<T> {
        PhotoResistor {
            adc_channel : adc_channel,
            r2_val : r2
        }
    }

    pub fn get_ratio(&self) -> Result<f32, AdcErr> {
        let vsig = self.adc_channel.get_voltage()?;
        let vref = self.adc_channel.get_ref_voltage();

        let r1_on_r2 = (vref - vsig)/vsig;
        Ok(r1_on_r2)
    }
}