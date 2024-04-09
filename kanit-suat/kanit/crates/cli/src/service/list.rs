use std::fs;
use std::path::Path;

use kanit_common::constants;
use kanit_common::error::{Context, Result, StaticError};
use kanit_rc::db::Database;

use crate::flags::List;

pub fn list(opts: List) -> Result<()> {
    let db_path = Path::new(constants::KAN_DB);

    if !db_path.exists() {
        Err(StaticError("failed to find kanit database"))?;
    }

    let db_data = fs::read(db_path).context("failed to read database")?;

    let db = Database::load(&db_data)?;

    for (i, level) in db.levels.iter().enumerate() {
        println!("level {}", i);

        if opts.plan {
            for (o, group) in level.get_order().iter().enumerate() {
                println!("|> group {}", o);

                for unit in group.iter() {
                    println!("    |> {}", unit);
                }
            }
        } else {
            for unit in level.get_order().iter().flatten() {
                println!("|> {}", unit);
            }
        }
    }

    Ok(())
}
