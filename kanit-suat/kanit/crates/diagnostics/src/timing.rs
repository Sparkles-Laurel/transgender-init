use std::cell::RefCell;
use std::rc::Rc;
use std::sync::OnceLock;
use std::time::Instant;

use send_wrapper::SendWrapper;

pub use crate::scope::Scope;

static GLOBAL_TIMER: OnceLock<SendWrapper<RefCell<Timer>>> = OnceLock::new();

pub struct Timer {
    scopes: Vec<Scope>,
    level: usize,
}

pub fn register() {
    let _ = GLOBAL_TIMER.set(SendWrapper::new(RefCell::new(Timer {
        scopes: vec![],
        level: 0,
    })));
}

pub fn push_scope<S: ToString>(name: S) -> Option<usize> {
    if let Some(timer) = GLOBAL_TIMER.get() {
        let level = timer.borrow().level;
        let mut timer = timer.borrow_mut();

        timer.scopes.push(Scope {
            name: name.to_string(),
            start: Instant::now(),
            duration: None,
            level,
        });

        let id = timer.scopes.len() - 1;

        timer.level += 1;

        Some(id)
    } else {
        None
    }
}

pub fn pop_scope(id: Option<usize>) {
    if let Some(timer) = GLOBAL_TIMER.get() {
        let mut timer = timer.borrow_mut();

        if let Some(id) = id {
            let scope = &mut timer.scopes[id];

            scope.duration = Some(Instant::now() - scope.start);
        }

        timer.level -= 1;
    }
}

pub fn get_scopes() -> Rc<[Scope]> {
    if let Some(timer) = GLOBAL_TIMER.get() {
        timer.borrow().scopes.clone().into()
    } else {
        Rc::from([])
    }
}
