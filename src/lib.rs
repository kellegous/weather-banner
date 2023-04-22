use cairo::{Context, FontSlant, FontWeight};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::f64::consts::PI;
use std::fs;
use std::path::{Path, PathBuf};

pub mod gsod;
pub mod list_stations;
pub mod render;
pub mod time;

pub const TAU: f64 = 2.0 * PI;

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

pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 0xff }
    }

    pub fn from_u32(c: u32) -> Self {
        Self {
            r: (c >> 16) as u8,
            g: (c >> 8) as u8,
            b: c as u8,
            a: 0xff,
        }
    }

    pub fn from_u32_with_alpha(c: u32, a: f64) -> Self {
        Self {
            r: (c >> 16) as u8,
            g: (c >> 8) as u8,
            b: c as u8,
            a: (a * 255.0) as u8,
        }
    }

    pub fn set(&self, ctx: &Context) {
        let r = self.r as f64 / 255.0;
        let g = self.g as f64 / 255.0;
        let b = self.b as f64 / 255.0;
        if self.a == 0xff {
            ctx.set_source_rgb(r, g, b);
        } else {
            ctx.set_source_rgba(r, g, b, self.a as f64 / 255.0)
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Unit {
    v: f64,
}

impl Unit {
    pub fn new(v: f64) -> Unit {
        Unit { v }
    }

    pub fn zero() -> Unit {
        Unit { v: 0.0 }
    }

    pub fn value(&self) -> f64 {
        self.v
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Range {
    min: f64,
    max: f64,
}

impl Range {
    pub fn new(min: f64, max: f64) -> Range {
        Range { min, max }
    }

    pub fn min(&self) -> f64 {
        self.min
    }

    pub fn max(&self) -> f64 {
        self.max
    }

    pub fn normalize(&self, v: f64) -> Unit {
        let rng = self.max - self.min;
        Unit::new((v - self.min) / rng)
    }

    pub fn project(&self, u: Unit) -> f64 {
        let rng = self.max - self.min;
        self.min + u.value() * rng
    }

    pub fn intersect(a: &Range, b: &Range) -> Range {
        Range {
            min: a.min.min(b.min),
            max: a.max.max(b.max),
        }
    }
}

#[derive(Debug)]
pub struct Font {
    family: &'static str,
    slant: FontSlant,
    weight: FontWeight,
    size: f64,
}

impl Font {
    pub fn new(family: &'static str, slant: FontSlant, weight: FontWeight, size: f64) -> Font {
        Font {
            family,
            slant,
            weight,
            size,
        }
    }

    pub fn set(&self, ctx: &Context) {
        ctx.select_font_face(self.family, self.slant, self.weight);
        ctx.set_font_size(self.size);
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Series {
    vals: Vec<f64>,
    rng: Range,
    min_index: isize,
    max_index: isize,
}

impl Series {
    pub fn from_iterator<I>(iter: I) -> Series
    where
        I: Iterator<Item = Option<f64>>,
    {
        let mut vals = Vec::new();
        let mut prev = 0.0;
        let mut max = f64::MIN;
        let mut min = f64::MAX;
        let mut max_index = 0;
        let mut min_index = 0;
        for (i, item) in iter.enumerate() {
            match item {
                Some(val) => {
                    if val > max {
                        max = val;
                        max_index = i;
                    }
                    if val < min {
                        min = val;
                        min_index = i;
                    }
                    vals.push(val);
                    prev = val;
                }
                None => vals.push(prev),
            }
        }

        Series {
            vals,
            rng: Range::new(min, max),
            min_index: min_index as isize,
            max_index: max_index as isize,
        }
    }

    pub fn for_each_day<'a, I, F>(year: time::Year, days: I, f: F) -> Series
    where
        I: Iterator<Item = &'a gsod::Day>,
        F: Fn(&gsod::Day) -> Option<f64>,
    {
        let mut idx = HashMap::new();
        for day in days {
            idx.insert(day.date().ordinal(), day);
        }

        Series::from_iterator(year.days().map(|day| match idx.get(&day.ordinal()) {
            Some(day) => f(day),
            None => None,
        }))
    }

    pub fn with_range(self, rng: &Range) -> Series {
        Series {
            vals: self.vals,
            rng: rng.clone(),
            min_index: self.min_index,
            max_index: self.max_index,
        }
    }

    pub fn normalize(&self) -> impl Iterator<Item = Unit> + '_ {
        self.vals.iter().map(move |v| self.rng.normalize(*v))
    }

    pub fn values(&self) -> &[f64] {
        &self.vals
    }

    pub fn range(&self) -> &Range {
        &self.rng
    }

    pub fn get(&self, i: isize) -> f64 {
        let n = self.vals.len() as isize;
        self.vals[(((i % n) + n) % n) as usize]
    }

    pub fn get_normalized(&self, i: isize) -> Unit {
        self.rng.normalize(self.get(i))
    }

    pub fn min_index(&self) -> isize {
        self.min_index
    }

    pub fn max_index(&self) -> isize {
        self.max_index
    }

    pub fn downsample_by<F>(&self, n: usize, agg: F) -> Series
    where
        F: Fn(&[f64]) -> f64,
    {
        let m = self.vals.len() / n;
        let mut vals = Vec::with_capacity(m);

        for i in 0..m {
            let j = i * n;
            let v = agg(&self.vals[j..(j + n)]);
            vals.push(v);
        }

        Series {
            vals,
            rng: self.rng.clone(),
            min_index: self.min_index / n as isize,
            max_index: self.max_index / n as isize,
        }
    }
}

#[derive(Debug)]
pub struct Scale {
    step: f64,
    steps: Vec<f64>,
}

impl Scale {
    pub fn from_range(r: &Range, lim: f64) -> Scale {
        let rng = r.max() - r.min();
        let mag = (10.0f64).powf((rng.log10() - 1.0).floor());
        let facs = vec![1, 2, 3, 5, 10, 20, 30, 50];
        for fac in facs {
            let step = fac as f64 * mag;
            let n = rng / step;
            if n < lim {
                return Self::from_range_with_step(r, step);
            }
        }

        panic!("unreachable");
    }

    pub fn from_range_with_step(r: &Range, step: f64) -> Scale {
        let mut min = (r.min() / step).floor() * step + step;
        let max = r.max();
        let mut steps = Vec::new();
        while min < max {
            steps.push(min);
            min += step;
        }
        Scale { step, steps }
    }

    pub fn label_for(&self, i: usize) -> String {
        let s = self.steps[i];
        if self.step() >= 1.0 {
            format!("{}", s as i32)
        } else {
            let p = s.log10().floor().abs() as usize;
            println!("step = {}, s = {}, p = {}", self.step(), s, p);
            format!("{0:.1$}", s, p)
        }
    }

    pub fn steps(&self) -> &[f64] {
        &self.steps
    }

    pub fn step(&self) -> f64 {
        self.step
    }
}

pub enum Direction {
    Right,
    Left,
}
