use anyhow::Result;
use clap::Parser;

use i3utils::sys;

#[derive(Parser)]
#[clap(about = "Control the display")]
struct Opts {
    #[clap(subcommand)]
    cmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    /// Volume up.
    Up,
    /// Volume down.
    Down,
    /// Switch to next sink.
    Switch,
}

fn main() -> Result<()> {
    env_logger::init();

    match Opts::parse().cmd {
        SubCommand::Up => sys::audio::volume_up(None)?,
        SubCommand::Down => sys::audio::volume_down(None)?,
        SubCommand::Switch => sys::audio::switch_to_next_sink()?,
    }
    Ok(())
}
