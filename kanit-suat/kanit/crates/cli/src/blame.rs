use std::fs;

use kanit_common::constants;
use kanit_common::error::{Context, Result};

use crate::flags::Blame;

#[derive(Copy, Clone)]
pub struct BlameEntry<'a> {
    name: &'a str,
    duration: u128,
    level: usize,
}

impl<'a> BlameEntry<'a> {
    pub fn parse_single_entry(line: &'a str) -> Result<Self> {
        let mut parts = line.split_whitespace();

        let name = parts.next().context("expected `name`")?;

        let duration = parts
            .next()
            .context("expected `duration`")?
            .parse::<u128>()
            .context("failed to parse `duration`")?;

        let level = parts
            .next()
            .context("expected `level`")?
            .parse::<usize>()
            .context("failed to parse `level`")?;

        Ok(Self {
            name,
            duration,
            level,
        })
    }
}

fn parse_blame(lines: &str) -> Vec<BlameEntry> {
    lines
        .lines()
        .filter_map(|line| BlameEntry::parse_single_entry(line).ok())
        .collect()
}

pub fn blame(opts: Blame) -> Result<()> {
    let timings = fs::read_to_string(constants::KAN_TIMINGS).context("failed to read timings")?;

    let blames = parse_blame(&timings);

    let mut filtered_timings = blames
        .iter()
        .filter(|l| l.name.starts_with("unit:"))
        .map(|l| BlameEntry {
            name: l.name,
            level: l.level,
            duration: ((l.duration as f64) / 1000.0).round() as u128,
        })
        .collect::<Vec<_>>();

    if opts.sorted {
        filtered_timings.sort_by(|a, b| b.duration.partial_cmp(&a.duration).unwrap());
    }

    let max_len = filtered_timings
        .iter()
        .map(|l| l.duration.to_string())
        .max_by(|a, b| a.len().cmp(&b.len()))
        .unwrap()
        .len();

    for timing in filtered_timings {
        let dur = timing.duration.to_string();

        println!(
            "{}{}ms {}",
            " ".repeat(max_len - dur.len()),
            dur,
            &timing.name[5..]
        )
    }

    Ok(())
}
