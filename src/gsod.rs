use std::error::Error;
use std::io;

use csv::StringRecord;
use serde::ser::SerializeTuple;
use serde::Serialize;

#[derive(Debug, Serialize)]
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

    pub fn location(&self) -> Option<&Location> {
        self.loc.as_ref()
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn elevation(&self) -> Option<&Elevation> {
        self.elevation.as_ref()
    }

    pub fn days(&self) -> &[Day] {
        &self.days
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

#[derive(Debug, Serialize)]
pub struct Day {
    day: chrono::NaiveDate,
    mean_temperature: Option<MeanTemperature>,
    mean_dewpoint: Option<MeanTemperature>,
    mean_sea_level_pressure: Option<MeanPressure>,
    mean_station_pressure: Option<MeanPressure>,
    mean_visibility: Option<MeanDistance>,
    mean_wind: Option<MeanWindSpeed>,
    max_sustained_wind: Option<WindSpeed>,
    max_wind_gust: Option<WindSpeed>,
    max_temperature: Option<TemperatureExtremity>,
    min_temperature: Option<TemperatureExtremity>,
    precipitation: Option<Precipitation>,
    snow_depth: Option<SnowDepth>,
}

impl Day {
    fn from_record(rec: &StringRecord) -> Result<Day, Box<dyn Error>> {
        let day = chrono::NaiveDate::parse_from_str(from_record(rec, 1)?, "%Y-%m-%d")?;
        let mean_temperature =
            MeanTemperature::from_gsod(from_record(rec, 6)?, from_record(rec, 7)?)?;
        let mean_dewpoint = MeanTemperature::from_gsod(from_record(rec, 8)?, from_record(rec, 9)?)?;
        let mean_sea_level_pressure =
            MeanPressure::from_gsod(from_record(rec, 10)?, from_record(rec, 11)?)?;
        let mean_station_pressure =
            MeanPressure::from_gsod(from_record(rec, 12)?, from_record(rec, 13)?)?;
        let mean_visibility =
            MeanDistance::from_gsod(from_record(rec, 14)?, from_record(rec, 15)?)?;
        let mean_wind = MeanWindSpeed::from_gsod(from_record(rec, 16)?, from_record(rec, 17)?)?;
        let max_sustained_wind = WindSpeed::from_gsod(from_record(rec, 18)?)?;
        let max_wind_gust = WindSpeed::from_gsod(from_record(rec, 19)?)?;
        let max_temperature =
            TemperatureExtremity::from_gsod(from_record(rec, 20)?, from_record(rec, 21)?)?;
        let min_temperature =
            TemperatureExtremity::from_gsod(from_record(rec, 22)?, from_record(rec, 23)?)?;
        let precipitation = Precipitation::from_gsod(from_record(rec, 24)?, from_record(rec, 25)?)?;
        let snow_depth = SnowDepth::from_gsod(from_record(rec, 26)?)?;
        Ok(Self {
            day,
            mean_temperature,
            mean_dewpoint,
            mean_sea_level_pressure,
            mean_station_pressure,
            mean_visibility,
            mean_wind,
            max_sustained_wind,
            max_wind_gust,
            max_temperature,
            min_temperature,
            precipitation,
            snow_depth,
        })
    }

    pub fn date(&self) -> chrono::NaiveDate {
        self.day
    }

    pub fn max_temperature(&self) -> Option<&TemperatureExtremity> {
        self.max_temperature.as_ref()
    }

    pub fn min_temperature(&self) -> Option<&TemperatureExtremity> {
        self.min_temperature.as_ref()
    }

    pub fn mean_temperature(&self) -> Option<&MeanTemperature> {
        self.mean_temperature.as_ref()
    }

    pub fn mean_wind(&self) -> Option<&MeanWindSpeed> {
        self.mean_wind.as_ref()
    }

    pub fn max_sustained_wind(&self) -> Option<&WindSpeed> {
        self.max_sustained_wind.as_ref()
    }

    pub fn precipitation(&self) -> Option<&Precipitation> {
        self.precipitation.as_ref()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PrecipitationAttr {
    SingleOf6HourAmount,
    SummationOf2ReportsOf6HourAmount,
    SummationOf3ReportsOf6HourAmount,
    SummationOf4ReportsOf6HourAmount,
    SingleReportOf12HourAmount,
    SummationOf2ReportsOf12HourAmount,
    SingleReportOf24HourAmount,
    ZeroDespiteHourlyObservations,
    NoReport,
}

impl PrecipitationAttr {
    fn from_gsod(s: &str) -> Result<Option<PrecipitationAttr>, Box<dyn Error>> {
        match s.trim() {
            "" => Ok(None),
            "A" => Ok(Some(PrecipitationAttr::SingleOf6HourAmount)),
            "B" => Ok(Some(PrecipitationAttr::SummationOf2ReportsOf6HourAmount)),
            "C" => Ok(Some(PrecipitationAttr::SummationOf3ReportsOf6HourAmount)),
            "D" => Ok(Some(PrecipitationAttr::SummationOf4ReportsOf6HourAmount)),
            "E" => Ok(Some(PrecipitationAttr::SingleReportOf12HourAmount)),
            "F" => Ok(Some(PrecipitationAttr::SummationOf2ReportsOf12HourAmount)),
            "G" => Ok(Some(PrecipitationAttr::SingleReportOf24HourAmount)),
            "H" => Ok(Some(PrecipitationAttr::ZeroDespiteHourlyObservations)),
            "I" => Ok(Some(PrecipitationAttr::NoReport)),
            s => Err(format!("invalid precipitation attr: {}", s).into()),
        }
    }

    fn to_char(self) -> char {
        match self {
            PrecipitationAttr::SingleOf6HourAmount => 'A',
            PrecipitationAttr::SummationOf2ReportsOf6HourAmount => 'B',
            PrecipitationAttr::SummationOf3ReportsOf6HourAmount => 'C',
            PrecipitationAttr::SummationOf4ReportsOf6HourAmount => 'D',
            PrecipitationAttr::SingleReportOf12HourAmount => 'E',
            PrecipitationAttr::SummationOf2ReportsOf12HourAmount => 'F',
            PrecipitationAttr::SingleReportOf24HourAmount => 'G',
            PrecipitationAttr::ZeroDespiteHourlyObservations => 'H',
            PrecipitationAttr::NoReport => 'I',
        }
    }
}

impl serde::ser::Serialize for PrecipitationAttr {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_char(self.to_char())
    }
}

#[derive(Debug)]
pub struct Precipitation {
    p: f64,
    attr: Option<PrecipitationAttr>,
}

impl Precipitation {
    fn from_gsod(p: &str, a: &str) -> Result<Option<Precipitation>, Box<dyn Error>> {
        let p = match p.trim() {
            "99.99" => return Ok(None),
            p => p.parse::<f64>()?,
        };

        Ok(Some(Precipitation {
            p,
            attr: PrecipitationAttr::from_gsod(a)?,
        }))
    }

    pub fn in_inches(&self) -> f64 {
        self.p
    }

    pub fn attr(&self) -> Option<PrecipitationAttr> {
        self.attr
    }
}

impl serde::ser::Serialize for Precipitation {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = s.serialize_tuple(2)?;
        s.serialize_element(&self.p)?;
        s.serialize_element(&self.attr)?;
        s.end()
    }
}

#[derive(Debug)]
pub struct SnowDepth {
    d: f64,
}

impl SnowDepth {
    fn from_gsod(d: &str) -> Result<Option<SnowDepth>, Box<dyn Error>> {
        match d.trim() {
            "999.9" => Ok(None),
            d => Ok(Some(SnowDepth {
                d: d.parse::<f64>()?,
            })),
        }
    }

    pub fn in_inches(&self) -> f64 {
        self.d
    }
}

impl serde::ser::Serialize for SnowDepth {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_f64(self.d)
    }
}

#[derive(Debug)]
pub enum DeterminedVia {
    ExplicitReading,
    DerivedFromHourly,
}

impl DeterminedVia {
    fn from_gsod(s: &str) -> Result<DeterminedVia, Box<dyn Error>> {
        match s.trim() {
            "*" => Ok(DeterminedVia::DerivedFromHourly),
            "" => Ok(DeterminedVia::ExplicitReading),
            _ => Err(format!("invalid DeterminedVia: {}", s).into()),
        }
    }

    fn to_str(&self) -> &'static str {
        match self {
            DeterminedVia::ExplicitReading => "",
            DeterminedVia::DerivedFromHourly => "*",
        }
    }
}

