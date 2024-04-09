#[cfg(feature = "units")]
use std::collections::HashMap;

#[cfg(feature = "units")]
use kanit_common::error::Result;
#[cfg(feature = "units")]
use kanit_units::{baked_units, default_levels};
#[cfg(feature = "units")]
pub use loader::Loader;
pub use sort::obtain_load_order;

#[cfg(feature = "units")]
use crate::db::Database;

#[allow(clippy::module_inception)]
#[cfg(feature = "units")]
mod loader;
mod sort;

#[cfg(feature = "units")]
fn default_database() -> Result<Database> {
    let mapped = baked_units()
        .iter()
        .map(|n| (n.borrow().name(), n.clone()))
        .collect::<HashMap<_, _>>();

    Database::new(HashMap::new(), mapped, default_levels())
}

#[cfg(feature = "units")]
pub fn init_loader() -> Result<()> {
    Loader::initialize(default_database)?;

    let mut loader = Loader::obtain()?.borrow_mut();

    loader.extend_map(
        baked_units()
            .iter()
            .map(|n| (n.borrow().name(), n.clone()))
            .collect::<Vec<_>>(),
    );

    Ok(())
}
