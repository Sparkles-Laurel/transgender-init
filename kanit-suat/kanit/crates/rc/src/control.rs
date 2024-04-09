use std::fs;
#[cfg(not(feature = "testing"))]
use std::io::{stdin, stdout, Write};

use async_process::driver;
use log::{debug, error, info, warn};

use kanit_common::constants;
use kanit_common::error::{Context, Error, Result};
use kanit_diagnostics::tap as kanit_tap;
use kanit_diagnostics::timing as kanit_timing;
use kanit_executor::{join_all, spawn};
use kanit_unit::{RcUnit, UnitName};

pub use crate::event::event;
use crate::loader;
use crate::loader::Loader;

#[cfg(not(feature = "testing"))]
fn critical_unit_fail(err: Error) -> Result<()> {
    loop {
        eprint!("a critical unit failed to start, continue? [y/n]: ");

        let _ = stdout().flush();

        let mut input = String::new();

        match stdin().read_line(&mut input) {
            Ok(_) => match input.chars().next() {
                Some('y') => return Ok(()),
                Some('n') => return Err(err),
                _ => {}
            },
            Err(e) => return Err(e).context("failed to read stdin"),
        }
    }
}

#[cfg(feature = "testing")]
fn critical_unit_fail(err: Error) -> Result<()> {
    kanit_tap::bail(Some(err.to_string()));

    Err(err)
}

fn write_db(loader: &Loader) -> Result<()> {
    fs::write(constants::KAN_DB, loader.dump_db()?).context("failed to write database")
}

async fn start_unit(tuple: (usize, RcUnit)) -> Result<Option<UnitName>> {
    let (j, unit) = tuple;

    let mut unit_b = unit.borrow_mut();

    debug!("loading unit {}", unit_b.name());

    let id = kanit_timing::push_scope(format!("unit:{}", unit_b.name()));

    if !unit_b.prepare().await? {
        warn!("failed preparations for {}", unit_b.name());
        kanit_tap::not_ok(j + 1, Some("failed preparations"));
        return Ok(None);
    }

    if let Err(e) = unit_b.start().await {
        if e.is_recoverable() {
            warn!("{}", e);
            kanit_tap::not_ok(j + 1, Some(e));
        } else {
            error!("{}", e);
            critical_unit_fail(e)?;
        }

        kanit_timing::pop_scope(id);

        return Ok(None);
    }

    kanit_timing::pop_scope(id);

    kanit_tap::ok(j + 1, Some(unit_b.name()));

    debug!("finished loading unit {}", unit_b.name());

    Ok(Some(unit_b.name().clone()))
}

pub async fn start() -> Result<()> {
    kanit_timing::register();

    loader::init_loader()?;

    let mut loader = Loader::obtain()?.borrow_mut();

    let loader_levels = loader.get_levels();

    kanit_tap::plan(loader_levels * 2); // include teardown as well

    let driver_task = spawn(driver());

    for i in 0..loader_levels {
        info!("starting level {}", i);

        let scope_str = format!("level:{}", i);

        kanit_tap::enter_subtest(Some(&scope_str));

        let id = kanit_timing::push_scope(&scope_str);

        let level = loader.get_level(i);

        kanit_tap::plan(level.len());

        for (j, group) in level.into_iter().enumerate() {
            let group_str = format!("group:{}", j);

            kanit_tap::enter_subtest(Some(&group_str));
            kanit_tap::plan(group.len());

            let handles = join_all(group.into_iter().enumerate().map(start_unit)).await;

            for handle in handles {
                if let Some(name) = handle? {
                    loader.mark_started(i, name);
                }
            }

            kanit_tap::exit_subtest();
            kanit_tap::ok(j + 1, Some(&group_str));
        }

        kanit_timing::pop_scope(id);

        kanit_tap::exit_subtest();
        kanit_tap::ok(i + 1, Some(&scope_str));
    }

    if loader.defaulted {
        if let Err(e) = write_db(&loader) {
            warn!("{}", e);
        }
    }

    driver_task.cancel().await;

    Ok(())
}

async fn stop_unit(tuple: (usize, RcUnit)) -> Result<()> {
    let (j, unit) = tuple;

    let mut unit_b = unit.borrow_mut();

    debug!("unloading unit {}", unit_b.name());

    match unit_b.stop().await {
        Ok(_) => kanit_tap::ok(j + 1, Some(unit_b.name())),
        Err(e) => {
            kanit_tap::not_ok(j + 1, Some(unit_b.name()));

            if e.is_recoverable() {
                warn!("{}", e)
            } else {
                error!("{}", e) // won't stop still
            }
        }
    }

    debug!("finished unloading unit {}", unit_b.name());

    Ok(())
}

pub async fn teardown() -> Result<()> {
    let loader = Loader::obtain()?.borrow();

    let loader_levels = loader.get_levels();

    for i in (0..loader_levels).rev() {
        info!("stopping level {}", i);

        #[cfg(feature = "testing")]
        let scope_str = format!("level:{}-stop", i);
        #[cfg(feature = "testing")]
        kanit_tap::enter_subtest(Some(&scope_str));

        let level = loader.get_level(i);

        kanit_tap::plan(level.len());

        for (j, group) in level.into_iter().enumerate() {
            let group_str = format!("group:{}", j);

            kanit_tap::enter_subtest(Some(&group_str));
            kanit_tap::plan(group.len());

            let handles = join_all(group.into_iter().enumerate().map(stop_unit)).await;

            for handle in handles {
                handle?;
            }

            kanit_tap::exit_subtest();
            kanit_tap::ok(j + 1, Some(&group_str));
        }

        kanit_tap::exit_subtest();
        #[cfg(feature = "testing")]
        kanit_tap::ok(loader_levels + (loader_levels - i), Some(&scope_str));
    }

    Ok(())
}
