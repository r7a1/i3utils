use std::io::Read;
use anyhow::Result;
use clap::Parser;

use i3utils::i3;

#[derive(Parser)]
struct Opts {
    #[clap(subcommand)]
    cmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    Clean,
    ToggleFullscreen,
    FocusNextmatch { name: String },
    RunOrRaise { cmd: String, class: String },
}

fn main() -> Result<()> {
    let mut controller = i3::Util::new()?;

    match Opts::parse().cmd {
        SubCommand::Clean => controller.clean_layout_backup()?,
        SubCommand::ToggleFullscreen => controller.toggle_fullscreen()?,
        SubCommand::FocusNextmatch { name } => controller.focus_nextmatch(name)?,
        SubCommand::RunOrRaise { cmd, class } => controller.run_or_raise(&cmd, &class)?,
    }
    Ok(())
}
