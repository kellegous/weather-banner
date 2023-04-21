use std::error::Error;
use std::io;

pub struct Station {
    id: String,
    loc: Option<Location>,
    name: Option<String>,
    days: Vec<Day>,
}

impl Station {
    pub fn from_entry<R: io::Read>(entry: &mut tar::Entry<R>) -> Result<Station, Box<dyn Error>> {
        let mut r = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(entry);
        let mut iter = r.records();

        if let Some(record) = iter.next() {
            let record = record?;
            let id = record.get(0).ok_or("missing id")?.to_owned();

            return Ok(Self {
                id,
                loc: None,
                name: None,
                days: Vec::new(),
            });
        }

        return Err("empty entry".into());
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

pub struct Day {
    day: chrono::NaiveDate,
}

pub struct Location {
    lat: f64,
    lng: f64,
}

impl Location {
    pub fn new(lat: f64, lng: f64) -> Self {
        Self { lat, lng }
    }

    pub fn lat(&self) -> f64 {
        self.lat
    }

    pub fn lng(&self) -> f64 {
        self.lng
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (lat_d, lat_m, lat_s) = to_dms(self.lat);
        let (lng_d, lng_m, lng_s) = to_dms(self.lng);
        write!(
            f,
            "{:02}°{:02}′{:02}″{} {:03}°{:02}′{:02}″{}",
            lat_d,
            lat_m,
            lat_s,
            if self.lat < 0.0 { 'S' } else { 'N' },
            lng_d,
            lng_m,
            lng_s,
            if self.lng < 0.0 { 'W' } else { 'E' }
        )
    }
}

impl std::str::FromStr for Location {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = regex::Regex::new(
            r#"(\d+)°(\d+)[′'](\d+)[″"]([NSns]) (\d+)°(\d+)[′'](\d+)[″"]([EWew])"#,
        )?;

        let caps = re.captures(s).ok_or("invalid dms")?;
        let lat_d = caps.get(1).unwrap().as_str().parse::<i32>()?;
        let lat_m = caps.get(2).unwrap().as_str().parse::<i32>()?;
        let lat_s = caps.get(3).unwrap().as_str().parse::<i32>()?;
        let lat_v = lat_d as f64 + (lat_m as f64) / 60.0 + (lat_s as f64) / 3600.0;
        let lat_v = match caps.get(4).ok_or("capture missing")?.as_str() {
            "N" | "n" => lat_v,
            "S" | "s" => -lat_v,
            _ => Err("latitude must be N or S")?,
        };

        let lng_d = caps.get(5).unwrap().as_str().parse::<i32>()?;
        let lng_m = caps.get(6).unwrap().as_str().parse::<i32>()?;
        let lng_s = caps.get(7).unwrap().as_str().parse::<i32>()?;
        let lng_v = lng_d as f64 + (lng_m as f64) / 60.0 + (lng_s as f64) / 3600.0;
        let lng_v = match caps.get(8).unwrap().as_str() {
            "E" | "e" => lng_v,
            "W" | "w" => -lng_v,
            _ => Err("longitude must be E or W")?,
        };

        Ok(Self {
            lat: lat_v,
            lng: lng_v,
        })
    }
}

fn to_dms(v: f64) -> (i32, i32, i32) {
    let v = v.abs();

    let mut d = v as i32;

    let v = v - d as f64;

    let mut m = (v * 60.0) as i32;

    let v = v - m as f64 / 60.0;

    let mut s = (v * 3600.0).round() as i32;

    if s == 60 {
        s = 0;
        m += 1;
    }

    if m == 60 {
        m = 0;
        d += 1;
    }

    (d, m, s)
}

pub fn url_for(year: i32) -> String {
    format!(
        "https://www.ncei.noaa.gov/data/global-summary-of-the-day/archive/{}.tar.gz",
        year
    )
}
