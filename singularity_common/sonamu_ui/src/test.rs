use crate::display_units::DisplayUnits;

#[test]
pub fn units_mapping_test() {
    let a = DisplayUnits::from_mixed(-1, 1.0);
    println!("`a` should represent one pixel less from full.");
    dbg!(&a);
    debug_assert_eq!(
        a.map_onto(DisplayUnits::Pixels(10), DisplayUnits::Pixels(20)),
        DisplayUnits::Pixels(19)
    );
}
