use std::cmp::Ordering;
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use atomic_write_file::AtomicWriteFile;

use kanit_common::constants;
use kanit_common::error::{Context, Result, StaticError, WithError};
use kanit_rc::db::{Database, DbUnit};

use crate::flags::Enable;

pub fn enable(opts: Enable) -> Result<()> {
    let db_path = Path::new(constants::KAN_DB);

    if !db_path.exists() {
        Err(StaticError("failed to find kanit database"))?;
    }

    let mut unit = PathBuf::from(constants::KAN_UNIT_DIR);

    unit.push(opts.unit);

    let unit_contents = fs::read_to_string(unit).context("failed to read unit")?;

    let unit_data: DbUnit = toml::from_str(&unit_contents).context("failed to parse")?;

    let db_data = fs::read(db_path).context("failed to read database")?;

    let mut db = Database::load(&db_data)?;

    // patch database
    let level = opts.runlevel.unwrap_or(1);

    match db.enabled.len().cmp(&level) {
        Ordering::Equal => db.enabled.push(HashSet::new()),
        Ordering::Less => Err(WithError::with(move || {
            format!("cannot create level {}", level)
        }))?,
        _ => {}
    }

    let level_set = db.enabled.get_mut(level).unwrap();

    if level_set.contains(&unit_data.name) {
        Err(StaticError("unit already enabled"))?;
    }

    level_set.insert(unit_data.name.clone());
    db.unit_infos
        .insert(unit_data.name.clone(), unit_data.get_unit_info());
    db.units.insert(unit_data.name.clone(), unit_data);

    db.rebuild_levels()?;

    let new_db_data = db.dump()?;

    let mut db_handle = AtomicWriteFile::open(db_path).context("failed to open database")?;

    db_handle.write_all(&new_db_data)?;

    db_handle.commit().context("failed to commit database")?;

    Ok(())
}
