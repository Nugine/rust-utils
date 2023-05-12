#[macro_export]
macro_rules! cfg_group {
    ($($item:item)*) => {
        $($item)*
    }
}
