use super::{
    gsod, gsod::Station, time, Color, Data, Direction, Font, Range, Scale, Series, Unit, TAU,
};
use cairo::{Context, FontSlant, FontWeight, Format, ImageSurface};
use chrono::prelude::*;
use flate2::read::GzDecoder;
use std::error::Error;
use std::f64::consts::PI;
use std::fs;
use std::io;
use tar::Archive;

#[derive(clap::Args, Debug)]
pub struct Args {
    #[clap(long, default_value_t = String::from("72309693727"))]
    station_id: String,

    #[clap(long, default_value_t = 1600)]
    width: i32,

    #[clap(long, default_value_t = 600)]
    height: i32,

    #[clap(long, default_value_t = Local::now().year()-1)]
    year: i32,

    #[clap(long, default_value_t = String::from(""))]
    destination: String,

    #[clap(long, default_value_t = false)]
    debug: bool,

    #[clap(long, default_value_t = 2)]
    downsample_by: u32,

    #[clap(long, default_value_t = true)]
    smooth: bool,
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

    let surface = ImageSurface::create(Format::ARgb32, args.width, args.height)?;
    let ctx = Context::new(&surface)?;
    render(
        &ctx,
        args.width as f64,
        args.height as f64,
        time::Year::from_ordinal(args.year),
        &station,
        &Options {
            debug: args.debug,
            downsample_by: args.downsample_by,
            smooth: args.smooth,
        },
    )?;

    let dst = if args.destination.is_empty() {
        format!("{}.png", args.station_id)
    } else {
        args.destination.clone()
    };
    surface.write_to_png(&mut fs::File::create(&dst)?)?;
    println!("{}", &dst);
    Ok(())
}

struct Options {
    debug: bool,
    downsample_by: u32,
    smooth: bool,
}

fn render(
    ctx: &Context,
    width: f64,
    height: f64,
    year: time::Year,
    station: &Station,
    opts: &Options,
) -> Result<(), Box<dyn Error>> {
    Color::from_u32(0x3b3938).set(ctx);
    ctx.rectangle(0.0, 0.0, width, height);
    ctx.fill()?;

    let lx = width / 6.0;
    let rx = 5.0 * width / 6.0;
    let cx = width / 2.0;

    if opts.debug {
        let n = 3.0;
        let dx = width / n;
        ctx.save()?;
        Color::from_u32_with_alpha(0xffffff, 0.2).set(ctx);
        for i in 0..n as usize {
            if i % 2 != 0 {
                continue;
            }

            ctx.new_path();
            let x = dx * i as f64;
            ctx.rectangle(x, 0.0, dx, height);
            ctx.fill()?;
        }
        ctx.restore()?;
    }

    ctx.save()?;
    let header_height = render_header(ctx, station, year, width, opts)?;
    ctx.restore()?;

    let body_height = height - header_height;

    if opts.debug {
        ctx.save()?;
        Color::from_u32_with_alpha(0xffffff, 0.2).set(ctx);
        ctx.new_path();
        ctx.rectangle(0.0, 0.0, width, header_height);
        ctx.fill()?;
        ctx.restore()?;
    }

    let r = lx.min(body_height / 2.0);
    let rrange = Range::new(r * 0.6, r * 0.9);

    ctx.save()?;
    ctx.translate(lx, header_height + body_height / 2.0);
    render_title(ctx, "TEMPERATURE", 0.0, -rrange.max() - 10.0)?;
    render_temperature(ctx, year, station, &rrange, opts)?;
    ctx.restore()?;

    ctx.save()?;
    ctx.translate(cx, header_height + body_height / 2.0);
    render_title(ctx, "WIND", 0.0, -rrange.max() - 10.0)?;
    render_wind(ctx, year, station, &rrange, opts)?;
    ctx.restore()?;

    ctx.save()?;
    ctx.translate(rx, header_height + body_height / 2.0);
    render_title(ctx, "PRECIPITATION", 0.0, -rrange.max() - 10.0)?;
    render_precipitation(ctx, year, station, &rrange, opts)?;
    ctx.restore()?;

    Ok(())
}

