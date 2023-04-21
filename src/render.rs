use super::gsod;
use super::Data;
use chrono::prelude::*;
use flate2::read::GzDecoder;
use std::error::Error;
use tar::Archive;

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
    let src = data.download_and_open(&gsod::url_for(args.year), format!("{}.tar.gz", args.year))?;
    let mut r = Archive::new(GzDecoder::new(src));
    for entry in r.entries()? {
        let station = gsod::Station::from_entry(&mut entry?)?;
        println!("{}", station.id());
        // println!("{:?}", station);
    }
    Ok(())
}
