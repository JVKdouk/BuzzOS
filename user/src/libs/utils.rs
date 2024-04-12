/// Perform page rounding up, to the next page boundary
#[macro_export]
macro_rules! ROUND_UP {
    ($val:expr, $align:expr) => {
        ($val + $align - 1) & !($align - 1)
    };
}

/// Perform page rounding down, to the previous page boundary
#[macro_export]
macro_rules! ROUND_DOWN {
    ($val:expr, $align:expr) => {
        $val & !($align - 1)
    };
}