fn render_header(
    ctx: &Context,
    station: &gsod::Station,
    year: time::Year,
    width: f64,
    opts: &Options,
) -> Result<f64, Box<dyn Error>> {
    let xoff = 20.0;
    let yoff = 20.0;

    Color::from_u32_with_alpha(0xffffff, 0.9).set(ctx);

    let title = shorten_station_name(station.name().unwrap_or("UNKNOWN"));
    ctx.select_font_face("HelveticaNeue-Thin", FontSlant::Normal, FontWeight::Normal);
    ctx.set_font_size(42.0);
    let title_exts = ctx.text_extents(&title)?;
    ctx.new_path();
    ctx.move_to(xoff, yoff - title_exts.y_bearing());
    ctx.show_text(&title)?;

    let time_desc = describe_year(year);
    ctx.select_font_face("HelveticaNeue", FontSlant::Normal, FontWeight::Normal);
    ctx.set_font_size(24.0);
    let time_desc_exts = ctx.text_extents(&time_desc)?;
    ctx.new_path();
    ctx.move_to(
        width - yoff - time_desc_exts.width(),
        yoff - title_exts.y_bearing(),
    );
    ctx.show_text(&time_desc)?;

    let details = describe_station_details(station);
    ctx.select_font_face("HelveticaNeue", FontSlant::Normal, FontWeight::Normal);
    ctx.set_font_size(16.0);
    let details_exts = ctx.text_extents(&details)?;
    ctx.new_path();
    ctx.move_to(
        xoff,
        yoff + title_exts.height() * 1.3 - details_exts.y_bearing(),
    );
    ctx.show_text(&details)?;

    if opts.debug {
        Color::from_u32(0xff9900).set(ctx);
        ctx.set_line_width(1.0);
        let y = yoff - title_exts.y_bearing();
        ctx.move_to(0.0, y);
        ctx.line_to(width, y);
        ctx.stroke()?;

        let y = yoff + title_exts.height() * 1.3 - details_exts.y_bearing();
        ctx.move_to(0.0, y);
        ctx.line_to(width, y);
        ctx.stroke()?;
    }

    Ok(2.0 * yoff + title_exts.height() * 1.3 + details_exts.height())
}

fn render_title(ctx: &Context, title: &str, x: f64, y: f64) -> Result<(), Box<dyn Error>> {
    ctx.save()?;
    let font = Font::new(
        "HelveticaNeue-Medium",
        FontSlant::Normal,
        FontWeight::Normal,
        12.0,
    );
    font.set(ctx);
    Color::from_u32_with_alpha(0xffffff, 0.6).set(ctx);
    let exts = ctx.text_extents(title)?;
    ctx.new_path();
    ctx.move_to(x - exts.width() / 2.0, y);
    ctx.show_text(title)?;
    ctx.restore()?;
    Ok(())
}

fn render_temperature(
    ctx: &Context,
    year: time::Year,
    station: &gsod::Station,
    rrange: &Range,
    opts: &Options,
) -> Result<(), Box<dyn Error>> {
    let min_temps = Series::for_each_day(year, station.days().iter(), |day| {
        day.min_temperature().map(|t| t.in_fahrenheit())
    });

    let max_temps = Series::for_each_day(year, station.days().iter(), |day| {
        day.max_temperature().map(|t| t.in_fahrenheit())
    });

    let mean_temps = Series::for_each_day(year, station.days().iter(), |day| {
        day.mean_temperature().map(|t| t.in_fahrenheit())
    });

    let range = Range::intersect(max_temps.range(), min_temps.range());

    let min_temps = min_temps.with_range(&range);
    let max_temps = max_temps.with_range(&range);
    let mean_temps = mean_temps.with_range(&range);

    let avg_mean_temp = mean_temps.values().iter().fold(0.0, |sum, val| sum + val)
        / mean_temps.values().len() as f64;

    let min_temps = if opts.downsample_by > 1 {
        min_temps.downsample_by(opts.downsample_by as usize, |vals| {
            vals.iter().fold(f64::MAX, |min, val| min.min(*val))
        })
    } else {
        min_temps
    };

    let max_temps = if opts.downsample_by > 1 {
        max_temps.downsample_by(opts.downsample_by as usize, |vals| {
            vals.iter().fold(f64::MIN, |max, val| max.max(*val))
        })
    } else {
        max_temps
    };

    let mean_temps = if opts.downsample_by > 1 {
        mean_temps.downsample_by(opts.downsample_by as usize, |vals| {
            vals.iter().fold(0.0, |sum, val| sum + val) / vals.len() as f64
        })
    } else {
        mean_temps
    };

    let range = min_temps.range();

    // let's draw the months
    ctx.save()?;
    render_months(
        ctx,
        year,
        &Range::new(rrange.min() - 40.0, rrange.min() - 5.0),
    )?;
    ctx.restore()?;

    // let's draw the scales
    ctx.save()?;
    let scale = Scale::from_range(range, 5.0);
    render_scales(ctx, &scale, range, rrange, "°F", Direction::Left)?;
    ctx.restore()?;

    // temperature range
    ctx.save()?;
    render_radial_range(
        ctx,
        &min_temps,
        &max_temps,
        rrange,
        Some(&Color::from_u32_with_alpha(0x6eb078, 0.1)),
        Some(&Color::from_u32(0x6eb078)),
        opts.smooth,
    )?;
    ctx.restore()?;

    ctx.save()?;
    render_radial_series(
        ctx,
        &mean_temps,
        rrange,
        &Color::from_u32(0xe45f91),
        opts.smooth,
    )?;
    ctx.restore()?;

    ctx.save()?;
    render_center_text(
        ctx,
        &[
            (String::from("MAX"), format!("{:.1}°F", range.max())),
            (String::from("AVG"), format!("{:.1}°F", avg_mean_temp)),
            (String::from("MIN"), format!("{:.1}°F", range.min())),
        ],
        &Font::new(
            "HelveticaNeue-Medium",
            FontSlant::Normal,
            FontWeight::Bold,
            11.0,
        ),
        &Font::new(
            "HelveticaNeue-Thin",
            FontSlant::Normal,
            FontWeight::Normal,
            32.0,
        ),
        &Color::from_u32_with_alpha(0xffffff, 0.6),
        opts,
    )?;
    ctx.restore()?;

    Ok(())
}

