use super::{gsod, gsod::Station, Data};
use chrono::prelude::*;
use flate2::read::GzDecoder;
use std::error::Error;
use std::io;
use tar::Archive;

#[derive(clap::Args, Debug)]
pub struct Args {
    #[clap(long, default_value_t = String::from("72309693727"))]
    station_id: String,

    #[clap(long, default_value_t = 1600)]
    width: u32,

    #[clap(long, default_value_t = 600)]
    height: u32,

    #[clap(long, default_value_t = Local::now().year()-1)]
    year: i32,
}

fn find_station<F, R: io::Read>(r: R, f: F) -> Result<Option<Station>, Box<dyn Error>>
where
    F: Fn(&Station) -> bool,
{
    let mut r = Archive::new(GzDecoder::new(r));
    for entry in r.entries()? {
        let station = gsod::Station::from_entry(&mut entry?)?;
        if f(&station) {
            return Ok(Some(station));
        }
    }
    Ok(None)
}

pub fn execute(data: &Data, args: &Args) -> Result<(), Box<dyn Error>> {
    let station = find_station(
        data.download_and_open(&gsod::url_for(args.year), format!("{}.tar.gz", args.year))?,
        |s| s.id() == args.station_id,
    )?
    .ok_or(format!("uknown station: {}", args.station_id))?;
    let json = serde_json::to_string_pretty(&station)?;
    println!("{}", json);
    Ok(())
}
