use std::collections::{HashMap, HashSet};
use std::mem;

#[cfg(feature = "postcard")]
use postcard::{from_bytes, to_stdvec};
#[cfg(feature = "rkyv")]
use rkyv::ser::serializers::AllocSerializer;
#[cfg(feature = "rkyv")]
use rkyv::ser::Serializer;
#[cfg(feature = "rkyv")]
use rkyv::{from_bytes, Archive, Deserialize, Serialize};
#[cfg(feature = "postcard")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "rkyv")]
use kanit_common::error::StaticError;
use kanit_common::error::{Context, Result, WithError};
use kanit_unit::{wrap_unit, RcUnit, UnitInfo, UnitName};
pub use unit::DbUnit;

use crate::loader::obtain_load_order;

mod unit;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "rkyv", derive(Archive))]
#[cfg_attr(feature = "rkyv", archive(check_bytes))]
pub struct Level(Vec<Vec<UnitName>>);

impl Level {
    pub fn build(map: &HashMap<UnitName, UnitInfo>, enabled: &HashSet<UnitName>) -> Result<Self> {
        let enabled_services = map
            .iter()
            .filter(|s| enabled.contains(s.0))
            .collect::<Vec<_>>();

        let deps = enabled_services
            .iter()
            .map(|s| s.1.dependencies.clone())
            .collect::<Vec<_>>();

        let mut to_load = enabled_services
            .iter()
            .map(|s| s.0.clone())
            .collect::<HashSet<_>>();

        let needs = deps.iter().flat_map(|d| &d.needs).collect::<HashSet<_>>();

        for need in needs {
            if !map.contains_key(need) {
                let need = need.clone();

                Err(WithError::with(move || {
                    format!("failed to find dependency `{}`", need)
                }))?;
            }

            to_load.insert(need.clone());
        }

        to_load.extend(
            deps.iter()
                .flat_map(|d| d.wants.clone())
                .filter(|d| map.contains_key(d)),
        );

        // only `needs` and `wants` put explicit dependency bounds
        // `uses` is just a recommendation
        // `before`, `after` are just recommendations

        // unwrap: to_load has been checked to exist in map
        let units = to_load
            .iter()
            .map(|n| (*map.get(n).unwrap()).clone())
            .collect::<Vec<_>>();

        let order = obtain_load_order(units)?
            .iter()
            .map(|n| n.iter().map(|m| m.name.clone()).collect::<Vec<_>>())
            .collect::<Vec<_>>();

        Ok(Level(order))
    }

    pub fn get_order(&self) -> &Vec<Vec<UnitName>> {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "rkyv", derive(Archive))]
#[cfg_attr(feature = "rkyv", archive(check_bytes))]
pub struct Database {
    pub enabled: Vec<HashSet<UnitName>>,
    pub levels: Vec<Level>,
    pub unit_infos: HashMap<UnitName, UnitInfo>,
    pub units: HashMap<UnitName, DbUnit>,
}

impl Database {
    pub fn new(
        units: HashMap<UnitName, DbUnit>,
        services: HashMap<UnitName, RcUnit>,
        enabled: Vec<HashSet<UnitName>>,
    ) -> Result<Self> {
        let mut levels = vec![];

        let unit_infos = services
            .iter()
            .map(|n| (n.0.clone(), UnitInfo::new(n.1)))
            .collect::<HashMap<_, _>>();

        for level in enabled.iter() {
            levels.push(Level::build(&unit_infos, level)?);
        }

        Ok(Self {
            unit_infos,
            enabled,
            levels,
            units,
        })
    }

    pub fn rebuild_levels(&mut self) -> Result<()> {
        self.levels = vec![];

        for level in self.enabled.iter() {
            self.levels.push(Level::build(&self.unit_infos, level)?);
        }

        Ok(())
    }

    pub fn get_base_map(&mut self) -> HashMap<UnitName, RcUnit> {
        let units = mem::take(&mut self.units);

        units
            .into_iter()
            .map(|u| (u.0.clone(), wrap_unit(u.1)))
            .collect()
    }

    pub fn get_levels(&self) -> usize {
        self.levels.len()
    }

    pub fn get_level(&self, map: &HashMap<UnitName, RcUnit>, level: usize) -> Vec<Vec<RcUnit>> {
        if let Some(groups) = self.levels.get(level) {
            groups
                .0
                .iter()
                .map(|n| {
                    n.iter()
                        .filter_map(|m| map.get(m).cloned())
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        } else {
            vec![]
        }
    }

    #[cfg(feature = "rkyv")]
    pub fn dump(&self) -> Result<Vec<u8>> {
        let mut serializer = AllocSerializer::<512>::default();
        serializer
            .serialize_value(self)
            .context("failed to serialize database")?;
        let bytes = serializer.into_serializer().into_inner();

        Ok(bytes.to_vec())
    }

    #[cfg(feature = "rkyv")]
    pub fn load(bytes: &[u8]) -> Result<Self> {
        Ok(from_bytes(bytes).map_err(|_| StaticError("failed to deserialize database"))?)
    }

    #[cfg(feature = "postcard")]
    pub fn dump(&self) -> Result<Vec<u8>> {
        to_stdvec(self).context("failed to serialize database")
    }

    #[cfg(feature = "postcard")]
    pub fn load(bytes: &[u8]) -> Result<Self> {
        from_bytes(bytes).context("failed to deserialize database")
    }
}