fn render_center_text(
    ctx: &Context,
    labels: &[(String, String)],
    label_font: &Font,
    value_font: &Font,
    color: &Color,
    opts: &Options,
) -> Result<(), Box<dyn Error>> {
    ctx.save()?;
    color.set(ctx);

    let (key, val) = labels.first().unwrap();
    value_font.set(ctx);
    let first_val_ext = ctx.text_extents(val)?;

    label_font.set(ctx);
    let first_key_ext = ctx.text_extents(key)?;

    value_font.set(ctx);
    let mut width = first_val_ext.width();
    for (_, val) in &labels[1..] {
        let ext = ctx.text_extents(val)?;
        if ext.width() > width {
            width = ext.width();
        }
    }

    let spacing = 2.3;
    let offset = first_key_ext.height();
    let height = offset + first_val_ext.height() * 2.0 * spacing - first_val_ext.y_bearing();

    let lx = -width / 2.0;
    let y = -height / 2.0;

    value_font.set(ctx);
    for (i, (_, val)) in labels.iter().enumerate() {
        ctx.new_path();
        ctx.move_to(
            lx,
            y + offset - first_val_ext.y_bearing() + spacing * first_val_ext.height() * i as f64,
        );
        ctx.show_text(val)?;
    }

    label_font.set(ctx);
    for (i, (key, _)) in labels.iter().enumerate() {
        ctx.new_path();
        ctx.move_to(
            lx,
            y + offset + spacing * first_val_ext.height() * i as f64 - 5.0,
        );
        ctx.show_text(key)?
    }

    if opts.debug {
        Color::from_u32_with_alpha(0xff9900, 0.1).set(ctx);
        ctx.new_path();
        ctx.rectangle(-width / 2.0, -height / 2.0, width, height);
        ctx.fill()?;
    }

    ctx.restore()?;
    Ok(())
}

fn render_months(ctx: &Context, year: time::Year, r: &Range) -> Result<(), Box<dyn Error>> {
    let num_days = year.duration().num_days();
    let months: Vec<(f64, f64)> = year
        .months()
        .map(|month| {
            let s = month.start().signed_duration_since(year.start()).num_days();
            let e = month.end().signed_duration_since(year.start()).num_days();
            (s as f64 / num_days as f64, e as f64 / num_days as f64)
        })
        .collect();

    let dt = 0.5 * TAU / num_days as f64;

    Color::from_u32_with_alpha(0xffffff, 0.05).set(ctx);
    for (s, e) in months.iter() {
        let s = s * TAU + dt;
        let e = e * TAU - dt;
        ctx.new_path();
        ctx.arc(0.0, 0.0, r.max(), s, e);
        ctx.arc_negative(0.0, 0.0, r.min(), e, s);
        ctx.fill()?;
    }

    Color::from_u32(0xffffff).set(ctx);
    ctx.select_font_face("HelveticaNeue", FontSlant::Normal, FontWeight::Normal);
    ctx.set_font_size(10.0);
    for (i, month) in year.months().enumerate() {
        let (s, e) = months[i];
        let y = (r.max() + r.min()) / 2.0;
        ctx.save()?;
        ctx.rotate((s + (e - s) / 2.0) * TAU);
        let name = format!("{}", month.start().format("%b"));
        let exts = ctx.text_extents(&name)?;
        ctx.move_to(-exts.width() / 2.0, -y + exts.height() / 2.0);
        ctx.show_text(&name)?;
        ctx.restore()?;
    }

    Ok(())
}

