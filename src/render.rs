use super::Data;
use chrono::prelude::*;
use std::error::Error;

#[derive(clap::Args, Debug)]
pub struct Args {
    #[clap(long, default_value_t = String::from(""))]
    station_id: String,

    #[clap(long, default_value_t = 1600)]
    width: u32,

    #[clap(long, default_value_t = 600)]
    height: u32,

    #[clap(long, default_value_t = Local::now().year()-1)]
    year: i32,
}

pub fn execute(data: &Data, args: &Args) -> Result<(), Box<dyn Error>> {
    println!("{:#?}", args);
    println!("{:#?}", data);
    Ok(())
}
