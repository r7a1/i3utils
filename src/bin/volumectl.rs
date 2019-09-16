use anyhow::Result;
use clap::Parser;

use tools::sys;

#[derive(Parser)]
#[clap(about = "Control the display")]
struct Opts {
    #[clap(subcommand)]
    cmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    Up,
    Down,
    Switch,
}

fn main() -> Result<()> {
    env_logger::init();

    match Opts::parse().cmd {
        SubCommand::Up => sys::audio::volume_up(None)?,
        SubCommand::Down => sys::audio::volume_down(None)?,
        SubCommand::Switch => sys::audio::switch_sink_next()?,
    }
    Ok(())
}