fn render_scales(
    ctx: &Context,
    scale: &Scale,
    trange: &Range,
    rrange: &Range,
    units: &str,
    dir: Direction,
) -> Result<(), Box<dyn Error>> {
    let tb = TAU * 0.75;

    // this is the y value of the inner most scale ring
    // let y = -rrange.project(trange.normalize(*steps.first().unwrap() as f64)) + 10.0;
    let y = -rrange.project(trange.normalize(*scale.steps().first().unwrap())) + 10.0;

    ctx.set_dash(&[1.0, 4.0], 0.0);
    Color::from_u32_with_alpha(0xffffff, 0.6).set(ctx);
    ctx.select_font_face("HelveticaNeue", FontSlant::Normal, FontWeight::Normal);
    ctx.set_font_size(10.0);
    if let Direction::Right = dir {
        for (i, step) in scale.steps().iter().enumerate() {
            let r = rrange.project(trange.normalize(*step));

            let ta = (y / r).asin();
            ctx.save()?;
            ctx.new_path();
            ctx.arc(0.0, 0.0, r, ta, tb);
            ctx.line_to(r * tb.cos() + rrange.max(), r * tb.sin());
            ctx.stroke()?;
            ctx.restore()?;

            ctx.save()?;
            let label = format!("{}{}", scale.label_for(i), units);
            let exts = ctx.text_extents(&label)?;
            ctx.move_to(
                r * tb.cos() + rrange.max() + 5.0,
                r * tb.sin() + exts.height() / 2.0,
            );
            ctx.show_text(&label)?;
            ctx.restore()?;
        }
    } else {
        for (i, step) in scale.steps().iter().enumerate() {
            let r = rrange.project(trange.normalize(*step));
            let ta = PI - (y / r).asin();
            let x = r * tb.cos();
            let y = r * tb.sin();
            ctx.save()?;
            ctx.new_path();
            ctx.arc_negative(0.0, 0.0, r, ta, tb);
            ctx.line_to(x - rrange.max(), y);
            ctx.stroke()?;
            ctx.restore()?;

            ctx.save()?;
            let label = format!("{}{}", scale.label_for(i), units);
            let exts = ctx.text_extents(&label)?;
            ctx.move_to(
                x - rrange.max() - exts.x_advance() - 5.0,
                y + exts.height() / 2.0,
            );
            ctx.show_text(&label)?;
            ctx.restore()?;
        }
    }

    Ok(())
}

