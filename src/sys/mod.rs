use anyhow::Result;

pub fn pidof(name: &str) -> Result<Vec<u32>> {
    Ok(duct::cmd!("pidof", name)
        .read()?
        .split_whitespace()
        .filter_map(|pid| pid.parse().ok())
        .collect())
}

pub mod audio;
pub mod monitor;
pub mod xwindow;
