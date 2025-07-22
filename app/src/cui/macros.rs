/// Cui::outマクロ
macro_rules! cui_out {
    ($cui:expr, $($arg:tt)*) => ({
        $cui.out(format_args!($($arg)*))
    })
}

/// Cui::outマクロ(改行付き)
macro_rules! cui_outln {
    ($cui:expr) => (cui_out!($cui, "\n"));
    ($cui:expr, $($arg:tt)*) => ({
        $cui.outln(format_args!($($arg)*))
    })
}
