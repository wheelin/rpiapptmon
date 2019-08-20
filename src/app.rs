use crate::sensors::{airq, humidity, light, press_temp};
use i2cdev::linux::LinuxI2CError;
use std::time::SystemTime;
use structopt::StructOpt;

use crate::sensors::{airq::*, humidity::*, light::*, press_temp::*};
use std::fmt;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(StructOpt, Debug)]
#[structopt(name = "rpiapptmon")]
pub struct CliOpts {
    /// Output record file
    #[structopt(short, long, default_value = "/home/alarm/log.csv")]
    pub output: String,

    /// Measurement period in seconds
    #[structopt(short, long, default_value = "60")]
    pub period: u64,
    /// Output values on stdout
    #[structopt(short = "s", long = "print")]
    pub prnt_stdout: bool,
}

pub struct SensorValues {
    pub temperature: f32,
    pub pressure: i32,
    pub humidity: f32,
    pub airquality: f32,
    pub light_clear: u32,
    pub light_red: u32,
    pub light_green: u32,
    pub light_blue: u32,
}

impl SensorValues {
    pub fn csv_header() -> String {
        "datetime;temperature[*C];pressure[hPa];humidity[rh%];airquality[-];light_clear[-];light_red[-];light_green[-];light_blue[-]\n".to_owned()
    }

    pub fn to_csv_record(&self) -> String {
        format!(
            "{:?};{};{};{};{};{};{};{};{}\n",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            self.temperature,
            self.pressure,
            self.humidity,
            self.airquality,
            self.light_clear,
            self.light_red,
            self.light_green,
            self.light_blue,
        )
    }
}

impl fmt::Display for SensorValues {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}
Pressure = {} hPa
Temperature = {} *C
Humidity = {} %RH
Clear light = {}
Red light = {}
Green light = {}
Blue light = {}
Air quality = {}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            self.pressure,
            self.temperature,
            self.humidity,
            self.light_clear,
            self.light_red,
            self.light_green,
            self.light_blue,
            self.airquality
        )
    }
}

pub struct SensorsSet {
    hum: humidity::HumiditySensor,
    airq: airq::AirQuality,
    bmp: press_temp::BMP180,
    light: light::Light,
}

impl SensorsSet {
    pub fn new() -> Result<SensorsSet, LinuxI2CError> {
        let hum = HumiditySensor::new(
            HumAvgCfg::AvgSmplx4,
            TempAvgCfg::AvgSmplx4,
            OutDataRate::SingleShot,
            ctrl1_msks::PWR_UP | ctrl1_msks::BDU_ENA,
            ctrl2_msks::ONE_SHOT_EN,
            0,
        )?;

        let mut bmp = BMP180::new()?;

        bmp.self_test().unwrap();

        let light = light::Light::new();

        let airq = AirQuality::new(AirQualityConf {
            cfg: misc_cfg::FIRE_SINGLE_CONV,
            ch: Channel::ChRelAn0,
            mode: ConvMode::SingleShot,
            dr: DataRate::DataRate32PerSec,
            fsr: FullScaleRange::Fsr6p144,
            comp_mode: CompQueueMode::CompQueueNotActvd,
        })?;

        Ok(SensorsSet {
            hum,
            airq,
            bmp,
            light,
        })
    }

    pub fn get_sensors_values(&mut self) -> Result<SensorValues, LinuxI2CError> {
        let light_val = self.light.get_all();

        Ok(SensorValues {
            temperature: self.bmp.read_temperature()?,
            pressure: self.bmp.read_pressure(Oss::Oss4)?,
            humidity: self.hum.get_humidity()?,
            airquality: self.airq.get_ratio_rs_r()?,
            light_clear: light_val.0,
            light_red: light_val.1,
            light_green: light_val.2,
            light_blue: light_val.3,
        })
    }
}

pub struct SnsrValLogger<'a> {
    header_written: bool,
    log_file: &'a str,
}

impl<'a> SnsrValLogger<'a> {
    pub fn new(f: &'a str) -> SnsrValLogger {
        SnsrValLogger {
            header_written: false,
            log_file: f,
        }
    }

    pub fn wr_snsr_vals(&mut self, rec: &SensorValues) {
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(self.log_file)
            .unwrap();

        if !self.header_written {
            file.write(&SensorValues::csv_header().as_bytes()).unwrap();
            self.header_written = true;
        }
        file.write(&rec.to_csv_record().as_bytes()).unwrap();
    }
}
