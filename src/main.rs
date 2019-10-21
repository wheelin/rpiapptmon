mod hmi;

use ads1115::*;
use airqualitysensor::*;
use hts221::*;
use lightsensor::*;
use bmp180::bmp180::*;

use std::net::{TcpStream, TcpListener};
use std::io::{Read, Write};

const TCP_PRESSURE_REQ    : u8 = 0x01;
const TCP_TEMPERATURE_REQ : u8 = 0x02;
const TCP_LIGHT_REQ       : u8 = 0x03;
const TCP_HUMIDITY_REQ    : u8 = 0x04;
const TCP_AIR_QUALITY_REQ : u8 = 0x05;
const TCP_IP_REQ          : u8 = 0x06;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hum = Hts221::new(Hts221Cfg::default())?;

    let mut bmp = Bmp180::new()?;

    bmp.self_test().unwrap();

    let adc = Ads1115::new(Ads1115Conf::default())?;
    let adc_ch1 = Ads1115Channel::new(&adc, Channel::ChRelAn0);
    let adc_ch2 = Ads1115Channel::new(&adc, Channel::ChRelAn1);
    let airq = AirQuality::new(&adc_ch1);
    let pr = lightsensor::PhotoResistor::new(&adc_ch2);

	bmp.read_pressure(Oss::Oss8).unwrap_or(0);
	bmp.read_temperature().unwrap_or(0.0);
	hum.get_humidity().unwrap_or(0.0);
	airq.get_ratio_rs_r().unwrap_or(0.0);
	pr.get_ratio().unwrap_or(0.0);

    let listener = TcpListener::bind("127.0.0.1:9999").unwrap();

    for stream in listener.incoming() {
        let mut data : Vec<u8> = Vec::new();
        let mut stream = stream?;
        if stream.read_to_end(&mut data)? == 1 {
            println!("Data : {:?}", data);
            match data[0] {
                TCP_PRESSURE_REQ    => writeln!(stream, "{}", bmp.read_pressure(Oss::Oss8)?)?,
                TCP_TEMPERATURE_REQ => writeln!(stream, "{}", bmp.read_temperature()?)?,
                TCP_LIGHT_REQ       => writeln!(stream, "{}", pr.get_ratio()?)?,
                TCP_HUMIDITY_REQ    => writeln!(stream, "{}", hum.get_humidity()?)?,
                TCP_AIR_QUALITY_REQ => writeln!(stream, "{}", airq.get_ratio_rs_r()?)?,
                TCP_IP_REQ          => writeln!(stream, "{}", get_external_ip()?)?,
                _                   => (),
            }
        } 
    }

    Ok(())
}

fn get_external_ip() -> Result<String, reqwest::Error> {
    let req = "http://ip.changeip.com";
    let mut resp = reqwest::get(req)?;
    Ok(resp
        .text()?
        .split('\n')
        .collect::<Vec<&str>>()
        .get(0)
        .unwrap()
        .to_owned()
        .to_string())
}