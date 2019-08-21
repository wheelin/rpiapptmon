mod sensors;
mod hmi;

use sensors::airq::*;
use sensors::press_temp::*;
use sensors::light;
use sensors::humidity::*;
use std::thread;
use std::time::Duration;
use chrono::prelude::*;
use hmi::matrix::*;
use hmi::color::*;
use reqwest::Client;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use serde_json::json;
use std::collections::HashMap;

const ACCESS_TOKEN : &str = "NVSTS3TBBO8O4YEH";

fn main() -> Result<(), std::io::Error> {
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

    let light = light::Light::new();

    let airq = AirQuality::new(
        AirQualityConf{
            cfg : misc_cfg::FIRE_SINGLE_CONV,
            ch : Channel::ChRelAn0,
            mode : ConvMode::SingleShot,
            dr : DataRate::DataRate32PerSec,
            fsr : FullScaleRange::Fsr6p144,
            comp_mode : CompQueueMode::CompQueueNotActvd,
        }
    )?;

    let sndr = Client::new();

    let mut cntr = 0;

    loop {
        let light_val = light.get_all();

        let request = format!("https://api.thingspeak.com/update?api_key={}&field1={}&field2={}&field3={}&field4={}&field5={}&field6={}&field7={}&field8={}", 
            ACCESS_TOKEN,
            bmp.read_temperature()?,
            bmp.read_pressure(Oss::Oss4)?,
            hum.get_humidity()?,
            airq.get_ratio_rs_r()?,
            light_val.0,
            light_val.1,
            light_val.2,
            light_val.3,
        );
        sndr.get(&request).send().unwrap();
        thread::sleep(Duration::new(60, 0));
    }
    
    Ok(())
}