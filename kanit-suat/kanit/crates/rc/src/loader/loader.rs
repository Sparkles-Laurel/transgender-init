use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::rc::Rc;
use std::sync::OnceLock;

use async_lock::Mutex;
use log::warn;
use send_wrapper::SendWrapper;

use kanit_common::constants;
use kanit_common::error::{Context, Result, StaticError};
use kanit_unit::{RcUnit, UnitName};

use crate::db::Database;

static LOADER: OnceLock<SendWrapper<RefCell<Loader>>> = OnceLock::new();

pub struct Loader {
    pub defaulted: bool,
    pub started: Vec<HashSet<UnitName>>,
    pub map: HashMap<UnitName, RcUnit>,
    pub ev_lock: Rc<Mutex<()>>, // i am pro at rust
    database: Database,
}

impl Loader {
    pub fn initialize<F>(default: F) -> Result<()>
    where
        F: FnOnce() -> Result<Database>,
    {
        let defaulted;

        let mut database = if let Ok(bytes) = fs::read(constants::KAN_DB) {
            defaulted = true;
            if let Ok(db) = Database::load(&bytes) {
                db
            } else {
                warn!("failed to load database, using default");
                // defaulted flag not set to preserve database just in case of recovery
                default()?
            }
        } else {
            defaulted = true;
            default()?
        };

        let map = database.get_base_map();
        let mut started = vec![];

        started.resize(database.levels.len(), HashSet::new());

        // ignore error since it just returns what we tried to load
        let _ = LOADER.set(SendWrapper::new(RefCell::new(Self {
            ev_lock: Rc::new(Mutex::new(())),
            defaulted,
            started,
            database,
            map,
        })));

        Ok(())
    }

    pub fn reload(&mut self) -> Result<()> {
        let mut database = if let Ok(bytes) = fs::read(constants::KAN_DB) {
            Database::load(&bytes)?
        } else {
            return Ok(());
        };

        self.map = database.get_base_map();
        self.database = database;

        Ok(())
    }

    pub fn obtain() -> Result<&'static SendWrapper<RefCell<Self>>> {
        let loader = LOADER.get().context("loader not initialized")?;

        if !loader.valid() {
            Err(StaticError("cannot obtain loader from different thread"))?;
        }

        Ok(loader)
    }

    pub fn extend_map<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (UnitName, RcUnit)>,
    {
        self.map.extend(iter);
    }

    pub fn get_levels(&self) -> usize {
        self.database.get_levels()
    }

    pub fn get_level(&self, level: usize) -> Vec<Vec<RcUnit>> {
        self.database.get_level(&self.map, level)
    }

    pub fn get_unit(&self, name: &UnitName) -> Option<RcUnit> {
        self.map.iter().for_each(|n| {
            dbg!(n.0);
        });
        self.map.get(name).cloned()
    }

    pub fn mark_started(&mut self, level: usize, name: UnitName) {
        if let Some(level) = self.started.get_mut(level) {
            level.insert(name);
        }
    }

    pub fn mark_stopped(&mut self, level: usize, name: &UnitName) {
        if let Some(level) = self.started.get_mut(level) {
            level.remove(name);
        }
    }

    pub fn is_started(&self, level: usize, name: &UnitName) -> bool {
        if let Some(level) = self.started.get(level) {
            level.contains(name)
        } else {
            false
        }
    }

    pub fn dump_db(&self) -> Result<Vec<u8>> {
        self.database.dump()
    }

    pub fn database(&self) -> &Database {
        &self.database
    }
}
