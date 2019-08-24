use std::error::Error;
use std::fmt;
use std::io;

pub struct Ch1;
pub struct Ch2;
pub struct Ch3;
pub struct Ch4;

pub struct AdcErr(String);

impl AdcErr {
    pub fn new(msg : &str) -> AdcErr {
        AdcErr(msg.to_owned())
    }
}

impl Error for AdcErr {
    fn description(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AdcErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for AdcErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<io::Error> for AdcErr {
    fn from(error : io::Error) -> Self {
        AdcErr(error.description().to_owned())
    }
}

impl From<AdcErr> for io::Error {
    fn from(error : AdcErr) -> Self {
        io::Error::new(io::ErrorKind::Other, error)
    }
}

pub trait AdcCh {
    fn get_voltage(&self) -> Result<f32, AdcErr>;
    fn get_ref_voltage(&self) -> f32;
}