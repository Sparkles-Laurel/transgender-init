use std::sync::atomic::{AtomicUsize, Ordering};

static LEVEL: AtomicUsize = AtomicUsize::new(0);

fn print_leveled<S: ToString>(s: S) {
    println!(
        "{}{}",
        "    ".repeat(LEVEL.load(Ordering::Relaxed)),
        s.to_string()
    );
}

pub fn header() {
    println!("TAP version 14");
}

pub fn enter_subtest<S: ToString>(desc: Option<S>) {
    if let Some(desc) = desc {
        print_leveled(format!("# Subtest: {}", desc.to_string()))
    } else {
        print_leveled("# Subtest")
    }

    LEVEL.fetch_add(1, Ordering::Relaxed);
}

pub fn exit_subtest() {
    LEVEL.fetch_sub(1, Ordering::Relaxed);
}

pub fn plan(tests: usize) {
    print_leveled(format!("1..{}", tests))
}

pub fn bail<S: ToString>(desc: Option<S>) {
    if let Some(desc) = desc {
        print_leveled(format!("Bail out! {}", desc.to_string()))
    } else {
        print_leveled("Bail out!");
    }
}

pub fn ok<S: ToString>(test: usize, desc: Option<S>) {
    if let Some(desc) = desc {
        print_leveled(format!("ok {} - {}", test, desc.to_string()));
    } else {
        print_leveled(format!("ok {}", test));
    }
}

pub fn not_ok<S: ToString>(test: usize, desc: Option<S>) {
    if let Some(desc) = desc {
        print_leveled(format!("not ok {} - {}", test, desc.to_string()));
    } else {
        print_leveled(format!("not ok {}", test));
    }
}
