use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};
use std::thread;
use std::time::Duration;


const LIGHT_I2C_ADDR : u16 = 0x29;
pub enum Registers {
    Enable  = 0x00,
    Atime   = 0x01,
    Wtime   = 0x03,
    Ailtl   = 0x04,
    Ailth   = 0x05,
    Aihtl   = 0x06,
    Aihth   = 0x07,
    Pers    = 0x0C,
    Config  = 0x0D,
    Control = 0x0F,
    Id      = 0x12,
    Status  = 0x13,
    Cdatal  = 0x14,
    Cdatah  = 0x15,
    Rdatal  = 0x16,
    Rdatah  = 0x17,
    Gdatal  = 0x18,
    Gdatah  = 0x19,
    Bdatal  = 0x1A,
    Bdatah  = 0x1B,
}

const LIGHT_REG_CMD_MSK        : u8 = 1 << 7;
const LIGHT_CMD_PRTCL_BYTE     : u8 = 0 << 5;
const LIGHT_CMD_PRTCL_AUTO_INC : u8 = 1 << 5;
const LIGHT_CMD_PRTCL_SPEC_FN  : u8 = 2 << 5;
const LIGHT_SPEC_FN_CLEAR_IRQ  : u8 = 3 << 1;

const LIGHT_ENA_MSK_IRQ_EN     : u8 = 1 << 4;
const LIGHT_ENA_MSK_WEN        : u8 = 1 << 3;
const LIGHT_ENA_MSK_RGBC_EN    : u8 = 1 << 1;
const LIGHT_ENA_MSK_PWR_EN     : u8 = 1 << 0;

const LIGHT_CFG_MSK_WLONG_EN   : u8 = 1 << 1;

pub enum Gain {
    X1  = 0,
    X4  = 1,
    X16 = 2,
    X60 = 3,    
}

pub enum Color {
    Clear,
    Red, 
    Green,
    Blue,
}

const LIGHT_ID_VAL : u8 = 0x44;

const LIGHT_STATUS_MSK_CLR_CH_INT : u8 = 1 << 4;
const LIGHT_STATUS_MSK_MEAS_VALID : u8 = 1 << 0;

fn time_reg_conv(ms : f32) -> u8 {
    let ms = if ms > 700.0 {
        700.0
    } else if ms < 2.4 {
        2.4
    } else {
        ms
    };

    (256 - std::cmp::min(256, ms.ceil() as u16)) as u8
}

pub struct Light {

}

impl Light {
    pub fn new() -> Light {
        let mut i2c = LinuxI2CDevice::new("/dev/i2c-1", LIGHT_I2C_ADDR).unwrap();
        i2c.smbus_write_byte_data(LIGHT_REG_CMD_MSK | LIGHT_CMD_PRTCL_BYTE | (Registers::Enable)  as u8, LIGHT_ENA_MSK_PWR_EN).unwrap();
        i2c.smbus_write_byte_data(LIGHT_REG_CMD_MSK | LIGHT_CMD_PRTCL_BYTE | (Registers::Atime)   as u8, time_reg_conv(30.0)).unwrap();
        i2c.smbus_write_byte_data(LIGHT_REG_CMD_MSK | LIGHT_CMD_PRTCL_BYTE | (Registers::Wtime)   as u8, time_reg_conv(30.0)).unwrap();
        i2c.smbus_write_byte_data(LIGHT_REG_CMD_MSK | LIGHT_CMD_PRTCL_BYTE | (Registers::Control) as u8, Gain::X16 as u8).unwrap();

        let id = i2c.smbus_read_byte_data(LIGHT_REG_CMD_MSK | LIGHT_CMD_PRTCL_BYTE | (Registers::Id as u8)).unwrap();
        if id != LIGHT_ID_VAL { panic!(format!("Problem while reading ID of light sensor. ID (=0x{:02X}) != 0x{:02X}", id, LIGHT_ID_VAL)) }
        Light{}
    }

    pub fn get_all(&self) -> (u32, u32, u32, u32) {
        let mut i2c = LinuxI2CDevice::new("/dev/i2c-1", LIGHT_I2C_ADDR).unwrap();
        i2c.smbus_write_byte_data(LIGHT_REG_CMD_MSK | LIGHT_CMD_PRTCL_BYTE | (Registers::Enable) as u8, LIGHT_ENA_MSK_PWR_EN | LIGHT_ENA_MSK_RGBC_EN).unwrap();
        let status = i2c.smbus_read_byte_data(LIGHT_REG_CMD_MSK | LIGHT_CMD_PRTCL_BYTE | (Registers::Status) as u8).unwrap();
        loop {
            thread::sleep(Duration::new(0, 10000000));
            let status = i2c.smbus_read_byte_data(LIGHT_REG_CMD_MSK | LIGHT_CMD_PRTCL_BYTE | (Registers::Status) as u8).unwrap();
            if status & LIGHT_STATUS_MSK_MEAS_VALID != 0 {
                break;
            }
        }

        let values = i2c.smbus_read_i2c_block_data(LIGHT_REG_CMD_MSK | LIGHT_CMD_PRTCL_AUTO_INC | (Registers::Cdatal) as u8, 8).unwrap();
        i2c.smbus_write_byte_data(LIGHT_REG_CMD_MSK | LIGHT_CMD_PRTCL_BYTE | (Registers::Enable) as u8, LIGHT_ENA_MSK_PWR_EN).unwrap();
        (
            ((values[0] as u32) << 8) + (values[1] as u32), 
            ((values[1] as u32) << 8) + (values[3] as u32), 
            ((values[2] as u32) << 8) + (values[5] as u32), 
            ((values[3] as u32) << 8) + (values[7] as u32)
        )
    }

    pub fn get(&self, color : Color) -> u32 {
        let all = self.get_all();
        match color {
            Color::Clear => all.0,
            Color::Red   => all.1,
            Color::Green => all.2,
            Color::Blue  => all.3,
        }
    }
}

