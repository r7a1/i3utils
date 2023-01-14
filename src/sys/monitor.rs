use std::{process::Command, str::FromStr, string::ToString};

use anyhow::{Error, Result};
use lazy_regex::{regex_captures, regex_is_match};

pub fn apply(brightness: Option<f64>, gamma: Option<String>) -> Result<()> {
    let mut cmd = Command::new("xrandr");

    cmd.args(&["--output", &get_primary_monitor()?]);
    if let Some(brightness) = brightness {
        cmd.args(&["--brightness", &brightness.to_string()]);
    }
    cmd.args(&["--gamma", &gamma.unwrap_or(get_gamma()?.to_string())]);

    cmd.status()?;
    Ok(())
}

#[derive(Debug)]
pub struct Gamma {
    r: f32,
    g: f32,
    b: f32,
}

impl FromStr for Gamma {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cap = regex_captures!(r"(?P<r>\d+\.\d+):(?P<g>\d+\.\d+):(?P<b>\d+\.\d+)", s);

        if let Some((r, g, b, _)) = cap {
            let gamma = Gamma {
                r: 1.0 / r.parse::<f32>()?,
                g: 1.0 / g.parse::<f32>()?,
                b: 1.0 / b.parse::<f32>()?,
            };
            return Ok(gamma);
        }

        anyhow::bail!("invalid gamma value");
    }
}

impl ToString for Gamma {
    fn to_string(&self) -> String {
        format!("{:1.1}:{:1.1}:{:1.1}", self.r, self.g, self.b)
    }
}

pub fn get_primary_monitor() -> Result<String> {
    duct::cmd!("xrandr")
        .read()?
        .lines()
        .find_map(|line| regex_captures!(r"^(?P<monitor>.*) connected .*\+0\+0 .*", line))
        .map(|(_, monitor)| monitor.to_string())
        .ok_or(anyhow::anyhow!("no primary monitor detected"))
}

pub fn get_brightness() -> Result<f64> {
    let brightness = duct::cmd!("xrandr", "--verbose")
        .read()?
        .lines()
        .find_map(|line| regex_captures!(r"Brightness:\s+(?P<num>.+)", line))
        .map(|(_, num)| num.to_string())
        .ok_or(anyhow::anyhow!("no brightness found"))?
        .parse()?;
    Ok(brightness)
}

pub fn get_gamma() -> Result<Gamma> {
    duct::cmd!("xrandr", "--verbose")
        .read()?
        .lines()
        .filter(|line| regex_is_match!(r"Gamma:\s+.+", line))
        .find_map(|line| line.parse::<Gamma>().ok())
        .ok_or(anyhow::anyhow!("no gamma found"))
}
