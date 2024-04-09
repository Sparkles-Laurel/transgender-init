use std::collections::HashSet;

use log::warn;

use kanit_common::error::{Context, Result, StaticError};
use kanit_unit::UnitName;

use crate::loader::Loader;

async fn modify_service(start: bool, level: usize, name: &[u8]) -> Result<()> {
    // this is horrible but it makes the compiler happy
    let (diff, groups) = {
        let loader = Loader::obtain()?.borrow();

        let unit_name = UnitName::from(String::from_utf8_lossy(name));

        if (start && loader.is_started(level, &unit_name))
            || (!start && !loader.is_started(level, &unit_name))
        {
            Err(StaticError("unit already started/stopped"))?;
        }

        // rebuild database and diff to find out what needs to start
        let mut db = loader.database().clone();

        {
            let enabled = db.enabled.get_mut(level).context("failed to get level")?;

            if start {
                enabled.insert(unit_name.clone());
            } else {
                enabled.remove(&unit_name);
            }
        }

        db.rebuild_levels()?;

        if !db.unit_infos.contains_key(&unit_name) {
            Err(StaticError("failed to find unit in database"))?;
        }

        let started = loader.started.get(level).context("failed to get level")?;

        // unwrap: we `get_mut` earlier
        let enabled = db.enabled.get(level).unwrap();

        let (diff, levels) = if start {
            (
                enabled.difference(started).cloned().collect::<HashSet<_>>(),
                db.levels,
            )
        } else {
            (
                started.difference(enabled).cloned().collect::<HashSet<_>>(),
                loader.database().clone().levels,
            )
        };

        let groups = levels
            .get(level)
            .context("failed to get level")?
            .get_order();

        (diff, groups.clone())
    };

    let mut loader = Loader::obtain()?.borrow_mut();

    if start {
        for group in groups {
            for unit_n in group.iter().filter(|u| diff.contains(*u)) {
                let unit = loader.get_unit(unit_n).context("failed to get unit")?;

                let mut unit_b = unit.borrow_mut();

                if !unit_b.prepare().await? {
                    warn!("failed preparations for {}", unit_b.name());

                    continue;
                }

                if let Err(e) = unit_b.start().await {
                    warn!("{}", e);
                    return Err(e);
                } else {
                    loader.mark_started(level, unit_b.name());
                }
            }
        }
    } else {
        for group in groups.iter().rev() {
            for unit_n in group.iter().filter(|u| diff.contains(*u)) {
                let unit = loader.get_unit(unit_n).context("failed to get unit")?;

                let mut unit_b = unit.borrow_mut();

                if let Err(e) = unit_b.stop().await {
                    warn!("{}", e);
                    return Err(e);
                } else {
                    loader.mark_stopped(level, &unit_b.name());
                }
            }
        }
    }

    Ok(())
}

pub async fn event(data: Vec<u8>) -> Result<()> {
    if data.starts_with(b"db-reload") {
        let mut loader = Loader::obtain()?.borrow_mut();

        let ev_lock = loader.ev_lock.clone();

        let lock = ev_lock.lock().await;

        loader.reload()?;

        drop(lock);
    } else if data.starts_with(b"start") || data.starts_with(b"stop") {
        // start:tty:1
        let mut parts = data.split(|b| *b == b':');

        parts.next(); // forward start/stop

        let name = parts.next().context("failed to get name")?;

        let level = String::from_utf8_lossy(parts.next().context("failed to get level")?)
            .trim()
            .parse::<usize>()
            .context("failed to parse level")?;

        let ev_lock = Loader::obtain()?.borrow().ev_lock.clone();

        let lock = ev_lock.lock().await; // get lock to ensure no one else is using the loader

        modify_service(data.starts_with(b"start"), level, name).await?;

        drop(lock);
    }

    Ok(())
}