pub fn render_radial_range(
    ctx: &Context,
    min: &Series,
    max: &Series,
    rrange: &Range,
    fill_color: Option<&Color>,
    stroke_color: Option<&Color>,
    smooth: bool,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(max.values().len(), min.values().len());
    let n = max.values().len();
    let dt = TAU / n as f64;
    let t0 = -TAU / 4.0;
    let t4 = TAU / 4.0;

    ctx.new_path();
    let r = rrange.project(max.get_normalized(0));
    ctx.move_to(r * t0.cos(), r * t0.sin());

    for i in 1..=n {
        let ta = i as f64 * dt - dt + t0;
        let tb = i as f64 * dt + t0;
        let ra = rrange.project(max.get_normalized(i as isize - 1));
        let rb = rrange.project(max.get_normalized(i as isize));
        let xa = ra * ta.cos();
        let ya = ra * ta.sin();
        let xb = rb * tb.cos();
        let yb = rb * tb.sin();
        let da = distance_across_arc(ra, dt) * 0.55;
        let db = distance_across_arc(rb, dt) * 0.55;
        if smooth {
            let ca = ta + t4;
            let cb = tb - t4;
            ctx.curve_to(
                xa + da * ca.cos(),
                ya + da * ca.sin(),
                xb + db * cb.cos(),
                yb + db * cb.sin(),
                xb,
                yb,
            );
        } else {
            ctx.line_to(xb, yb);
        }
    }

    let r = rrange.project(min.get_normalized(n as isize - 1));
    let t = (n as f64 - 1.0) * dt + t0;
    ctx.move_to(r * t.cos(), r * t.sin());

    for i in 0..=n {
        let i = n as isize - i as isize - 1;
        let ta = i as f64 * dt + t0;
        let tb = i as f64 * dt - dt + t0;
        let ra = rrange.project(min.get_normalized(i));
        let rb = rrange.project(min.get_normalized(i - 1));
        let xa = ra * ta.cos();
        let ya = ra * ta.sin();
        let xb = rb * tb.cos();
        let yb = rb * tb.sin();
        let da = distance_across_arc(ra, dt) * 0.55;
        let db = distance_across_arc(rb, dt) * 0.55;
        if smooth {
            let ca = ta - t4;
            let cb = tb + t4;
            ctx.curve_to(
                xa + da * ca.cos(),
                ya + da * ca.sin(),
                xb + db * cb.cos(),
                yb + db * cb.sin(),
                xb,
                yb,
            );
        } else {
            ctx.line_to(xb, yb);
        }
    }

    if let Some(fill_color) = fill_color {
        fill_color.set(ctx);
        ctx.fill_preserve()?;
    }

    if let Some(stroke_color) = stroke_color {
        stroke_color.set(ctx);
        ctx.stroke()?;
    }

    Ok(())
}

pub fn render_radial_series(
    ctx: &Context,
    series: &Series,
    rrange: &Range,
    color: &Color,
    smooth: bool,
) -> Result<(), Box<dyn Error>> {
    let n = series.values().len();
    let dt = TAU / n as f64;
    let t0 = -TAU / 4.0;
    let t4 = TAU / 4.0;

    ctx.new_path();
    let r = rrange.project(series.get_normalized(0));
    ctx.move_to(r * t0.cos(), r * t0.sin());

    for i in 1..=n {
        let ta = i as f64 * dt - dt + t0;
        let tb = i as f64 * dt + t0;
        let ra = rrange.project(series.get_normalized(i as isize - 1));
        let rb = rrange.project(series.get_normalized(i as isize));
        let xa = ra * ta.cos();
        let ya = ra * ta.sin();
        let xb = rb * tb.cos();
        let yb = rb * tb.sin();
        let da = distance_across_arc(ra, dt) * 0.55;
        let db = distance_across_arc(rb, dt) * 0.55;
        if smooth {
            let ca = ta + t4;
            let cb = tb - t4;
            ctx.curve_to(
                xa + da * ca.cos(),
                ya + da * ca.sin(),
                xb + db * cb.cos(),
                yb + db * cb.sin(),
                xb,
                yb,
            );
        } else {
            ctx.line_to(xb, yb);
        }
    }

    color.set(ctx);
    ctx.stroke()?;

    Ok(())
}

fn render_wind(
    ctx: &Context,
    year: time::Year,
    station: &gsod::Station,
    rrange: &Range,
    opts: &Options,
) -> Result<(), Box<dyn Error>> {
    let mean_wind = Series::for_each_day(year, station.days().iter(), |day| {
        day.mean_wind().map(|s| s.in_knots())
    });

    let max_sustained_wind = Series::for_each_day(year, station.days().iter(), |day| {
        day.max_sustained_wind().map(|s| s.in_knots())
    });

    let range = Range::intersect(mean_wind.range(), max_sustained_wind.range());

    let mean_wind = mean_wind.with_range(&range);
    let max_sustained_wind = max_sustained_wind.with_range(&range);

    let avg_mean_wind =
        mean_wind.values().iter().fold(0.0, |sum, val| sum + val) / mean_wind.values().len() as f64;

    let mean_wind = if opts.downsample_by > 1 {
        mean_wind.downsample_by(opts.downsample_by as usize, |vals| {
            vals.iter().fold(0.0, |sum, val| sum + val) / vals.len() as f64
        })
    } else {
        mean_wind
    };

    let max_sustained_wind = if opts.downsample_by > 1 {
        max_sustained_wind.downsample_by(opts.downsample_by as usize, |vals| {
            vals.iter().fold(f64::MIN, |max, val| max.max(*val))
        })
    } else {
        max_sustained_wind
    };

    ctx.save()?;
    render_months(
        ctx,
        year,
        &Range::new(rrange.min() - 40.0, rrange.min() - 5.0),
    )?;
    ctx.restore()?;

    ctx.save()?;
    let scale = Scale::from_range(&range, 5.0);
    render_scales(ctx, &scale, &range, rrange, " kts", Direction::Left)?;
    ctx.restore()?;

    ctx.save()?;
    render_radial_range(
        ctx,
        &mean_wind,
        &max_sustained_wind,
        rrange,
        Some(&Color::from_u32_with_alpha(0x9f83c3, 0.1)),
        Some(&Color::from_u32(0x9f83c3)),
        opts.smooth,
    )?;
    ctx.restore()?;

    ctx.save()?;
    render_center_text(
        ctx,
        &[
            (String::from("MAX"), format!("{:.1} kts", range.max())),
            (String::from("AVG"), format!("{:.1} kts", avg_mean_wind)),
        ],
        &Font::new(
            "HelveticaNeue-Medium",
            FontSlant::Normal,
            FontWeight::Bold,
            11.0,
        ),
        &Font::new(
            "HelveticaNeue-Thin",
            FontSlant::Normal,
            FontWeight::Normal,
            32.0,
        ),
        &Color::from_u32_with_alpha(0xffffff, 0.6),
        opts,
    )?;
    ctx.restore()?;

    Ok(())
}

