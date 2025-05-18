#[macro_export]
macro_rules! log_err {
    ($expr:expr, $fmt:literal $(, $arg:expr)*) => {{
        use log::warn;
        $expr.map_err(|err| {
            warn!($fmt, err = err $(, $arg)*);
            err
        })
    }};
}