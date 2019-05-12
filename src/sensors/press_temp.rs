use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

enum Reg {
    FirstCoeffReg = 0xAA,
    DataOutMsb = 0xF6,
    CtrlMeas = 0xF4,
    SoftReset = 0xE0,
    ID = 0xD0,
}

#[derive(Copy, Clone)]
pub enum Oss {
    Oss1,
    Oss2,
    Oss4,
    Oss8,
}

enum CtrlMeasureCmd {
    Temperature = 0x2E,
    PressureSingle = 0x34,
    PressureOss2 = 0x74,
    PressureOss4 = 0xB4,
    PressureOss8 = 0xF4,
}

pub struct BMP180 {
    ac1: i16,
    ac2: i16,
    ac3: i16,
    ac4: u16,
    ac5: u16,
    ac6: u16,
    b1: i16,
    b2: i16,
    mb: i16,
    mc: i16,
    md: i16,
}

const ADDR: u16 = 0x77;
const ID: u8 = 0x55;
const RESET_ORDER: u8 = 0xB6;
const MEAS_READY_MASK: u8 = 0x20;

fn to_i16(msb: u8, lsb: u8) -> i16 {
    (((msb as u16) << 8) | (lsb as u16)) as i16
}

fn to_u16(msb: u8, lsb: u8) -> u16 {
    ((msb as u16) << 8) | lsb as u16
}

impl BMP180 {
    pub fn new() -> Result<BMP180, LinuxI2CError> {
        let mut tmpi2c = LinuxI2CDevice::new("/dev/i2c-1", ADDR).unwrap();

        let compensation_coeffs = tmpi2c.smbus_read_i2c_block_data(Reg::FirstCoeffReg as u8, 22)?;

        Ok(
            BMP180 {
                ac1: to_i16(compensation_coeffs[ 0], compensation_coeffs[ 1]),
                ac2: to_i16(compensation_coeffs[ 2], compensation_coeffs[ 3]),
                ac3: to_i16(compensation_coeffs[ 4], compensation_coeffs[ 5]),
                ac4: to_u16(compensation_coeffs[ 6], compensation_coeffs[ 7]),
                ac5: to_u16(compensation_coeffs[ 8], compensation_coeffs[ 9]),
                ac6: to_u16(compensation_coeffs[10], compensation_coeffs[11]),
                b1:  to_i16(compensation_coeffs[12], compensation_coeffs[13]),
                b2:  to_i16(compensation_coeffs[14], compensation_coeffs[15]),
                mb:  to_i16(compensation_coeffs[16], compensation_coeffs[17]),
                mc:  to_i16(compensation_coeffs[18], compensation_coeffs[19]),
                md:  to_i16(compensation_coeffs[20], compensation_coeffs[21]),
            }
        )
    }

    pub fn self_test(&mut self) -> Result<(), ()> {
        let mut tmpi2c = LinuxI2CDevice::new("/dev/i2c-1", ADDR).unwrap();
        let tmp_id = match tmpi2c.smbus_read_byte_data(Reg::ID as u8) {
            Ok(d) => d,
            Err(_) => return Err(()),
        };
        if tmp_id != ID {
            return Err(());
        } else {
            return Ok(());
        }
    }

    pub fn print_comp_coeffs(&self) {
        println!("ac1 = {}", self.ac1);
        println!("ac2 = {}", self.ac2);
        println!("ac3 = {}", self.ac3);
        println!("ac4 = {}", self.ac4);
        println!("ac5 = {}", self.ac5);
        println!("ac6 = {}", self.ac6);
        println!("b1 = {}", self.b1);
        println!("b2 = {}", self.b2);
        println!("mb = {}", self.mb);
        println!("mc = {}", self.mc);
        println!("md = {}", self.md);
    }

    fn read_raw_temperature(&mut self) -> Result<i32, LinuxI2CError> {
        let mut tmpi2c = LinuxI2CDevice::new("/dev/i2c-1", ADDR)?;
        let mut data_ready = false;
        while data_ready == false {
            data_ready = (tmpi2c.smbus_read_byte_data(Reg::CtrlMeas as u8)? & MEAS_READY_MASK) == 0;
        }
        let raw_data = tmpi2c.smbus_read_i2c_block_data(Reg::DataOutMsb as u8, 3)?;
        let raw_temp = ((raw_data[0] as i32) << 8) | (raw_data[1] as i32);
        Ok(raw_temp)
    }

    fn read_raw_pressure(&mut self, oss: Oss) -> Result<i32, LinuxI2CError> {
        let mut tmpi2c = LinuxI2CDevice::new("/dev/i2c-1", ADDR)?;
        let cmd = match oss {
            Oss::Oss1 => CtrlMeasureCmd::PressureSingle,
            Oss::Oss2 => CtrlMeasureCmd::PressureOss2,
            Oss::Oss4 => CtrlMeasureCmd::PressureOss4,
            Oss::Oss8 => CtrlMeasureCmd::PressureOss8,
        };

        tmpi2c.smbus_write_byte_data(Reg::CtrlMeas as u8, cmd as u8)?;
        let mut data_ready = false;
        while data_ready == false {
            data_ready = (tmpi2c.smbus_read_byte_data(Reg::CtrlMeas as u8)?) & MEAS_READY_MASK == 0;
        }
        let raw_data = tmpi2c.smbus_read_i2c_block_data(Reg::DataOutMsb as u8, 3)?;
        let raw_pressure = (((raw_data[0] as i32) << 16) | ((raw_data[1] as i32) << 8) |
                            (raw_data[2] as i32)) >> (8 - (oss as u8));
        Ok(raw_pressure)
    }

    pub fn read_pressure(&mut self, oss: Oss) -> Result<i32, LinuxI2CError> {
        let ut = self.read_raw_temperature()?;
        let up = self.read_raw_pressure(oss)?;

        let mut x1 = (ut - (self.ac6 as i32)) * (self.ac5 as i32) >> 15;
        let mut x2 = ((self.mc as i32) << 11) / (x1 + self.md as i32);
        let b5 = x1 + x2;
        let b6 = b5 - 4000;
        x1 = ((self.b2 as i32) * ((b6 * b6) >> 12)) >> 11;
        x2 = ((self.ac2 as i32) * b6) >> 11;
        let mut x3 = x1 + x2;
        let b3 = ((((self.ac1 as i32) * 4 + x3) << (oss as i32)) + 2) / 4;
        x1 = ((self.ac3 as i32) * b6) >> 13;
        x2 = ((self.b1 as i32) * ((b6.pow(2) >> 12))) >> 16;
        x3 = ((x1 + x2) + 2) >> 2;
        let b4 = ((self.ac4 as u32) * ((x3 + 32768) as u32)) >> 15;
        let b7 = ((up - b3) as u32) * (50000 >> (oss as u32));
        let p = if b7 < 0x80000000 {
            (b7 * 2) / b4
        } else {
            (b7 / b4) * 2
        };
        x1 = (p >> 8).pow(2) as i32;
        x1 = (x1 * 3038) >> 16;
        x2 = (-7357 * (p as i32)) >> 16;
        let p = (p as i32) + ((x1 + x2 + 3791) >> 4);
        return Ok(p);
    }

    pub fn read_temperature(&mut self) -> Result<f32, LinuxI2CError> {
        let ut = self.read_raw_temperature()?;
        let x1 = ((ut - (self.ac6 as i32)) * (self.ac5 as i32)) >> 15;
        let x2 = ((self.mc as i32) << 11) / (x1 + (self.md as i32));
        let b5 = x1 + x2;
        let t = (b5 + 8) >> 4;
        Ok((t as f32) / 10.0)
    }
}