fn render_precipitation(
    ctx: &Context,
    year: time::Year,
    station: &gsod::Station,
    rrange: &Range,
    opts: &Options,
) -> Result<(), Box<dyn Error>> {
    let percipitation = Series::for_each_day(year, station.days().iter(), |day| {
        match day.precipitation() {
            Some(p) => Some(p.in_inches()),
            None => Some(0.0),
        }
    });

    let num_days = percipitation
        .values()
        .iter()
        .fold(0, |sum, val| if *val > 0.0 { sum + 1 } else { sum });

    let total = percipitation.values().iter().sum::<f64>();

    ctx.save()?;
    render_months(
        ctx,
        year,
        &Range::new(rrange.min() - 40.0, rrange.min() - 5.0),
    )?;
    ctx.restore()?;

    let scale = Scale::from_range(percipitation.range(), 4.0);

    ctx.save()?;
    render_scales(
        ctx,
        &scale,
        percipitation.range(),
        rrange,
        " in",
        Direction::Left,
    )?;
    ctx.restore()?;

    let n = percipitation.values().len();
    let dt = TAU / n as f64;
    let t0 = -TAU / 4.0;

    ctx.save()?;
    let ra = rrange.project(Unit::zero());
    Color::from_u32(0x2fcbcc).set(ctx);
    ctx.new_path();
    for i in 0..n {
        let t = i as f64 * dt + t0;
        let rb = rrange.project(percipitation.get_normalized(i as isize));
        ctx.move_to(ra * t.cos(), ra * t.sin());
        ctx.line_to(rb * t.cos(), rb * t.sin());
    }
    ctx.stroke()?;
    ctx.restore()?;

    ctx.save()?;
    render_center_text(
        ctx,
        &[
            (String::from("DAYS"), format!("{}", num_days)),
            (String::from("TOTAL"), format!("{:.1} in", total)),
        ],
        &Font::new(
            "HelveticaNeue-Medium",
            FontSlant::Normal,
            FontWeight::Bold,
            11.0,
        ),
        &Font::new(
            "HelveticaNeue-Thin",
            FontSlant::Normal,
            FontWeight::Normal,
            32.0,
        ),
        &Color::from_u32_with_alpha(0xffffff, 0.6),
        opts,
    )?;
    ctx.restore()?;

    Ok(())
}

fn distance_across_arc(r: f64, t: f64) -> f64 {
    let dx = r * t.cos() - r;
    let dy = r * t.sin();
    (dx * dx + dy * dy).sqrt()
}

fn shorten_station_name(name: &str) -> String {
    name.replace("INTERNATIONAL", "INTL")
}

fn describe_station_details(station: &gsod::Station) -> String {
    let id = station.id();
    if let Some(location) = station.location() {
        format!("{}  {}", id, location)
    } else {
        id.to_owned()
    }
}

fn describe_year(year: time::Year) -> String {
    let s = year.start();
    let e = time::Day::new(year.end()).prev().date();
    format!("{} – {}", s.format("%b %-d, %Y"), e.format("%b %-d, %Y"))
}
