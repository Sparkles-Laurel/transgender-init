use std::fs;
use std::io::Write;
use std::path::Path;

use atomic_write_file::AtomicWriteFile;

use kanit_common::constants;
use kanit_common::error::{Context, Result, StaticError};
use kanit_rc::db::Database;
use kanit_unit::UnitName;

use crate::flags::Disable;

pub fn disable(opts: Disable) -> Result<()> {
    let db_path = Path::new(constants::KAN_DB);

    if !db_path.exists() {
        Err(StaticError("failed to find kanit database"))?;
    }

    let db_data = fs::read(db_path).context("failed to read database")?;

    let mut db = Database::load(&db_data)?;

    let unit_name = UnitName::from(opts.unit.as_str());

    // patch database
    let level = opts.runlevel.unwrap_or(1);

    let level_set = db.enabled.get_mut(level).unwrap();

    if !level_set.contains(&unit_name) {
        Err(StaticError("unit was not enabled"))?;
    }

    level_set.remove(&unit_name);
    db.unit_infos.remove(&unit_name);
    db.units.remove(&unit_name);

    db.rebuild_levels()?;

    let new_db_data = db.dump()?;

    let mut db_handle = AtomicWriteFile::open(db_path).context("failed to open database")?;

    db_handle.write_all(&new_db_data)?;

    db_handle.commit().context("failed to commit database")?;

    Ok(())
}
