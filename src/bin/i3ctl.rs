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
    /// Clean up cache.
    Clean,
    /// Toggle fullscreen.
    ToggleFullscreen,
    /// Focus matched window.
    FocusNextmatch { name: String },
    /// Focus the window if it exists, run command otherwise.
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
