use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};
use std::thread;
use std::time::Duration;

const ADC_I2C_ADDR : u16 = 0x48;

const SCALE_6P144_UNIT : f32 = 9.375e-5;
const SCALE_4P096_UNIT : f32 = 6.23e-5;
const SCALE_2P048_UNIT : f32 = 3.125e-5;
const SCALE_1P024_UNIT : f32 = 1.5625e-05;
const SCALE_0P512_UNIT : f32 = 7.8125e-06;
const SCALE_0P256_UNIT : f32 = 3.90625e-06;


mod regs {
    pub const CONV  : u8 = 0x00;
    pub const CONF  : u8 = 0x01;
    pub const LOTHD : u8 = 0x02;
    pub const HITHD : u8 = 0x03;
}

pub mod misc_cfg {
    pub const FIRE_SINGLE_CONV  : u16 = 1 << 15;
    pub const COMP_ALRM_ACTV_HI : u16 = 1 << 3;
    pub const COMP_LAT_ACTV     : u16 = 1 << 2;
}

#[derive(Copy, Clone)]
pub enum ConvStatus {
    NotOver = 0,
    Over    = 1 << 15,
}

#[derive(Copy, Clone)]
pub enum ConvMode {
    Continuous = 0,
    SingleShot = 1 << 8,
}

#[derive(Copy, Clone)]
pub enum CompQueueMode {
    CompQueue1        = 0 << 0,
    CompQueue2        = 1 << 0,
    CompQueue4        = 2 << 0,
    CompQueueNotActvd = 3 << 0,
}

#[derive(Copy, Clone)]
pub enum DataRate {
    DataRate8PerSec   = 0 << 5, 
    DataRate16PerSec  = 1 << 5, 
    DataRate32PerSec  = 2 << 5, 
    DataRate64PerSec  = 3 << 5, 
    DataRate128PerSec = 4 << 5, 
    DataRate250PerSec = 5 << 5, 
    DataRate475PerSec = 6 << 5, 
    DataRate860PerSec = 7 << 5,    
}

#[derive(Copy, Clone)]
pub enum Channel {
    ChDiffAn0An1 = 0 << 12,
    ChDiffAn0An3 = 1 << 12,
    ChDiffAn1An3 = 2 << 12,
    ChDiffAn2An3 = 3 << 12,
    ChRelAn0     = 4 << 12,
    ChRelAn1     = 5 << 12,
    ChRelAn2     = 6 << 12,
    ChRelAn3     = 7 << 12,
}

#[derive(Copy, Clone)]
pub enum FullScaleRange {
    Fsr6p144  = 0 << 9,
    Fsr4p096  = 1 << 9,
    Fsr2p048  = 2 << 9,
    Fsr1p024  = 3 << 9,
    Fsr0p512  = 4 << 9,
    Fsr0p256  = 5 << 9,
}

#[derive(Copy, Clone)]
pub struct AirQualityConf {
    pub cfg : u16,
    pub ch : Channel,
    pub mode : ConvMode,
    pub dr : DataRate,
    pub fsr : FullScaleRange,
    pub comp_mode : CompQueueMode,
}

impl AirQualityConf {
    pub fn as_bit_msk(&self) -> u16 {
        self.cfg |
        self.ch as u16 |
        self.mode as u16 |
        self.dr as u16 |
        self.fsr as u16 |
        self.comp_mode as u16
    }
}

pub struct AirQuality {
    aqc : AirQualityConf
}

impl AirQuality {
    pub fn new(aqc : AirQualityConf) -> Result<AirQuality, LinuxI2CError> {
        let conf = aqc.as_bit_msk();
        let mut dev = LinuxI2CDevice::new("/dev/i2c-1", ADC_I2C_ADDR)?;
        dev.smbus_write_i2c_block_data(regs::CONF, &[(conf >> 8) as u8, conf as u8])?;

        Ok(AirQuality{
            aqc,
        })
    }

    pub fn get_raw_val(&self) -> Result<u16, LinuxI2CError> {
        let mut dev = LinuxI2CDevice::new("/dev/i2c-1", ADC_I2C_ADDR)?;
        let conf = self.aqc.as_bit_msk() | misc_cfg::FIRE_SINGLE_CONV;
        dev.smbus_write_i2c_block_data(regs::CONF, &[(conf >> 8) as u8, conf as u8])?;
        loop { 
            thread::sleep(Duration::new(0, 10e6 as u32));
            let status = dev.smbus_read_i2c_block_data(regs::CONF, 2)?;
            let status = ((status[0] as u16) << 8) + (status[1] as u16);
            if status & (ConvStatus::Over as u16) != 0 { break; }
        }
        let val = dev.smbus_read_i2c_block_data(regs::CONV, 2)?;
        Ok((((val[0] as u16) << 8) + (val[1] as u16)) as u16)
    }

    pub fn get_voltage(&self) -> Result<f32, LinuxI2CError> {
        let raw = self.get_raw_val()?;
        let voltage = match self.aqc.fsr {
            FullScaleRange::Fsr6p144 => raw as f32 * SCALE_6P144_UNIT,
            FullScaleRange::Fsr4p096 => raw as f32 * SCALE_4P096_UNIT,
            FullScaleRange::Fsr2p048 => raw as f32 * SCALE_2P048_UNIT,
            FullScaleRange::Fsr1p024 => raw as f32 * SCALE_1P024_UNIT,
            FullScaleRange::Fsr0p512 => raw as f32 * SCALE_0P512_UNIT,
            FullScaleRange::Fsr0p256 => raw as f32 * SCALE_0P256_UNIT,
        };
        Ok(
            match self.aqc.ch {
                Channel::ChDiffAn0An1 | Channel::ChDiffAn0An3 | Channel::ChDiffAn1An3 | Channel::ChDiffAn2An3 => {
                    voltage
                },
                _ => voltage * 2.0,
            }
        )
    }
}

