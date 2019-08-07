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

const ACCESS_TOKEN : &str = "Rbq9v5HG1b023o2rPcEZ";
const ERR_CNTR_MAX : u8   = 10;
const LOG_FILE     : &str = "/home/pi/log.txt";

fn log_to_file<T : Into<String>>(msg : T) -> Result<(), std::io::Error> {
    let now = Local::now();
    let mut file = OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(LOG_FILE)?;

    writeln!(file, "{} [{}]", 
        now.format("%d/%m/%Y-%H:%M:%S"),
        msg.into(),
    )?;
    Ok(())
}


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

        let data = json!({
            "temperature" : bmp.read_temperature()?,
            "pressure" : bmp.read_pressure(Oss::Oss4)?,
            "humidity" : hum.get_humidity()?,
            "light_clear" : light_val.0,
            "light_red"   : light_val.1,
            "light_green" : light_val.2,
            "light_blue"  : light_val.3,
            "airquality"  : airq.get_ratio_rs_r()?,
        });
        let request = format!("http://localhost:8080/api/v1/{}/telemetry", ACCESS_TOKEN);
        match sndr.post(&request).json(&data).send() {
            Ok(_) => {
                cntr = 0;
            },
            Err(e) => {
                cntr += 1;
                log_to_file(format!("Error (counter = {}) while sending data : {:?}", cntr, e))?;
                if cntr > ERR_CNTR_MAX {
                    log_to_file(format!("Error : max (={}) reached", ERR_CNTR_MAX))?;
                    panic!("Too many errors while attempting to send data to cloud");
                };
            },
        }
        log_to_file("Data sent with succes to the cloud.")?;

        log_to_file(format!("temperature : {} *C", bmp.read_temperature()?))?;
        log_to_file(format!("pressure    : {} Pa", bmp.read_pressure(Oss::Oss4)?))?;
        log_to_file(format!("humidity    : {} %" , hum.get_humidity()?))?;
        log_to_file(format!("airquality  : {}"   , airq.get_ratio_rs_r()?))?;
        log_to_file(format!("light_clear : {}"   , light_val.0))?;
        log_to_file(format!("light_red   : {}"   , light_val.1))?;
        log_to_file(format!("light_green : {}"   , light_val.2))?;
        log_to_file(format!("light_blue  : {}"   , light_val.3))?;

        thread::sleep(Duration::new(30, 0));
    }
    
    Ok(())
}