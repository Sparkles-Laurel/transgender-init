pub use logger::*;

mod logger;
mod scope;
#[cfg(feature = "tap")]
pub mod tap;
#[cfg(feature = "timings")]
pub mod timing;

#[cfg(not(feature = "tap"))]
pub mod tap {
    pub fn header() {}

    pub fn enter_subtest<S: ToString>(_: Option<S>) {}

    pub fn exit_subtest() {}

    pub fn plan(_: usize) {}

    pub fn bail<S: ToString>(_: Option<S>) {}

    pub fn ok<S: ToString>(_: usize, _: Option<S>) {}

    pub fn not_ok<S: ToString>(_: usize, _: Option<S>) {}
}

#[cfg(not(feature = "timings"))]
pub mod timing {
    use std::rc::Rc;

    pub use crate::scope::Scope;

    pub fn register() {}

    pub fn push_scope<S: ToString>(_name: S) -> usize {
        0
    }

    pub fn pop_scope(_id: usize) {}

    pub fn get_scopes() -> Rc<[Scope]> {
        Rc::from([])
    }
}
