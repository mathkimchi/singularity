pub use singularity_macros::combine_events_macro;

#[macro_export]
macro_rules! combine_events {
    {$inner:tt} => {$crate::macro_reexports::combine_events_macro!(inner);}
}
