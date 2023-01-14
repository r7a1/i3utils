use std::{io, string::ToString};

use anyhow::{Error, Result};
use lazy_regex::{regex_captures, regex_is_match};

pub fn apply(brightness: Option<f32>, gamma: Option<String>) -> Result<()> {
    let mut cmd = std::process::Command::new("xrandr");

    cmd.args(&[
        "--output",
        &get_primary_monitor().unwrap_or("HDMI-0".to_string()),
    ]);

    if let Some(brightness) = brightness {
        cmd.args(&["--brightness", &brightness.to_string()]);
    }
    cmd.args(&["--gamma", &gamma.unwrap_or(get_gamma()?.to_string())]);

    cmd.output()?;
    Ok(())
}

#[derive(Debug)]
pub struct Gamma {
    r: f32,
    g: f32,
    b: f32,
}

impl std::str::FromStr for Gamma {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if let Some((_, r, _, g, _, b, _)) = regex_captures!(
            r"(?P<r>\d+(\.\d+)?):(?P<g>\d+(\.\d+)?):(?P<b>\d+(\.\d+)?)",
            s
        ) {
            Ok(Gamma {
                r: 1.0 / r.parse::<f32>()?,
                g: 1.0 / g.parse::<f32>()?,
                b: 1.0 / b.parse::<f32>()?,
            })
        } else {
            Err(io::Error::from(io::ErrorKind::NotFound).into())
        }
    }
}

impl ToString for Gamma {
    fn to_string(&self) -> String {
        format!(
            "{r:1.1}:{g:1.1}:{b:1.1}",
            r = self.r,
            g = self.g,
            b = self.b
        )
    }
}

pub fn get_primary_monitor() -> Option<String> {
    duct::cmd!("xrandr")
        .read()
        .unwrap_or_default()
        .lines()
        .find_map(|line| regex_captures!(r"^(?P<monitor>.*) connected .*\+0\+0 .*", line))
        .map(|(_, monitor)| monitor.to_string())
}

pub fn get_brightness() -> Result<f32> {
    duct::cmd!("xrandr", "--verbose")
        .read()?
        .lines()
        .find_map(|line| regex_captures!(r"Brightness: +(?P<num>.*)", line))
        .map(|(_, num)| num.to_string())
        .ok_or(io::Error::from(io::ErrorKind::NotFound))?
        .parse::<f32>()
        .map_err(|e| e.into())
}

pub fn get_gamma() -> Result<Gamma> {
    duct::cmd!("xrandr", "--verbose")
        .read()?
        .lines()
        .filter(|line| regex_is_match!(r"Gamma: *.*", line))
        .map(|line| line.parse::<Gamma>().map_err(|e| e.into()))
        .next()
        .ok_or(io::Error::from(io::ErrorKind::NotFound))?
}
