use std::sync::OnceLock;

static VERBOSE: OnceLock<bool> = OnceLock::new();

pub fn set_verbose(enabled: bool) {
    VERBOSE.set(enabled).ok();
}

pub fn is_verbose() -> bool {
    *VERBOSE.get().unwrap_or(&false)
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        if $crate::logging::is_verbose() {
            println!($($arg)*)
        }
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        eprintln!($($arg)*)
    };
}
