// A needs B | A -> B
// A uses B | <ignored>
// A wants B | A -> B (if impossible tree, it will be discarded)
// A before B | B -> A
// A after B | A -> B

use std::rc::Rc;

#[cfg(feature = "rkyv")]
use rkyv::Archive;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{RcUnit, UnitName};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "rkyv", derive(Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "rkyv", archive(check_bytes))]
#[derive(Debug, Clone)]
pub struct UnitInfo {
    pub name: UnitName,
    pub dependencies: Rc<Dependencies>,
}

impl UnitInfo {
    pub fn new(unit: &RcUnit) -> Self {
        Self {
            name: unit.borrow().name(),
            dependencies: Rc::new(unit.borrow().dependencies()),
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "rkyv", derive(Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "rkyv", archive(check_bytes))]
#[derive(Default, Debug, Clone)]
pub struct Dependencies {
    /// The unit requires a previous unit to be started before it.
    /// The unit will fail to start if any of its needs fail to start.
    pub needs: Vec<UnitName>,
    /// The unit uses another unit but doesn't require it.
    /// The used unit will not be started, however.
    pub uses: Vec<UnitName>,
    /// Similar to `uses` with the exception that wanted unit will be started.
    pub wants: Vec<UnitName>,
    /// The unit should run before another unit.
    pub before: Vec<UnitName>,
    /// The unit should run after another unit.
    pub after: Vec<UnitName>,
}

impl Dependencies {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn need(&mut self, dependency: UnitName) -> &mut Self {
        self.needs.push(dependency);
        self
    }

    pub fn uses(&mut self, dependency: UnitName) -> &mut Self {
        self.uses.push(dependency);
        self
    }

    pub fn want(&mut self, dependency: UnitName) -> &mut Self {
        self.wants.push(dependency);
        self
    }

    pub fn before(&mut self, dependency: UnitName) -> &mut Self {
        self.before.push(dependency);
        self
    }

    pub fn after(&mut self, dependency: UnitName) -> &mut Self {
        self.after.push(dependency);
        self
    }
}
