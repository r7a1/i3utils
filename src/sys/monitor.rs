use anyhow::{Error, Result};
use lazy_static::lazy_static;
use regex::Regex;

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
        lazy_static! {
            static ref PAT: Regex =
                Regex::new(r"(?P<r>\d+(\.\d+)?):(?P<g>\d+(\.\d+)?):(?P<b>\d+(\.\d+)?)")
                    .expect("failed to compile regex");
        }

        if let Some(c) = PAT.captures(s) {
            Ok(Gamma {
                r: 1.0 / c["r"].parse::<f32>()?,
                g: 1.0 / c["g"].parse::<f32>()?,
                b: 1.0 / c["b"].parse::<f32>()?,
            })
        } else {
            Err(std::io::Error::from(std::io::ErrorKind::NotFound).into())
        }
    }
}

impl std::string::ToString for Gamma {
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
    lazy_static! {
        static ref PAT: Regex =
            Regex::new(r"^(?P<monitor>.*) connected .*\+0\+0 .*").expect("failed to compile regex");
    }

    duct::cmd!("xrandr")
        .read()
        .unwrap_or_default()
        .lines()
        .find_map(|line| PAT.captures(line))
        .map(|cap| cap[1].to_string())
}

pub fn get_brightness() -> Result<f32> {
    lazy_static! {
        static ref PAT: Regex =
            Regex::new(r"Brightness: +(?P<num>.*)").expect("failed to compile regex");
    }

    duct::cmd!("xrandr", "--verbose")
        .read()?
        .lines()
        .find_map(|line| PAT.captures(line))
        .map(|cap| cap["num"].to_string())
        .ok_or(std::io::Error::from(std::io::ErrorKind::NotFound))?
        .parse::<f32>()
        .map_err(|e| e.into())
}

pub fn get_gamma() -> Result<Gamma> {
    lazy_static! {
        static ref PAT: Regex = Regex::new(r"Gamma: *.*").expect("failed to compile regex");
    }

    duct::cmd!("xrandr", "--verbose")
        .read()?
        .lines()
        .filter(|line| PAT.is_match(line))
        .map(|line| line.parse::<Gamma>().map_err(|e| e.into()))
        .next()
        .ok_or(std::io::Error::from(std::io::ErrorKind::NotFound))?
}
