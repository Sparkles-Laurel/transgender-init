pub use clock::Clock;
pub use devfs::DevFs;
pub use hostname::Hostname;
pub use hwdrivers::HwDrivers;
pub use localmount::LocalMount;
pub use mdev::MDev;
pub use modules::Modules;
pub use procfs::ProcFs;
pub use rootfs::RootFs;
pub use run::Run;
pub use seed::Seed;
pub use swap::Swap;
pub use sysfs::SysFs;

mod clock;
mod devfs;
mod hostname;
mod hwdrivers;
mod localmount;
mod mdev;
mod modules;
mod procfs;
mod rootfs;
mod run;
mod seed;
mod swap;
mod sysctl;
mod sysfs;

// deduplication of unit names
// as unit names are used as keys and things, having them deduplicated would be good
// it also allows comparisons to be O(1) by comparing addresses
#[macro_export]
macro_rules! unit_name {
    ($name:literal) => {
        fn name(&self) -> kanit_unit::UnitName {
            static NAME: std::sync::OnceLock<kanit_unit::UnitName> = std::sync::OnceLock::new();

            return NAME
                .get_or_init(|| kanit_unit::UnitName::from($name))
                .clone();
        }
    };
}
