use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

pub enum Regs {
    Id          = 0x0F,
    AvConf      = 0x10,
    CtrlReg1    = 0x20,
    CtrlReg2    = 0x21,
    CtrlReg3    = 0x22,
    Status      = 0x27,
    HumOutLo    = 0x28,
    HumOutHi    = 0x29,
    TempOutLo   = 0x2A,
    TempOutHi   = 0x2B,
    H0rHx2      = 0x30,
    H1rhx2      = 0x31,
    T0DegCx8    = 0x32,
    T1DegCx8    = 0x33,
    T1T0Msb     = 0x35,
    H0T0OutLsb  = 0x36,
    H0T0OutMsb  = 0x37,
    H1T0OutLsb  = 0x3A,
    H1T0OutMsb  = 0x3B,
    T0OutLsb    = 0x3C,
    T0OutMsb    = 0x3D,
    T1OutLsb    = 0x3E,
    T1OutMsb    = 0x3F,
}

const HUM_I2C_ADDR : u16 = 0x5F;
const HUM_ID_VAL   : u8  = 0xBC;

#[derive(Copy, Clone)]
pub enum TempAvgCfg {
    AvgSmplx2   = 0,
    AvgSmplx4   = 1,
    AvgSmplx8   = 2,
    AvgSmplx16  = 3,
    AvgSmplx32  = 4,
    AvgSmplx64  = 5,
    AvgSmplx128 = 6,
    AvgSmplx256 = 7,
}

#[derive(Copy, Clone)]
pub enum HumAvgCfg {
    AvgSmplx4   = 0,
    AvgSmplx8   = 1,
    AvgSmplx16  = 2,
    AvgSmplx32  = 3,
    AvgSmplx64  = 4,
    AvgSmplx128 = 5,
    AvgSmplx256 = 6,
    AvgSmplx512 = 7,
}

pub mod ctrl1_msks {
    pub const PWR_UP    : u8 = 1 << 7;
    pub const BDU_ENA   : u8 = 1 << 2; 
}

#[derive(Copy, Clone, PartialEq)]
pub enum OutDataRate {
    SingleShot     = 0,
    DataRate1Hz    = 1,
    DataRate7Hz    = 2,
    DataRate12p5Hz = 3,
}

pub mod ctrl2_msks {
    pub const REBOOT        : u8 = 1 << 7;
    pub const HEATER_ON     : u8 = 1 << 1;
    pub const ONE_SHOT_EN   : u8 = 1 << 0;
}

pub mod ctrl3_msks {
    pub const DRDY_H_L : u8 = 1 << 7;
    pub const NPP_OD   : u8 = 1 << 6;
    pub const DRDY_EN  : u8 = 1 << 2;
}

pub mod status_msks {
    pub const HUM_DRDY  : u8 = 1 << 1;
    pub const TEMP_DRDY : u8 = 1 << 0;
}

pub struct HumiditySensor {
    havg : HumAvgCfg,
    tavg : TempAvgCfg,
    odr  : OutDataRate,
    cfg1 : u8,
    cfg2 : u8,
    cfg3 : u8,
}

impl HumiditySensor {
    pub fn new(
        havg : HumAvgCfg, 
        tavg : TempAvgCfg, 
        odr  : OutDataRate,
        cfg1 : u8, 
        cfg2 : u8, 
        cfg3 : u8
    ) -> Result<HumiditySensor, LinuxI2CError> {
    
        let mut i2c = LinuxI2CDevice::new("/dev/i2c-1", HUM_I2C_ADDR)?;
        let id = i2c.smbus_read_byte_data(Regs::Id as u8)?;
        if id != HUM_ID_VAL { panic!("Cannot read humidity ID"); }

        let cfg = (cfg1 | (odr as u8)) & 0x87;
        i2c.smbus_write_byte_data(Regs::CtrlReg1 as u8, cfg)?;
        let cfg = cfg2 & 0x83;       
        i2c.smbus_write_byte_data(Regs::CtrlReg2 as u8, cfg)?;
        let cfg = cfg3 & 0xC4;
        i2c.smbus_write_byte_data(Regs::CtrlReg3 as u8, cfg)?;

        let avg = ((tavg as u8) << 3 | (havg as u8)) & 0x3F;
        i2c.smbus_write_byte_data(Regs::AvConf as u8, avg)?;

        
        Ok(
            HumiditySensor {
                havg,
                tavg,
                odr,
                cfg1,
                cfg2,
                cfg3,
            }
        )
    }