impl serde::ser::Serialize for DeterminedVia {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(self.to_str())
    }
}

#[derive(Debug)]
pub struct TemperatureExtremity {
    t: Temperature,
    d: DeterminedVia,
}

impl TemperatureExtremity {
    fn new(t: Temperature, d: DeterminedVia) -> TemperatureExtremity {
        TemperatureExtremity { t, d }
    }

    fn from_gsod(t: &str, d: &str) -> Result<Option<TemperatureExtremity>, Box<dyn Error>> {
        match Temperature::from_gsod(t)? {
            Some(t) => Ok(Some(TemperatureExtremity::new(
                t,
                DeterminedVia::from_gsod(d)?,
            ))),
            None => Ok(None),
        }
    }

    pub fn temperature(&self) -> Temperature {
        self.t
    }

    pub fn in_fahrenheit(&self) -> f64 {
        self.t.in_fahrenheit()
    }

    pub fn in_celsius(&self) -> f64 {
        self.t.in_celsius()
    }
}

impl serde::ser::Serialize for TemperatureExtremity {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = s.serialize_tuple(2)?;
        s.serialize_element(&self.t)?;
        s.serialize_element(&self.d)?;
        s.end()
    }
}

#[derive(Debug)]
pub struct MeanWindSpeed {
    s: WindSpeed,
    n: i32,
}

impl MeanWindSpeed {
    fn new(s: WindSpeed, n: i32) -> MeanWindSpeed {
        MeanWindSpeed { s, n }
    }

