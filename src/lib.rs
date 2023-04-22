use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

pub mod gsod;
pub mod list_stations;
pub mod render;

#[derive(Debug)]
pub struct Data {
    dir: PathBuf,
}

impl Data {
    pub fn from<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let path = path.as_ref();
        if !path.exists() {
            fs::create_dir_all(path)?;
        }

        Ok(Self {
            dir: path.to_owned(),
        })
    }

    pub fn download_and_open<P: AsRef<Path>>(
        &self,
        url: &str,
        dst: P,
    ) -> Result<fs::File, Box<dyn Error>> {
        let dst = self.dir.join(dst);
        if !dst.exists() {
            reqwest::blocking::get(url)?.copy_to(&mut fs::File::create(&dst)?)?;
        }
        Ok(fs::File::open(&dst)?)
    }
}
