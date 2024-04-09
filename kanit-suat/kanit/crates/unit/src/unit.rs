use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use async_trait::async_trait;
use send_wrapper::SendWrapper;

use kanit_common::error::Result;

use crate::Dependencies;

// has to be async lock to allow querying without possible blocks
pub type RcUnit = SendWrapper<Rc<RefCell<dyn Unit>>>;
pub type UnitName = Arc<str>;

/// Unit lifecycle:
///
///
/// Startup:
/// ```rs
/// if !unit.prepare().await? { return; }
///
/// unit.start().await?;
/// ```
///
/// Stop:
/// ```rs
/// unit.stop().await?;
/// unit.teardown().await?;
/// ```
///
/// Restart:
/// ```rs
/// unit.stop().await?;
/// unit.start().await?;
/// ```
#[async_trait]
pub trait Unit: Send + Sync {
    /// The name of the unit.
    fn name(&self) -> UnitName;

    /// A description of the unit.
    fn description(&self) -> Option<&str> {
        None
    }

    /// Dependencies of the unit.
    fn dependencies(&self) -> Dependencies {
        Dependencies::new()
    }

    /// Starts the unit.
    async fn start(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called when a unit is ordered to stop or as a step in restarting.
    async fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    /// Preconditions for starting a unit.
    async fn prepare(&self) -> Result<bool> {
        Ok(true)
    }

    /// Tearing down a unit once finished.
    async fn teardown(&self) -> Result<()> {
        Ok(())
    }
}

pub fn wrap_unit<U: Unit + 'static>(unit: U) -> RcUnit {
    SendWrapper::new(Rc::new(RefCell::new(unit)))
}