    pub fn get_temperature(&self) -> Result<f32, LinuxI2CError> {
        let mut i2c = LinuxI2CDevice::new("/dev/i2c-1", HUM_I2C_ADDR)?;

        let cfg = self.cfg1 | ctrl1_msks::PWR_UP;
        i2c.smbus_write_byte_data(Regs::CtrlReg1 as u8, cfg)?;
        
        let t0_deg_c = i2c.smbus_read_byte_data(Regs::T0DegCx8 as u8)?;
        let t1_deg_c = i2c.smbus_read_byte_data(Regs::T1DegCx8 as u8)?;
        let t10_deg_c_msb = i2c.smbus_read_byte_data(Regs::T1T0Msb as u8)?;
        let t0_deg_c = ((( (t10_deg_c_msb & 0x03)       as u16) << 8) + (t0_deg_c as u16)) as f32 / 8.0;
        let t1_deg_c = (((((t10_deg_c_msb & 0x0C) >> 2) as u16) << 8) + (t1_deg_c as u16)) as f32 / 8.0;

        let t0_outlsb = i2c.smbus_read_byte_data(Regs::T0OutLsb as u8)?;
        let t0_outmsb = i2c.smbus_read_byte_data(Regs::T0OutMsb as u8)?;

        let t0_out = (((t0_outmsb as u16) << 8) + (t0_outlsb as u16)) as f32 / 8.0;

        let t1_outlsb = i2c.smbus_read_byte_data(Regs::T1OutLsb as u8)?;
        let t1_outmsb = i2c.smbus_read_byte_data(Regs::T1OutMsb as u8)?;

        let t1_out = (((t1_outmsb as u16) << 8) + (t1_outlsb as u16)) as f32 / 8.0;

        if self.odr == OutDataRate::SingleShot {
            let cfg = self.cfg2 | ctrl2_msks::ONE_SHOT_EN;
            i2c.smbus_write_byte_data(Regs::CtrlReg2 as u8, cfg)?;
        }

        loop {
            let status = i2c.smbus_read_byte_data(Regs::Status as u8)?;
            if status & status_msks::TEMP_DRDY != 0 { break; }
        }

        let tout_lsb = i2c.smbus_read_byte_data(Regs::TempOutLo as u8)?;
        let tout_msb = i2c.smbus_read_byte_data(Regs::TempOutHi as u8)?;
        let tout = ((((tout_msb as u16) << 8) + (tout_lsb as u16)) as i16) as f32 / 8.0;

        let a  = (t1_deg_c - t0_deg_c) / (t1_out - t0_out);
        let b  = ((t1_out * t0_deg_c) - (t0_out * t1_deg_c)) / (t1_out - t0_out);
        let t_float = a * tout + b;

        Ok(t_float)
    }

    pub fn get_humidity(&self) -> Result<f32, LinuxI2CError> {
        let mut i2c = LinuxI2CDevice::new("/dev/i2c-1", HUM_I2C_ADDR)?;

        let cfg = self.cfg1 | ctrl1_msks::PWR_UP;
        i2c.smbus_write_byte_data(Regs::CtrlReg1 as u8, cfg)?;

        let h0_out =  (((i2c.smbus_read_byte_data(Regs::H0T0OutMsb as u8)? as u16) << 8) +
                        (i2c.smbus_read_byte_data(Regs::H0T0OutLsb as u8)? as u16     ))   as i16 as f32;
        let h1_out =  (((i2c.smbus_read_byte_data(Regs::H1T0OutMsb as u8)? as u16) << 8) +
                        (i2c.smbus_read_byte_data(Regs::H1T0OutLsb as u8)? as u16     ))   as i16 as f32;

        let h0_rh = i2c.smbus_read_byte_data(Regs::H0rHx2 as u8)? as f32 / 2.0;
        let h1_rh = i2c.smbus_read_byte_data(Regs::H1rhx2 as u8)? as f32 / 2.0;

        let status = i2c.smbus_read_byte_data(Regs::Status as u8)?;
        if status & status_msks::TEMP_DRDY != 0 {
            let _ = i2c.smbus_read_byte_data(Regs::TempOutHi as u8)?;
        }

        if self.odr == OutDataRate::SingleShot {
            let cfg = self.cfg2 | ctrl2_msks::ONE_SHOT_EN;
            i2c.smbus_write_byte_data(Regs::CtrlReg2 as u8, cfg)?;
        }

        loop {
            let status = i2c.smbus_read_byte_data(Regs::Status as u8)?;
            if status & status_msks::HUM_DRDY != 0 { break; }
        }

        let h_out = (((i2c.smbus_read_byte_data(Regs::HumOutHi as u8)? as u16) << 8) +
                      (i2c.smbus_read_byte_data(Regs::HumOutLo as u8)? as u16)     ) as i16 as f32;

        let a = (h1_rh - h0_rh) / (h1_out - h0_out);
        let b = ((h1_out * h0_rh) - (h0_out * h1_rh)) / (h1_out - h0_out);
        let h_float = a * h_out + b;

        Ok(h_float)
    }
}