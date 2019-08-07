mod sensors;
mod hmi;
mod app;

use app::{
    SensorsSet,
    SensorValues,
    CliOpts,
};

use std::thread;
use std::time::Duration;
use structopt::StructOpt;

fn main() -> Result<(), std::io::Error> {
    let opts = CliOpts::from_args();
    let mut ss = SensorsSet::new()?;
    let mut logger = app::SnsrValLogger::new(&opts.output);

    loop {
        let snsr_val = ss.get_sensors_values()?;
        logger.wr_snsr_vals(&snsr_val);
        thread::sleep(Duration::new(opts.period, 0));
    }
    
    Ok(())
}

