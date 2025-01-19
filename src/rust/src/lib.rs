pub mod callbacks;
pub mod js;

#[macro_export]
macro_rules! console_log {
    ($fmt:expr) => { js::invoke("console.log({})", &[format!($fmt).into()]); };
    ($fmt:expr, $($arg:tt)*) => { js::invoke("console.log({})", &[format!($fmt, $($arg)*).into()]); };
}
