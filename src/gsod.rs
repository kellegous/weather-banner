use std::error::Error;

pub struct Station {
    id: String,
    name: Option<String>,
    days: Vec<Day>,
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
