mod hmi;
mod sensors;
mod telegram;

use sensors::ads1115::{Ads1115, Ads1115Channel, Ads1115Conf, Channel};
use sensors::airq::*;
use sensors::humidity::*;
use sensors::light;
use sensors::pir;
use sensors::press_temp::*;
use std::thread;
use std::time::Duration;

use telegram::*;

const TELEGRAM_BOT_TOKEN: &str = include_str!("../bot_token.txt");
const TELEGRAM_CHAT_ID: &str = "9542300";
const TELEGRAM_USER_ID: u32 = 9_542_300;

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
    let mut bot = TelegramBot::new(TELEGRAM_BOT_TOKEN, TELEGRAM_CHAT_ID, Some(TELEGRAM_USER_ID));

    loop {
        let resp = bot.get_unread_updates()?;
        let useless_msg_ids = resp
            .result
            .iter()
            .filter(|x| (x.message.text != "ip" && x.message.text != "Ip") && 
                        (x.message.text != "env" && x.message.text != "Env"))
            .map(|x| x.message.message_id)
            .collect::<Vec<u32>>();
        for msg_id in useless_msg_ids.iter() {
            bot.remove_message(*msg_id)?;
        }
        if resp.result.iter().any(|x| x.message.text == "ip" || x.message.text == "Ip") {
            bot.send_message(get_external_ip()?, true)?;
        }
        if resp.result.iter().any(|x| x.message.text == "env" || x.message.text == "Env") {
            let msg = format!("Temperature : {:.2} °C", bmp.read_temperature()?);
            send(bot, msg, true);
            let msg = format!("Pressure    : {:.2} hPa", bmp.read_pressure(Oss::Oss4)?);
            send(bot, msg, false);
            let msg = format!("Humidity    : {:.2} %RH", hum.get_humidity()?);
            send(bot, msg, false);
            let msg = format!("Air quality : {:.2}", airq.get_ratio_rs_r()?);
            send(bot, msg, false);
            let msg = format!("Luminosity  : {:.2}", pr.get_ratio()?);
            send(bot, msg, false);
        }
        thread::sleep(Duration::from_secs(5));
    }
    Ok(())
}

fn send(bot : &mut TelegramBot, msg : String, notif : bool)
{
    match bot.send_message(msg, notif)
    {
        Ok(_) => (),
        Err(_) => thread::sleep(Duration::from_secs(10)),
    }
}