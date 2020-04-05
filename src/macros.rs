
#[macro_export(local_inner_macros)]
macro_rules! value {
    // Hide distracting implementation details from the generated rustdoc.
    ($($kv:tt)+) => {
        json!($($kv)+).into()
    };
}
