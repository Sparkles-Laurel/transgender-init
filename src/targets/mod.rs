// targets/mod.rs - the module "targets"
// This module holds the types of targets
// Types of targets:
// 1. Services
// 2. Sockets
// 3. Timers
// 4. Devices
// 5. Mounts
// 6. Automounts
// 7. Swap devices
// 8. Paths
// 9. Slices (this one is used for grouping processes)

// the trait that all targets must implement.
pub trait Target {
    // Conducts the target.
    fn conduct(&self, args: InitArgs) -> Result<(), String>;
}