use std::error::Error;
use std::io;

use csv::StringRecord;

#[derive(Debug)]
pub struct Station {
    id: String,
    name: Option<String>,
    loc: Option<Location>,
    elevation: Option<Elevation>,
    days: Vec<Day>,
}

impl Station {
    pub fn from_entry<R: io::Read>(entry: &mut tar::Entry<R>) -> Result<Station, Box<dyn Error>> {
        let mut r = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(entry);
        let mut iter = r.records();
        let mut days = Vec::new();
        if let Some(record) = iter.next() {
            let record = record?;
            let id = from_record(&record, 0)?.to_owned();
            let loc = parse_location(from_record(&record, 2)?, from_record(&record, 3)?)?;
            let name = from_record(&record, 5)?;
            let name = if name.is_empty() {
                None
            } else {
                Some(name.to_owned())
            };
            let elevation = Elevation::from_gsod(from_record(&record, 4)?)?;

            days.push(Day::from_record(&record)?);
            for record in iter {
                days.push(Day::from_record(&record?)?);
            }

            return Ok(Self {
                id,
                name,
                loc,
                elevation,
                days,
            });
        }

        Err("empty entry".into())
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

fn from_record(rec: &StringRecord, ix: usize) -> Result<&str, Box<dyn Error>> {
    rec.get(ix)
        .ok_or_else(|| format!("missing field {}", ix).into())
}

fn parse_location(lat: &str, lng: &str) -> Result<Option<Location>, Box<dyn Error>> {
    if lat.is_empty() || lng.is_empty() {
        return Ok(None);
    }

    Ok(Some(Location::new(
        lat.parse::<f64>()?,
        lng.parse::<f64>()?,
    )))
}

#[derive(Debug)]
pub struct Day {
    day: chrono::NaiveDate,
    mean_temperature: Option<MeanTemperature>,
    mean_dewpoint: Option<MeanTemperature>,
}

impl Day {
    fn from_record(rec: &StringRecord) -> Result<Day, Box<dyn Error>> {
        let day = chrono::NaiveDate::parse_from_str(from_record(rec, 1)?, "%Y-%m-%d")?;
        Ok(Self {
            day,
            mean_temperature: None,
            mean_dewpoint: None,
        })
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Temperature {
    f: f64,
}

impl Temperature {
    fn from_fahrenheit(f: f64) -> Self {
        Self { f }
    }

    pub fn in_fahrenheit(&self) -> f64 {
        self.f
    }

    pub fn in_celsius(&self) -> f64 {
        (self.f - 32.0) * 5.0 / 9.0
    }

    fn from_gsod(s: &str) -> Result<Option<Self>, Box<dyn Error>> {
        match s.trim() {
            "9999.9" => Ok(None),
            s => Ok(Some(Temperature::from_fahrenheit(s.parse::<f64>()?))),
        }
    }
}

#[derive(Debug)]
pub struct MeanTemperature {
    t: Temperature,
    n: i32,
}

impl MeanTemperature {
    fn new(t: Temperature, n: i32) -> Self {
        Self { t, n }
    }

    pub fn in_fahrenheit(&self) -> f64 {
        self.t.in_fahrenheit() / self.n as f64
    }

    pub fn in_celsius(&self) -> f64 {
        self.t.in_celsius() / self.n as f64
    }

    pub fn samples(&self) -> i32 {
        self.n
    }

    pub fn temperature(&self) -> Temperature {
        self.t
    }
}

#[derive(Debug)]
pub struct Elevation {
    m: f64,
}

impl Elevation {
    fn new(m: f64) -> Self {
        Self { m }
    }

    pub fn in_meters(&self) -> f64 {
        self.m
    }

    fn from_gsod(s: &str) -> Result<Option<Self>, Box<dyn Error>> {
        match s.trim() {
            "" => Ok(None),
            m => Ok(Some(Self::new(m.parse::<f64>()?))),
        }
    }
}

#[derive(Debug)]
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
