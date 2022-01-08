use anyhow::Result;
use clap::Parser;

use i3utils::sys;

#[derive(Parser)]
struct Opts {
    #[clap(subcommand)]
    cmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    Reset,
    Up,
    Down,
    Dark,
    Darkest,
}

fn main() -> Result<()> {
    // Determine brightness and gamma to set.
    let (brightness, gamma) = match Opts::parse().cmd {
        SubCommand::Reset => (Some(1.0), Some("1.0:1.0:1.0".to_owned())),
        SubCommand::Up => (Some(sys::monitor::get_brightness()? + 0.05), None),
        SubCommand::Down => (Some(sys::monitor::get_brightness()? - 0.05), None),
        SubCommand::Dark => (Some(1.0), Some("1.0:0.6:0.3".to_owned())),
        SubCommand::Darkest => (Some(0.2), Some("0.5:0.2:0.1".to_owned())),
    };

    // Provide brightness limits.
    if let Some(brightness) = brightness {
        if brightness < 0.05 || 1.0 < brightness {
            eprintln!("reached to the limit.");
            std::process::exit(0);
        }
    }

    sys::monitor::apply(brightness, gamma).map_err(|e| e.into())
}
