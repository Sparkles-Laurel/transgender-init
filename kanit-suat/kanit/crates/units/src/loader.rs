use std::collections::HashSet;

use kanit_unit::{wrap_unit, RcUnit, UnitName};

use crate::oneshot::*;
use crate::services::*;

pub fn baked_units() -> [RcUnit; 16] {
    [
        wrap_unit(ProcFs),
        wrap_unit(SysFs),
        wrap_unit(Run),
        wrap_unit(DevFs),
        wrap_unit(MDev),
        wrap_unit(HwDrivers),
        wrap_unit(Modules),
        wrap_unit(Clock),
        wrap_unit(RootFs),
        wrap_unit(Swap),
        wrap_unit(LocalMount),
        wrap_unit(Seed),
        wrap_unit(Hostname),
        wrap_unit(Syslog::new()),
        wrap_unit(GeTTY::new("tty1", false)),
        wrap_unit(GeTTY::new("ttyS0", true)),
    ]
}

pub fn default_levels() -> Vec<HashSet<UnitName>> {
    let level_1 = baked_units()
        .iter()
        .map(|n| n.borrow().name())
        .filter(|n| !n.starts_with("getty"))
        .collect::<HashSet<_>>();
    #[cfg(not(feature = "testing"))]
    let level_2 = baked_units()
        .iter()
        .map(|n| n.borrow().name())
        .filter(|n| n.starts_with("getty"))
        .collect::<HashSet<_>>();
    #[cfg(feature = "testing")]
    let level_2 = HashSet::new();

    vec![level_1, level_2]
}
