use singularity_macros::{hashing_macro, jank_hashing_helper, jank_hashing_macro};

const HASH: u64 = hashing_macro!();
const JANK_HASH_HELPER: u64 = jank_hashing_helper!(module_path!());
const JANK_HASH: u64 = jank_hashing_macro!();

#[test]
fn print_hash() {
    dbg!(HASH);
    dbg!(JANK_HASH_HELPER);
    dbg!(JANK_HASH);
    dbg!(module_path!());
}
