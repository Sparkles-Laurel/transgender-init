// since the loader is locked in the ev loop, only 1 reference to a unit can be held at await points
// in start/teardown, only 1 reference is held as well
#![allow(clippy::await_holding_refcell_ref)]

#[cfg(all(feature = "units", any(feature = "rkyv", feature = "postcard")))]
pub use control::*;

#[cfg(all(feature = "units", any(feature = "rkyv", feature = "postcard")))]
mod control;
#[cfg(any(feature = "rkyv", feature = "postcard"))]
pub mod db;
#[cfg(all(feature = "units", any(feature = "rkyv", feature = "postcard")))]
mod event;
#[cfg(any(feature = "rkyv", feature = "postcard"))]
mod loader;

#[cfg(not(any(feature = "rkyv", feature = "postcard")))]
compile_error!("rc requires feature 'rkyv' or 'postcard' selected");
