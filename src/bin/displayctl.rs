use anyhow::Result;
use clap::Parser;

use i3utils::sys::monitor;

#[derive(Parser)]
struct Opts {
    #[clap(subcommand)]
    cmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    /// Reset to default display parameters.
    Reset,
    /// Brighten up.
    Up,
    /// Dim down.
    Down,
    /// Set dark display parameters.
    Dark,
    /// Set darkest display parameters.
    Darkest,
}

const STEP: f64 = 0.05;

fn main() -> Result<()> {
    // Determine brightness and gamma to set.
    let (brightness, gamma) = match Opts::parse().cmd {
        SubCommand::Reset => (Some(1.0), Some("1.0:1.0:1.0".to_owned())),
        SubCommand::Up => (Some(monitor::get_brightness()? + STEP), None),
        SubCommand::Down => (Some(monitor::get_brightness()? - STEP), None),
        SubCommand::Dark => (Some(1.0), Some("1.0:0.6:0.3".to_owned())),
        SubCommand::Darkest => (Some(0.1), Some("0.5:0.2:0.1".to_owned())),
    };

    if let Some(brightness) = brightness {
        if brightness < (0.0 + STEP) || (1.0 - STEP) < brightness {
            anyhow::bail!("reached to the limit.");
        }
    }

    monitor::apply(brightness, gamma)?;
    Ok(())
}
