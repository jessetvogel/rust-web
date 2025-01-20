#[macro_export]
macro_rules! console_log {
    ($fmt:expr) => { js::invoke("console.log({})", &[format!($fmt).into()]); };
    ($fmt:expr, $($arg:tt)*) => { js::invoke("console.log({})", &[format!($fmt, $($arg)*).into()]); };
}

#[macro_export]
macro_rules! console_info {
    ($fmt:expr) => { js::invoke("console.info({})", &[format!($fmt).into()]); };
    ($fmt:expr, $($arg:tt)*) => { js::invoke("console.info({})", &[format!($fmt, $($arg)*).into()]); };
}

#[macro_export]
macro_rules! console_warn {
    ($fmt:expr) => { js::invoke("console.warn({})", &[format!($fmt).into()]); };
    ($fmt:expr, $($arg:tt)*) => { js::invoke("console.warn({})", &[format!($fmt, $($arg)*).into()]); };
}

#[macro_export]
macro_rules! console_error {
    ($fmt:expr) => { js::invoke("console.error({})", &[format!($fmt).into()]); };
    ($fmt:expr, $($arg:tt)*) => { js::invoke("console.error({})", &[format!($fmt, $($arg)*).into()]); };
}
