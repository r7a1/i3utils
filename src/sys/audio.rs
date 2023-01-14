use anyhow::Result;
use lazy_regex::regex_captures;
use log::{debug, info};

type Index = u32;

#[derive(Debug)]
struct Sink(Index, String);

pub fn volume_up(inc: Option<u8>) -> Result<()> {
    let inc = inc.unwrap_or(5);
    set_volume(&format!("+{inc}%"))
}

pub fn volume_down(dec: Option<u8>) -> Result<()> {
    let dec = dec.unwrap_or(5);
    set_volume(&format!("-{dec}%"))
}

pub fn switch_sink_next() -> Result<()> {
    let current = default_sink()?;
    let sinks = list_sinks()?;
    let inputs = list_inputs()?;

    switch_next(sinks, &current, inputs)
}

// Implementation

fn set_volume(param: &str) -> Result<()> {
    let sink = default_sink()?;

    debug!("volume control: {}", &param);
    duct::cmd!("pactl", "set-sink-volume", sink, param).run()?;
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
        .map(|(_, index, name)| Ok(Sink(index.parse::<Index>()?.to_owned(), name.to_string())))
        .collect::<Result<Vec<Sink>>>()
}

fn default_sink() -> Result<String> {
    duct::cmd!("pactl", "info")
        .read()?
        .lines()
        .inspect(|line| debug!("inputs: {}", line))
        .filter_map(|line| regex_captures!(r"Default Sink: +(?P<name>.+)", line))
        .map(|(_, name)| name.to_string())
        .next()
        .ok_or_else(|| anyhow::format_err!("no default sink found."))
}

fn list_inputs() -> Result<Vec<Index>> {
    let inputs = duct::cmd!("pactl", "list", "short", "sink-inputs")
        .read()?
        .lines()
        .filter_map(|line| regex_captures!(r"(?P<index>[0-9]+) +.*", line))
        .filter_map(|(_, index)| index.parse::<Index>().ok())
        .collect::<Vec<Index>>();
    Ok(inputs)
}

fn switch_next(sinks: Vec<Sink>, current: &str, inputs: Vec<Index>) -> Result<()> {
    let sink = sinks.iter().enumerate().find(|(_, sink)| sink.1 == current);

    if let Some((idx, _)) = sink {
        let Sink(id, next) = &sinks[(idx + 1) % sinks.len()];
        info!("switching sink: {id}: {next}");

        duct::cmd!("pactl", "set-default-sink", next).run()?;
        for input in &inputs {
            duct::cmd!("pactl", "move-sink-input", &input.to_string(), next).run()?;
        }
    }
    Ok(())
}
