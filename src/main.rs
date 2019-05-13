mod sensors;
mod hmi;

use sensors::airq::*;
use sensors::press_temp::*;
use sensors::light;
use sensors::humidity::*;

use std::thread;
use std::time::Duration;

use hmi::matrix::*;
use hmi::color::*;

fn main() -> Result<(), std::io::Error> {
    let mut matrix = Matrix::new(String::from("/dev/fb1"), Orientation::Cw180);
    let hum = HumiditySensor::new(
        HumAvgCfg::AvgSmplx4, 
        TempAvgCfg::AvgSmplx4, 
        OutDataRate::SingleShot, 
        ctrl1_msks::PWR_UP | ctrl1_msks::BDU_ENA, 
        ctrl2_msks::ONE_SHOT_EN, 
        0
    )?;
    let mut bmp = BMP180::new()?;
    bmp.self_test().unwrap();

    println!("H:{:.1}%", hum.get_humidity()?);
    println!("P:{:.2}HPa", bmp.read_pressure(Oss::Oss1)? as f32 / 100.0);
    println!("T:{:.1}*C", bmp.read_temperature()?);

    matrix.write(format!("H:{:.1}%", hum.get_humidity()?), colors::LIGHT_BLUE, colors::BLACK, Duration::new(0, 500000000))?;
    matrix.write(format!("P:{:.1}HPa", bmp.read_pressure(Oss::Oss1)?), colors::GREEN, colors::BLACK, Duration::new(0, 500000000))?;
    matrix.write(format!("T:{:.1}*C", bmp.read_temperature()?), colors::LIGHT_RED, colors::BLACK, Duration::new(0, 500000000))?;
    
    Ok(())
}


fn main_h() -> Result<(), std::io::Error> {
    let hum = HumiditySensor::new(
        HumAvgCfg::AvgSmplx4, 
        TempAvgCfg::AvgSmplx4, 
        OutDataRate::DataRate12p5Hz, 
        ctrl1_msks::PWR_UP | ctrl1_msks::BDU_ENA, 
        ctrl2_msks::ONE_SHOT_EN, 
        0
    )?;

    println!("Temperature from humidity sensor = {}*C", hum.get_temperature()?);
    println!("Humidity = {} %rh", hum.get_humidity()?);
    
    Ok(())
}