    fn from_gsod(s: &str, n: &str) -> Result<Option<MeanWindSpeed>, Box<dyn Error>> {
        match WindSpeed::from_gsod(s)? {
            Some(s) => Ok(Some(MeanWindSpeed::new(s, n.trim().parse::<i32>()?))),
            None => Ok(None),
        }
    }

    pub fn in_knots(&self) -> f64 {
        self.s.in_knots()
    }
}

impl serde::ser::Serialize for MeanWindSpeed {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = s.serialize_tuple(2)?;
        s.serialize_element(&self.s)?;
        s.serialize_element(&self.n)?;
        s.end()
    }
}

#[derive(Debug)]
pub struct WindSpeed {
    s: f64,
}

impl WindSpeed {
    fn from_knots(s: f64) -> WindSpeed {
        WindSpeed { s }
    }

    pub fn in_knots(&self) -> f64 {
        self.s
    }

    fn from_gsod(s: &str) -> Result<Option<WindSpeed>, Box<dyn Error>> {
        match s.trim() {
            "999.9" => Ok(None),
            s => Ok(Some(WindSpeed::from_knots(s.parse::<f64>()?))),
        }
    }
}

impl serde::ser::Serialize for WindSpeed {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_f64(self.s)
    }
}

#[derive(Debug)]
pub struct MeanDistance {
    d: Distance,
    n: i32,
}

impl MeanDistance {
    fn new(d: Distance, n: i32) -> MeanDistance {
        MeanDistance { d, n }
    }

    fn from_gsod(d: &str, n: &str) -> Result<Option<MeanDistance>, Box<dyn Error>> {
        match Distance::from_gsod(d)? {
            Some(d) => Ok(Some(MeanDistance::new(d, n.trim().parse::<i32>()?))),
            None => Ok(None),
        }
    }
}

impl serde::ser::Serialize for MeanDistance {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = s.serialize_tuple(2)?;
        s.serialize_element(&self.d)?;
        s.serialize_element(&self.n)?;
        s.end()
    }
}

#[derive(Debug)]
pub struct Distance {
    m: f64,
}

impl Distance {
    fn from_miles(m: f64) -> Distance {
        Distance { m }
    }

    pub fn in_miles(&self) -> f64 {
        self.m
    }

    fn from_gsod(d: &str) -> Result<Option<Distance>, Box<dyn Error>> {
        match d.trim() {
            "999.9" => Ok(None),
            s => Ok(Some(Distance::from_miles(s.parse::<f64>()?))),
        }
    }
}

impl serde::ser::Serialize for Distance {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_f64(self.m)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Pressure {
    p: f64,
}

impl Pressure {
    fn from_millibars(p: f64) -> Self {
        Self { p }
    }

    pub fn in_millibars(&self) -> f64 {
        self.p
    }

    fn from_gsod(s: &str) -> Result<Option<Pressure>, Box<dyn Error>> {
        match s.trim() {
            "9999.9" => Ok(None),
            s => Ok(Some(Pressure::from_millibars(s.parse::<f64>()?))),
        }
    }
}

impl serde::ser::Serialize for Pressure {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_f64(self.p)
    }
}

#[derive(Debug)]
pub struct MeanPressure {
    p: Pressure,
    n: i32,
}

impl MeanPressure {
    fn new(p: Pressure, n: i32) -> Self {
        Self { p, n }
    }

    fn from_gsod(p: &str, n: &str) -> Result<Option<MeanPressure>, Box<dyn Error>> {
        match Pressure::from_gsod(p)? {
            Some(p) => Ok(Some(MeanPressure::new(p, n.trim().parse::<i32>()?))),
            None => Ok(None),
        }
    }
}

impl serde::ser::Serialize for MeanPressure {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = s.serialize_tuple(2)?;
        s.serialize_element(&self.p)?;
        s.serialize_element(&self.n)?;
        s.end()
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

impl serde::ser::Serialize for Temperature {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_f64(self.in_fahrenheit())
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
        self.t.in_fahrenheit()
    }

    pub fn in_celsius(&self) -> f64 {
        self.t.in_celsius()
    }

    pub fn samples(&self) -> i32 {
        self.n
    }

    pub fn temperature(&self) -> Temperature {
        self.t
    }

    fn from_gsod(t: &str, n: &str) -> Result<Option<MeanTemperature>, Box<dyn Error>> {
        if let Some(t) = Temperature::from_gsod(t)? {
            Ok(Some(MeanTemperature::new(t, n.trim().parse::<i32>()?)))
        } else {
            Ok(None)
        }
    }
}

impl serde::ser::Serialize for MeanTemperature {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = s.serialize_tuple(2)?;
        s.serialize_element(&self.t)?;
        s.serialize_element(&self.n)?;
        s.end()
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

impl serde::ser::Serialize for Elevation {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_f64(self.m)
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

impl serde::ser::Serialize for Location {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = s.serialize_tuple(2)?;
        s.serialize_element(&self.lat)?;
        s.serialize_element(&self.lng)?;
        s.end()
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
