use anyhow::Result;
use lazy_regex::regex_captures;
use log::{debug, info};

type Index = u32;

#[derive(Debug)]
struct Sink(Index, String);

const VOL_STEP: u8 = 5;

pub fn volume_up(inc: Option<u8>) -> Result<()> {
    let inc = inc.unwrap_or(VOL_STEP);
    set(&format!("+{inc}%"))
}

pub fn volume_down(dec: Option<u8>) -> Result<()> {
    let dec = dec.unwrap_or(VOL_STEP);
    set(&format!("-{dec}%"))
}

pub fn switch_to_next_sink() -> Result<()> {
    let sinks = list_sinks()?;
    let current = default_sink()?;

    if let Some((idx, _)) = sinks.iter().enumerate().find(|(_, s)| s.1 == current) {
        let Sink(next_id, next_sink) = &sinks[(idx + 1) % sinks.len()];
        info!("switching sink to [{next_id}] {next_sink}");

        duct::cmd!("pactl", "set-default-sink", next_sink).run()?;
        for input in &list_inputs()? {
            duct::cmd!("pactl", "move-sink-input", &input.to_string(), next_sink).run()?;
        }
    }
    Ok(())
}

// Implementation

fn set(volume: &str) -> Result<()> {
    let sink = default_sink()?;

    debug!("volume control: {volume}");
    duct::cmd!("pactl", "set-sink-volume", sink, volume).run()?;
    Ok(())
}

fn list_sinks() -> Result<Vec<Sink>> {
    duct::cmd!("pactl", "list", "short", "sinks")
        .read()?
        .lines()
        .inspect(|line| debug!("sinks: {line}"))
        .filter_map(|line| {
            regex_captures!(r"^(?P<index>[0-9]+)\s+(?P<name>.+)\s+PipeWire\s+.*", line)
        })
        .map(|(_, index, name)| Ok(Sink(index.parse::<Index>()?.into(), name.into())))
        .collect::<Result<Vec<Sink>>>()
}

fn default_sink() -> Result<String> {
    duct::cmd!("pactl", "info")
        .read()?
        .lines()
        .inspect(|line| debug!("inputs: {}", line))
        .find_map(|line| regex_captures!(r"Default Sink: +(?P<name>.+)", line))
        .map(|(_, name)| name.into())
        .ok_or_else(|| anyhow::format_err!("no default sink detected."))
}

fn list_inputs() -> Result<Vec<Index>> {
    let inputs = duct::cmd!("pactl", "list", "short", "sink-inputs")
        .read()?
        .lines()
        .filter_map(|line| regex_captures!(r"(?P<index>[0-9]+) +.*", line))
        .map(|(_, index)| index.parse::<Index>().unwrap())
        .collect::<Vec<Index>>();
    Ok(inputs)
}
