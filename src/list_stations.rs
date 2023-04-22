use super::{gsod, Data};
use chrono::prelude::*;
use flate2::read::GzDecoder;
use std::error::Error;
use tar::Archive;

#[derive(clap::Args, Debug)]
pub struct Args {
    #[clap(long, default_value_t = Local::now().year()-1)]
    year: i32,
}

pub fn execute(data: &Data, args: &Args) -> Result<(), Box<dyn Error>> {
    let mut r = Archive::new(GzDecoder::new(
        data.download_and_open(&gsod::url_for(args.year), format!("{}.tar.gz", args.year))?,
    ));
    for entry in r.entries()? {
        let station = gsod::Station::from_entry(&mut entry?)?;
        let json = serde_json::to_string_pretty(&station)?;
        println!("{}", json);
    }
    Ok(())
}
