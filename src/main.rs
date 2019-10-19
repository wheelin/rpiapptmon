mod hmi;

use ads1115::*;
use airq::*;
use hts221::*;
use photoresistor::*;
use bmp180::*;
use std::thread;
use std::time::Duration;

use jsonrpc_core::*;
use jsonrpc_tcp_server::ServerBuilder;

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hum = Hts221::new(Hts221Cfg::default())?;

    let mut bmp = BMP180::new()?;

    bmp.self_test().unwrap();

    let adc = Ads1115::new(Ads1115Conf::default())?;
    let adc_ch1 = Ads1115Channel::new(&adc, Channel::ChRelAn0);
    let adc_ch2 = Ads1115Channel::new(&adc, Channel::ChRelAn1);
    let airq = AirQuality::new(&adc_ch1);
    let pr = light::PhotoResistor::new(&adc_ch2, 10000);

    let mut io = IoHandler::default();
	
    io.add_method("pressure", |_params| {
		Ok(Value::Number(bmp.read_pressure(Oss::Oss8)?))
	});

    io.add_method("temperature", |_params| {
		Ok(Value::Number(bmp.read_temperature()?))
	});

    io.add_method("humidity", |_params| {
		Ok(Value::Number(hum.get_humidity()?))
	});

    io.add_method("airq", |_params| {
		Ok(Value::Number(airq.get_ratio_rs_r()?))
	});

    io.add_method("luminosity", |_params| {
		Ok(Value::Number(pr.get_ratio()?))
	});

	let server = ServerBuilder::new(io)
		.start(&"0.0.0.0:0".parse().unwrap())
		.expect("Server must start with no issues.");

	server.wait();
}