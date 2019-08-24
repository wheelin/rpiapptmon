mod hmi;
mod sensors;

use reqwest::Client;
use sensors::ads1115::{Ads1115, Ads1115Channel, Ads1115Conf, Channel};
use sensors::airq::*;
use sensors::humidity::*;
use sensors::light;
use sensors::pir;
use sensors::press_temp::*;
use std::thread;
use std::time::Duration;

const ACCESS_TOKEN: &str = "NVSTS3TBBO8O4YEH";

fn when_detected() {
    println!("Detected");
}

fn main() -> Result<(), std::io::Error> {
    let hum = Hts221::new(Hts221Cfg::default())?;

    let mut bmp = BMP180::new()?;

    bmp.self_test().unwrap();

    let adc = Ads1115::new(Ads1115Conf::default())?;
    let adc_ch1 = Ads1115Channel::new(&adc, Channel::ChRelAn0);
    let adc_ch2 = Ads1115Channel::new(&adc, Channel::ChRelAn1);
    let airq = AirQuality::new(&adc_ch1);

    let pir = pir::PirSensor::new(18).unwrap();
    pir.on_detection(when_detected);
    let pr = light::PhotoResistor::new(&adc_ch2, 10000);

    let sndr = Client::new();

    loop {
        //let request = format!("https://api.thingspeak.com/update?api_key={}&field1={}&field2={}&field3={}&field4={}&field5={}&field6={}&field7={}&field8={}",
        //    ACCESS_TOKEN,
        //    bmp.read_temperature()?,
        //    bmp.read_pressure(Oss::Oss4)?,
        //    hum.get_humidity()?,
        //    airq.get_ratio_rs_r()?,
        //    light_val.0,
        //    light_val.1,
        //    light_val.2,
        //    light_val.3,
        //);
        //sndr.get(&request).send().unwrap();
        //thread::sleep(Duration::new(60, 0));
        println!("light : {}", pr.get_ratio()?);
        thread::sleep(Duration::from_secs(1));
    }
    Ok(())
}
