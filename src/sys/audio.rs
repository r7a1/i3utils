use anyhow::Result;
use lazy_regex::regex;
use log::{debug, info};

#[derive(Debug)]
struct Sink {
    index: u16,
    name: String,
}

pub fn volume_up(inc: Option<i8>) -> Result<()> {
    volume(inc.unwrap_or(5))
}

pub fn volume_down(dec: Option<i8>) -> Result<()> {
    volume(-dec.unwrap_or(5))
}

pub fn switch_sink_next() -> Result<()> {
    let current = default_sink()?;
    switch_next(list_sinks()?, &current, list_inputs()?)
}

fn volume(percent: i8) -> Result<()> {
    let sink = default_sink()?;
    let param = if percent > 0 {
        format!("+{}%", percent)
    } else {
        format!("-{}%", -percent)
    };
    debug!("volume control: {}", &param);
    duct::cmd!("pactl", "set-sink-volume", sink, &param).run()?;
    Ok(())
}

fn list_sinks() -> Result<Vec<Sink>> {
    duct::cmd!("pactl", "list", "short", "sinks")
        .read()?
        .lines()
        .inspect(|line| debug!("sinks: {}", line))
        .filter_map(|line| {
            regex!(r"^(?P<index>[0-9]+)\s+(?P<name>.+)\s+PipeWire\s+.*").captures(line)
        })
        .map(|cap| {
            Ok(Sink {
                index: cap["index"].parse::<u16>()?.to_owned(),
                name: cap["name"].to_string(),
            })
        })
        .collect::<Result<Vec<Sink>>>()
}

fn default_sink() -> Result<String> {
    duct::cmd!("pactl", "info")
        .read()?
        .lines()
        .inspect(|line| debug!("inputs: {}", line))
        .filter_map(|line| regex!(r"Default Sink: +(?P<name>.+)").captures(line))
        .map(|cap| cap["name"].to_string())
        .next()
        .ok_or_else(|| anyhow::format_err!("no default sink found."))
}

fn list_inputs() -> Result<Vec<u16>> {
    Ok(duct::cmd!("pactl", "list", "short", "sink-inputs")
        .read()?
        .lines()
        .filter_map(|line| regex!(r"(?P<index>[0-9]+) +.*").captures(line))
        .filter_map(|cap| cap["index"].parse::<u16>().ok())
        .collect::<Vec<u16>>())
}

fn switch_next(sinks: Vec<Sink>, current: &str, inputs: Vec<u16>) -> Result<()> {
    if let Some((idx, _)) = sinks
        .iter()
        .enumerate()
        .find(|(_, sink)| sink.name == current)
    {
        let next = &sinks[(idx + 1) % sinks.len()];
        info!("switch to sink: {:?}", &next);

        duct::cmd!("pactl", "set-default-sink", &next.name).run()?;
        for input in &inputs {
            duct::cmd!("pactl", "move-sink-input", &input.to_string(), &next.name).run()?;
        }
    }
    Ok(())
}
