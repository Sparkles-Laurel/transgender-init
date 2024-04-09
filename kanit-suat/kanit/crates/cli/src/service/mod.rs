#[cfg(any(feature = "rkyv", feature = "postcard"))]
pub use disable::disable;
#[cfg(any(feature = "rkyv", feature = "postcard"))]
pub use enable::enable;
#[cfg(any(feature = "rkyv", feature = "postcard"))]
pub use list::list;

mod disable;
#[cfg(any(feature = "rkyv", feature = "postcard"))]
mod enable;
#[cfg(any(feature = "rkyv", feature = "postcard"))]
mod list;

#[cfg(not(any(feature = "rkyv", feature = "postcard")))]
compile_error!("feature `postcard` or `rkyv` is needed to compile with feature `service`");
